//! CPU-only check of production Glider vector sampling at source desktop scales and display densities.
#![allow(dead_code)]
mod assets {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets.rs"));
    pub fn sample(root: &std::path::Path, key: &str, density: f32) -> (Image, Vec2) {
        let decoded = decode_with_metrics(root, key, Some(density)).unwrap();
        (decoded.image, decoded.logical_size)
    }
}
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../static");
    let policy: serde_json::Value =
        serde_json::from_str(include_str!("../data/glider-svg-sampling.json")).unwrap();
    let mut rows = vec![];
    let mut totals = vec![];
    for (width, density) in [
        (1500., 1.),
        (1500., 1.25),
        (1500., 2.),
        (1500., 3.),
        (375., 2.),
        (980., 1.25),
        (1440., 2.),
        (1920., 3.),
    ] {
        let effective_density =
            santa_tracker::glider_sampling::density_for_viewport(width, density);
        let mut bytes = 0usize;
        let mut elapsed = 0.;
        for name in policy["boxes"].as_object().unwrap().keys() {
            let key = format!("scenes/glider/img/{name}");
            let data = std::fs::read(root.join(&key)).unwrap();
            let tree = resvg::usvg::Tree::from_data(&data, &Default::default()).unwrap();
            let natural = [tree.size().width(), tree.size().height()];
            let required = santa_tracker::glider_sampling::fitted_maximum(&key, natural)
                .unwrap()
                .map(|v| v * effective_density);
            let start = std::time::Instant::now();
            let (image, logical) = assets::sample(&root, &key, effective_density);
            let ms = start.elapsed().as_secs_f64() * 1000.;
            bytes += image.bytes.len();
            elapsed += ms;
            assert_eq!([logical.x, logical.y], natural);
            assert!(
                image.width as f32 + 0.001 >= required[0]
                    && image.height as f32 + 0.001 >= required[1]
            );
            assert!(image
                .bytes
                .chunks_exact(4)
                .all(|p| p[..3].iter().all(|c| *c <= p[3])));
            let coverage = image
                .bytes
                .chunks_exact(4)
                .map(|p| p[3] as f64 / 255.)
                .sum::<f64>()
                / (image.width as f64 * image.height as f64);
            // Independent resvg target at the actual chosen backing scale.
            let mut reference =
                resvg::tiny_skia::Pixmap::new(image.width as u32, image.height as u32).unwrap();
            let factor = image.width as f32 / natural[0];
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::from_scale(factor, factor),
                &mut reference.as_mut(),
            );
            assert_eq!(image.bytes, reference.data());
            rows.push(serde_json::json!({"file":key,"width":width,"density":density,"effective_density":effective_density,"natural":natural,"logical":natural,"required_maximum_physical":required,"physical":[image.width,image.height],"scale":factor,"alpha_coverage":coverage,"cpu_decode_ms":ms}));
        }
        totals.push(
            serde_json::json!({"width":width,"density":density,"rgba_bytes":bytes,"cpu_decode_ms_sum":elapsed}),
        );
    }
    let report = serde_json::json!({"scope":"45 selected original Glider SVGs across eight desktop viewport/display-density combinations: actual production CPU decoder, associated bytes, logical geometry and physical coverage. Maximum desktop CSS em1.25 policy normalized by actual source desktop scale; baseline atlas3 retained. Declaration-derived policy, not a browser/GPU raster/performance oracle. Single decode timing samples include parse/raster/allocation; independent reference excluded. No runtime upload/DPI-switch or input-latency proof. Wide stars/mountains and dedicated collision crop have separate audits.","totals":totals,"rows":rows});
    if let Some(out) = std::env::args().nth(1) {
        std::fs::write(out, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
    println!("360 production SVG density cases passed");
}
