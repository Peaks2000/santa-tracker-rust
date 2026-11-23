//! CPU-only production crop/density audit. Never opens a GPU context or window.
#![allow(dead_code)]
mod assets {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets.rs"));
    pub fn sample(root: &std::path::Path, density: f32) -> (Image, Vec2) {
        let decoded = decode_with_metrics(
            root,
            santa_tracker::glider_sampling::COLLISION_KEY,
            Some(density),
        )
        .unwrap();
        (decoded.image, decoded.logical_size)
    }
}
fn main() {
    use santa_tracker::glider_sampling as sampling;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../static");
    let source = std::fs::read(root.join(sampling::collision_source())).unwrap();
    let tree = resvg::usvg::Tree::from_data(&source, &Default::default()).unwrap();
    let crop = sampling::collision_crop();
    let mut rows = vec![];
    let mut previous_coverage: Option<f64> = None;
    for (width, density) in [
        (375., 2.),
        (980., 1.),
        (1200., 1.25),
        (1440., 1.),
        (1440., 2.),
        (1500., 2.),
        (1920., 3.),
        (3440., 3.),
    ] {
        let effective = sampling::density_for_viewport(width, density);
        let required = sampling::fitted_maximum(sampling::COLLISION_KEY, [crop[2], crop[3]])
            .unwrap()
            .map(|v| v * effective);
        let start = std::time::Instant::now();
        let (image, logical) = assets::sample(&root, effective);
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.;
        assert_eq!([logical.x, logical.y], [crop[2], crop[3]]);
        assert!(image.width as f32 >= required[0] && image.height as f32 >= required[1]);
        let scale = image.width as f32 / crop[2];
        let mut reference =
            resvg::tiny_skia::Pixmap::new(image.width as u32, image.height as u32).unwrap();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_row(
                scale,
                0.,
                0.,
                scale,
                -crop[0] * scale,
                -crop[1] * scale,
            ),
            &mut reference.as_mut(),
        );
        assert_eq!(image.bytes, reference.data());
        assert!(image
            .bytes
            .chunks_exact(4)
            .all(|p| p[..3].iter().all(|c| *c <= p[3])));
        let mut edge_alpha = 0;
        for y in 0..image.height as usize {
            for x in 0..image.width as usize {
                if x == 0
                    || y == 0
                    || x + 1 == image.width as usize
                    || y + 1 == image.height as usize
                {
                    edge_alpha =
                        edge_alpha.max(image.bytes[(y * image.width as usize + x) * 4 + 3]);
                }
            }
        }
        assert_eq!(
            edge_alpha, 0,
            "original transparent margin must survive crop"
        );
        let coverage = image
            .bytes
            .chunks_exact(4)
            .map(|p| p[3] as f64 / 255.)
            .sum::<f64>()
            / (image.width as f64 * image.height as f64);
        if let Some(previous) = previous_coverage {
            assert!((coverage - previous).abs() < 0.005);
        }
        previous_coverage = Some(coverage);
        rows.push(serde_json::json!({"width":width,"density":density,"effective_density":effective,"logical_crop":crop,"peak_physical_required":required,"physical":[image.width,image.height],"backing_scale":scale,"rgba_bytes":image.bytes.len(),"full_atlas_same_scale_rgba_bytes":(400.*600.*scale*scale*4.) as usize,"edge_alpha":edge_alpha,"alpha_coverage":coverage,"cpu_decode_ms":elapsed_ms}));
    }
    let report = serde_json::json!({"scope":"Actual production cropped Glider SVG decoder at eight desktop width/DPR pairs, independent translated resvg raster, associated bytes, unchanged logical crop, transparent outer edges, peak-scale coverage and resource estimates. Full atlas crop equality at scales1/3/4 is checked separately by svg_crop unit test. Not browser/GPU pixels, compositor animation raster policy, runtime cache swaps, frame pacing or input latency. No window/device/input/focus/audio.","source":sampling::collision_source(),"crop":crop,"source_peak_scale":sampling::collision_peak_scale(),"rows":rows});
    if let Some(out) = std::env::args().nth(1) {
        std::fs::write(out, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
    println!("8 production collision crop/density cases passed");
}
