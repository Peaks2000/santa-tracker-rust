//! Sample an original SVG viewport region without allocating the whole atlas.
//! Output remains associated RGBA. Selected Glider crops use integer scales.
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
pub fn rasterize(tree: &resvg::usvg::Tree, crop: [f32; 4], scale: f32) -> Result<Image, String> {
    if !crop.iter().all(|v| v.is_finite())
        || !scale.is_finite()
        || scale <= 0.
        || crop[2] <= 0.
        || crop[3] <= 0.
    {
        return Err("Invalid SVG crop".into());
    }
    if !(crop[0] * scale).is_finite() || !(crop[1] * scale).is_finite() {
        return Err("Invalid SVG crop origin".into());
    }
    let width = (crop[2] * scale).ceil();
    let height = (crop[3] * scale).ceil();
    if width > 16384. || height > 16384. {
        return Err("SVG crop exceeds texture limit".into());
    }
    let (width, height) = (width as u32, height as u32);
    let mut pixels =
        resvg::tiny_skia::Pixmap::new(width, height).ok_or("SVG crop allocation failed")?;
    resvg::render(
        tree,
        resvg::tiny_skia::Transform::from_row(
            scale,
            0.,
            0.,
            scale,
            -crop[0] * scale,
            -crop[1] * scale,
        ),
        &mut pixels.as_mut(),
    );
    Ok(Image {
        width,
        height,
        rgba: pixels.data().to_vec(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_collision_crop_matches_full_atlas_and_keeps_transparent_edges() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../static")
            .join(crate::glider_sampling::collision_source());
        let tree = resvg::usvg::Tree::from_data(&std::fs::read(path).unwrap(), &Default::default())
            .unwrap();
        let crop = crate::glider_sampling::collision_crop();
        for scale in [1., 3., 4.] {
            let selected = rasterize(&tree, crop, scale).unwrap();
            let mut atlas = resvg::tiny_skia::Pixmap::new(
                (tree.size().width() * scale) as u32,
                (tree.size().height() * scale) as u32,
            )
            .unwrap();
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::from_scale(scale, scale),
                &mut atlas.as_mut(),
            );
            for y in 0..selected.height as usize {
                let start = ((y + (crop[1] * scale) as usize) * atlas.width() as usize
                    + (crop[0] * scale) as usize)
                    * 4;
                assert_eq!(
                    &selected.rgba
                        [y * selected.width as usize * 4..(y + 1) * selected.width as usize * 4],
                    &atlas.data()[start..start + selected.width as usize * 4],
                    "scale{scale} row{y}"
                );
            }
            let mut edge_max = 0;
            for y in 0..selected.height as usize {
                for x in 0..selected.width as usize {
                    if x == 0
                        || y == 0
                        || x + 1 == selected.width as usize
                        || y + 1 == selected.height as usize
                    {
                        edge_max =
                            edge_max.max(selected.rgba[(y * selected.width as usize + x) * 4 + 3]);
                    }
                }
            }
            assert_eq!(
                edge_max, 0,
                "crop needs a guard band if source art reaches the edge"
            );
        }
    }
    #[test]
    fn rejects_invalid_or_oversized_crop_before_allocating() {
        let tree = resvg::usvg::Tree::from_data(
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"/>"#,
            &Default::default(),
        )
        .unwrap();
        for (crop, scale) in [
            ([0., 0., 0., 10.], 1.),
            ([0., 0., 10., 10.], 0.),
            ([0., 0., 10., 10.], f32::NAN),
            ([0., 0., 10000., 10.], 2.),
            ([f32::MAX, 0., 1e-20, 1e-20], 1e20),
        ] {
            assert!(rasterize(&tree, crop, scale).is_err());
        }
    }
}
