//! Narrow full-height SVG strips with bilinear guards and shared image geometry.
//! Each core pixel belongs to one tile; filtering reads the original neighbours.
use crate::svg_crop::Image;
pub const TILE_SIZE: u32 = 512;
pub const GUARD: u32 = 1;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Plan {
    natural: [u32; 2],
    scale: u32,
    tile_size: u32,
}
impl Plan {
    pub fn new(natural: [u32; 2], scale: u32, tile_size: u32) -> Result<Self, String> {
        if natural.contains(&0)
            || scale == 0
            || tile_size < 2
            || tile_size > 1024
            || natural.iter().any(|v| v.checked_mul(scale).is_none())
            || natural[1].checked_mul(scale).is_some_and(|h| h > 16382)
        {
            return Err("Invalid SVG tile plan".into());
        }
        Ok(Self {
            natural,
            scale,
            tile_size,
        })
    }
    pub fn full(self) -> [u32; 2] {
        self.natural.map(|v| v * self.scale)
    }
    pub fn core(self, at: [u32; 2]) -> Option<[u32; 4]> {
        let [w, h] = self.full();
        let x = at[0].checked_mul(self.tile_size)?;
        if x >= w || at[1] != 0 {
            return None;
        }
        Some([x, 0, (w - x).min(self.tile_size), h])
    }
    pub fn visible(self, image: [f64; 4], viewport: [f64; 4]) -> Vec<[u32; 2]> {
        if !image.iter().chain(viewport.iter()).all(|v| v.is_finite())
            || image[2] <= 0.
            || image[3] <= 0.
            || viewport[2] <= 0.
            || viewport[3] <= 0.
        {
            return vec![];
        }
        let left = image[0].max(viewport[0]);
        let top = image[1].max(viewport[1]);
        let right = (image[0] + image[2]).min(viewport[0] + viewport[2]);
        let bottom = (image[1] + image[3]).min(viewport[1] + viewport[3]);
        if left >= right || top >= bottom {
            return vec![];
        }
        let full = self.full();
        let step = self.tile_size as f64;
        let first = (((left - image[0]) / image[2]) * full[0] as f64 / step).floor() as u32;
        let end = ((((right - image[0]) / image[2]) * full[0] as f64 / step).ceil() as u32)
            .min(full[0].div_ceil(self.tile_size));
        (first..end).map(|x| [x, 0]).collect()
    }

    /// Boundaries use the same expression for both neighbours, avoiding cracks
    /// from accumulating fractional tile widths in logical screen coordinates.
    pub fn bounds(self, at: [u32; 2], image: [f64; 4]) -> Option<[f64; 4]> {
        let [x, y, w, h] = self.core(at)?;
        let full = self.full();
        Some([
            image[0] + x as f64 / full[0] as f64 * image[2],
            image[1] + y as f64 / full[1] as f64 * image[3],
            image[0] + (x + w) as f64 / full[0] as f64 * image[2],
            image[1] + (y + h) as f64 / full[1] as f64 * image[3],
        ])
    }
}
fn rasterize_body(tree: &resvg::usvg::Tree, plan: Plan, at: [u32; 2]) -> Result<Image, String> {
    if [tree.size().width(), tree.size().height()] != plan.natural.map(|v| v as f32) {
        return Err("SVG tile source dimensions changed".into());
    }
    let [x, y, w, h] = plan.core(at).ok_or("SVG tile outside source")?;
    // Render two extra pixels outside the bilinear guard, then trim them. This
    // keeps guard samples away from software rasterizer surface clipping.
    let margin = GUARD + 2;
    // Full-height strips preserve the original Y raster coordinates. Splitting
    // Y introduced AA differences at horizontal joins in the source mountains.
    let full = plan.full();
    if full[1] > 16382 {
        return Err("SVG strip exceeds texture height limit".into());
    }
    let mut raster = resvg::tiny_skia::Pixmap::new(w + 2 * margin, full[1])
        .ok_or("SVG strip allocation failed")?;
    resvg::render(
        tree,
        resvg::tiny_skia::Transform::from_row(
            plan.scale as f32,
            0.,
            0.,
            plan.scale as f32,
            margin as f32 - x as f32,
            0.,
        ),
        &mut raster.as_mut(),
    );
    let out_w = w + 2 * GUARD;
    let out_h = h + 2 * GUARD;
    let mut rgba = vec![0; (out_w * out_h * 4) as usize];
    let full = plan.full();
    for oy in 0..out_h {
        for ox in 0..out_w {
            // Match the old full texture's clamp-to-edge sampler at the root frame.
            let absolute_x = (x as i64 + ox as i64 - GUARD as i64).clamp(0, full[0] as i64 - 1);
            let absolute_y = (y as i64 + oy as i64 - GUARD as i64).clamp(0, full[1] as i64 - 1);
            let rx = (absolute_x - x as i64 + margin as i64) as u32;
            let ry = absolute_y as u32;
            let src = ((ry * raster.width() + rx) * 4) as usize;
            let dst = ((oy * out_w + ox) * 4) as usize;
            rgba[dst..dst + 4].copy_from_slice(&raster.data()[src..src + 4]);
        }
    }
    Ok(Image {
        width: out_w,
        height: out_h,
        rgba,
    })
}
/// The shared two-column overlap is sampled from the right strip's original
/// SVG raster. Independent float origins can otherwise disagree at a join.
/// No colour averaging or alpha compositing is used.
pub fn rasterize(tree: &resvg::usvg::Tree, plan: Plan, at: [u32; 2]) -> Result<Image, String> {
    let mut image = rasterize_body(tree, plan, at)?;
    let next = [at[0] + 1, 0];
    if plan.core(next).is_some() {
        let right = rasterize_body(tree, plan, next)?;
        for y in 0..image.height {
            let dst = ((y * image.width + image.width - 2) * 4) as usize;
            let src = (y * right.width * 4) as usize;
            image.rgba[dst..dst + 8].copy_from_slice(&right.rgba[src..src + 8]);
        }
    }
    Ok(image)
}
/// A worker-owned source and one cached neighbouring raw strip. Sequential
/// requests reuse the overlap render; plan changes/out-of-order requests miss.
pub struct Rasterizer {
    tree: resvg::usvg::Tree,
    next: Option<(Plan, [u32; 2], Image)>,
}
impl Rasterizer {
    pub fn new(tree: resvg::usvg::Tree) -> Self {
        Self { tree, next: None }
    }
    pub fn rasterize(&mut self, plan: Plan, at: [u32; 2]) -> Result<Image, String> {
        let mut image = match self.next.take() {
            Some((p, a, image)) if p == plan && a == at => image,
            _ => rasterize_body(&self.tree, plan, at)?,
        };
        let next = [at[0] + 1, 0];
        if plan.core(next).is_some() {
            let right = rasterize_body(&self.tree, plan, next)?;
            for y in 0..image.height {
                let dst = ((y * image.width + image.width - 2) * 4) as usize;
                let src = (y * right.width * 4) as usize;
                image.rgba[dst..dst + 8].copy_from_slice(&right.rgba[src..src + 8]);
            }
            self.next = Some((plan, next, right));
        }
        Ok(image)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_worker_strip_rasterizer_matches_uncached_after_jumps_and_scale_changes() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../static/scenes/glider/img/stars-tile.svg");
        let data = std::fs::read(path).unwrap();
        let tree = resvg::usvg::Tree::from_data(&data, &Default::default()).unwrap();
        let mut cached =
            Rasterizer::new(resvg::usvg::Tree::from_data(&data, &Default::default()).unwrap());
        for (scale, x) in [(1, 2), (1, 3), (1, 0), (1, 1), (2, 0), (2, 1), (1, 0)] {
            let plan = Plan::new([5500, 530], scale, TILE_SIZE).unwrap();
            assert_eq!(
                cached.rasterize(plan, [x, 0]).unwrap().rgba,
                rasterize(&tree, plan, [x, 0]).unwrap().rgba
            );
        }
    }
    #[test]
    fn selected_tiles_track_full_source_with_recorded_interior_rounding() {
        for name in ["stars-tile.svg", "bg-mountains.svg"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../static/scenes/glider/img")
                .join(name);
            let tree =
                resvg::usvg::Tree::from_data(&std::fs::read(path).unwrap(), &Default::default())
                    .unwrap();
            for scale in [1, 2] {
                let plan = Plan::new(
                    [tree.size().width() as u32, tree.size().height() as u32],
                    scale,
                    TILE_SIZE,
                )
                .unwrap();
                let full = plan.full();
                let mut reference = resvg::tiny_skia::Pixmap::new(full[0], full[1]).unwrap();
                resvg::render(
                    &tree,
                    resvg::tiny_skia::Transform::from_scale(scale as f32, scale as f32),
                    &mut reference.as_mut(),
                );
                let last = [(full[0] - 1) / TILE_SIZE, 0];
                for at in [[0, 0], [last[0] / 2, 0], [last[0], last[1]]] {
                    let tile = rasterize(&tree, plan, at).unwrap();
                    let core = plan.core(at).unwrap();
                    let mut max = 0;
                    let mut changed = 0;
                    let mut differences = vec![];
                    for y in 0..tile.height {
                        for x in 0..tile.width {
                            let rx =
                                (core[0] as i64 + x as i64 - 1).clamp(0, full[0] as i64 - 1) as u32;
                            let ry =
                                (core[1] as i64 + y as i64 - 1).clamp(0, full[1] as i64 - 1) as u32;
                            for c in 0..4 {
                                let a = tile.rgba[((y * tile.width + x) * 4 + c) as usize];
                                let b = reference.data()[((ry * full[0] + rx) * 4 + c) as usize];
                                max = max.max(a.abs_diff(b));
                                if a != b {
                                    changed += 1;
                                    assert!(
                                        x > 0 && y > 0 && x + 1 < tile.width && y + 1 < tile.height,
                                        "guard must match full source"
                                    );
                                    if differences.len() < 12 {
                                        differences.push((x, y, c, a, b));
                                    }
                                }
                            }
                        }
                    }
                    if name == "bg-mountains.svg" && scale == 2 && at == last {
                        // Independent resvg surface origins change six interior
                        // AA channels by <=2. Guards/seams still must be exact.
                        assert!(max <= 2 && changed <= 6, "{differences:?}");
                    } else {
                        assert_eq!(
                            max, 0,
                            "{name} scale{scale} tile{at:?}, changed{changed} {differences:?}"
                        );
                    }
                }
                let rect = [-14.125, 57.25, 1440.5, 300.25];
                let a = plan.bounds([0, 0], rect).unwrap();
                let b = plan.bounds([1, 0], rect).unwrap();
                assert_eq!(a[2], b[0]);
            }
        }
    }
    #[test]
    fn original_scenery_adjacent_tile_guards_share_identical_samples() {
        for name in ["stars-tile.svg", "bg-mountains.svg"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../static/scenes/glider/img")
                .join(name);
            let tree =
                resvg::usvg::Tree::from_data(&std::fs::read(path).unwrap(), &Default::default())
                    .unwrap();
            for scale in [1, 2] {
                let plan = Plan::new(
                    [tree.size().width() as u32, tree.size().height() as u32],
                    scale,
                    TILE_SIZE,
                )
                .unwrap();
                let full = plan.full();
                for ty in 0..1 {
                    let mut previous: Option<Image> = None;
                    for tx in 0..full[0].div_ceil(TILE_SIZE) {
                        let image = rasterize(&tree, plan, [tx, ty]).unwrap();
                        if let Some(left) = &previous {
                            for y in 0..image.height {
                                for (lx, rx) in [(left.width - 2, 0), (left.width - 1, 1)] {
                                    let a = ((y * left.width + lx) * 4) as usize;
                                    let b = ((y * image.width + rx) * 4) as usize;
                                    assert_eq!(
                                        &left.rgba[a..a + 4],
                                        &image.rgba[b..b + 4],
                                        "{name} scale{scale} seam{tx}/{ty} y{y}"
                                    );
                                }
                            }
                        }
                        previous = Some(image);
                    }
                }
            }
        }
    }
    #[test]
    fn source_strips_preserve_full_height_without_vertical_partitions() {
        let plan = Plan::new([10000, 564], 3, 512).unwrap();
        assert_eq!(plan.core([2, 0]), Some([1024, 0, 512, 1692]));
        assert_eq!(plan.core([2, 1]), None);
        assert_eq!(
            plan.visible([0., 0., 1000., 600.], [0., 550., 10., 10.]),
            vec![[0, 0]]
        );
    }
    #[test]
    fn visibility_clips_tiles_without_losing_fractional_edges() {
        let plan = Plan::new([1000, 600], 2, 512).unwrap();
        assert_eq!(
            plan.visible([0., 0., 1000., 600.], [0., 0., 256., 256.]),
            vec![[0, 0]]
        );
        assert_eq!(
            plan.visible([-0.5, 0., 1000., 600.], [0., 0., 256., 256.]),
            vec![[0, 0], [1, 0]]
        );
        assert!(plan
            .visible([1001., 0., 1000., 600.], [0., 0., 1000., 600.])
            .is_empty());
        assert!(Plan::new([u32::MAX, 1], 2, 512).is_err());
    }
}
