//! CPU-only sampling audit of original Selfie CSS SVGs; no window/device.
use santa_tracker::{raster_density, selfie_vector::Raster};
fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("Pass original asset root"));
    let mut rows = vec![];
    for (name, logical_w, logical_h) in [
        ("face/eye.svg", 35., 35.),
        ("face/eye--blink.svg", 35., 35.),
        ("face/eye--closed.svg", 35., 35.),
        ("face/nose.svg", 127., 104.),
        ("face/mouth--breath.svg", 57., 57.),
        ("face/mouth--smile.svg", 90., 45.),
        ("santa/face.svg", 558., 497.),
        ("santa/body.svg", 1052., 1068.),
    ] {
        let bytes = std::fs::read(root.join("scenes/santaselfie/img").join(name)).unwrap();
        let raster = Raster::new(&bytes).unwrap();
        let natural = raster.natural_size();
        let mut baseline: Option<f64> = None;
        for (scale, density) in [(0.8, 1.), (0.8, 2.), (1., 1.), (1., 2.)] {
            let fit = (logical_w * scale / natural.0).min(logical_h * scale / natural.1);
            let (w, h) = raster_density::size(natural.0 * fit, natural.1 * fit, density);
            let rgba = raster.pixels(w, h).unwrap();
            assert!(rgba
                .chunks_exact(4)
                .all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]));
            let coverage = rgba
                .chunks_exact(4)
                .map(|p| p[3] as f64 / 255.)
                .sum::<f64>()
                / (w * h) as f64;
            if let Some(value) = baseline {
                assert!((coverage - value).abs() < 0.01, "{name} coverage changed");
            }
            baseline = Some(coverage);
            rows.push(serde_json::json!({"name":name,"source_natural":natural,"scale":scale,"density":density,"fitted_logical":[natural.0*fit,natural.1*fit],"physical":[w,h],"old_magnification":fit*density,"alpha_coverage":coverage}));
        }
    }
    let report = serde_json::json!({"scope":"Actual original SVGs rendered by production native CPU sampler. Dimensions/associated alpha/normalized coverage only; not browser pixels, GPU interpolation, runtime cache interaction or performance.","rows":rows,"original_beard_canvas_retained":true});
    let text = serde_json::to_string_pretty(&report).unwrap() + "\n";
    if let Some(path) = std::env::args().nth(2) {
        std::fs::write(path, &text).unwrap();
    }
    println!("32 original SVG sampling cases passed");
}
