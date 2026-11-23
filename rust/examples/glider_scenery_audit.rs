//! CPU-only original scenery strip/seam audit. No window, GPU or audio context.
use santa_tracker::svg_tiles::{self, Plan, TILE_SIZE};
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../static/scenes/glider/img");
    let mut rows = vec![];
    for name in ["stars-tile.svg", "bg-mountains.svg"] {
        let bytes = std::fs::read(root.join(name)).unwrap();
        let tree = resvg::usvg::Tree::from_data(&bytes, &Default::default()).unwrap();
        for scale in [1, 2, 3, 4] {
            let plan = Plan::new(
                [tree.size().width() as u32, tree.size().height() as u32],
                scale,
                TILE_SIZE,
            )
            .unwrap();
            let full = plan.full();
            let reference = if scale <= 2 {
                let mut p = resvg::tiny_skia::Pixmap::new(full[0], full[1]).unwrap();
                resvg::render(
                    &tree,
                    resvg::tiny_skia::Transform::from_scale(scale as f32, scale as f32),
                    &mut p.as_mut(),
                );
                Some(p)
            } else {
                None
            };
            let mut rasterizer = svg_tiles::Rasterizer::new(
                resvg::usvg::Tree::from_data(&bytes, &Default::default()).unwrap(),
            );
            let mut previous: Option<santa_tracker::svg_crop::Image> = None;
            let mut seam_channels = 0usize;
            let mut differences = 0usize;
            let mut maximum = 0u8;
            let mut ms = 0.;
            let mut maximum_surface = 0usize;
            for x in 0..full[0].div_ceil(TILE_SIZE) {
                let start = std::time::Instant::now();
                let tile = rasterizer.rasterize(plan, [x, 0]).unwrap();
                ms += start.elapsed().as_secs_f64() * 1000.;
                maximum_surface = maximum_surface.max(tile.rgba.len());
                assert!(tile
                    .rgba
                    .chunks_exact(4)
                    .all(|p| p[..3].iter().all(|c| *c <= p[3])));
                if let Some(left) = &previous {
                    for y in 0..tile.height {
                        for (lx, rx) in [(left.width - 2, 0), (left.width - 1, 1)] {
                            let a = ((y * left.width + lx) * 4) as usize;
                            let b = ((y * tile.width + rx) * 4) as usize;
                            assert_eq!(
                                &left.rgba[a..a + 4],
                                &tile.rgba[b..b + 4],
                                "{name} scale{scale} seam{x} y{y}"
                            );
                            seam_channels += 4;
                        }
                    }
                }
                if let Some(reference) = &reference {
                    let core = plan.core([x, 0]).unwrap();
                    for y in 0..tile.height {
                        for ox in 0..tile.width {
                            let rx = (core[0] as i64 + ox as i64 - 1).clamp(0, full[0] as i64 - 1)
                                as u32;
                            let ry = (y as i64 - 1).clamp(0, full[1] as i64 - 1) as u32;
                            for c in 0..4 {
                                let a = tile.rgba[((y * tile.width + ox) * 4 + c) as usize];
                                let b = reference.data()[((ry * full[0] + rx) * 4 + c) as usize];
                                let d = a.abs_diff(b);
                                maximum = maximum.max(d);
                                if d > 0 {
                                    differences += 1;
                                }
                            }
                        }
                    }
                }
                previous = Some(tile);
            }
            if reference.is_some() {
                if name == "bg-mountains.svg" && scale == 2 {
                    assert!(maximum <= 2 && differences <= 10);
                } else {
                    assert_eq!(maximum, 0);
                }
            }
            rows.push(serde_json::json!({"source":name,"scale":scale,"full_physical":full,"strips":full[0].div_ceil(TILE_SIZE),"largest_uploaded_surface_bytes":maximum_surface,"seam_channels_compared":seam_channels,"maximum_seam_error":0,"full_reference_compared":reference.is_some(),"maximum_full_reference_error":reference.as_ref().map(|_|maximum),"changed_full_reference_channels":reference.as_ref().map(|_|differences),"cpu_all_strip_raster_ms":ms}));
        }
    }
    let report = serde_json::json!({"scope":"Actual production worker-owned cached narrow/full-height SVG strip rasterizer for original stars/mountains, all horizontal joins at scales1/2/3/4, associated bytes, root clamp and resource sizes. Full independent resvg surface compared at1/2 only. Higher-scale full surfaces not allocated. All strips rendered here for audit; runtime requests visible strips. Times single CPU samples include render/allocation/copy, exclude independent reference and GPU/driver/cache/runtime/focus/input. No claim of Chromium/native GPU pixel equivalence.","rows":rows});
    if let Some(out) = std::env::args().nth(1) {
        std::fs::write(out, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
    println!("8 full-height scenery strip cases passed");
}
