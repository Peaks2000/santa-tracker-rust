//! CPU-only Snowball image decode audit. Does not create a window or GPU context.
#![allow(dead_code)]
mod assets {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets.rs"));
    pub fn cpu_decode(root: &std::path::Path, key: &str) -> Result<Image, String> {
        decode_image(root, key)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../static");
    let output = std::env::args().nth(1).map(std::path::PathBuf::from);
    if let Some(path) = &output {
        std::fs::create_dir_all(path)?;
    }
    let mut rows = vec![];
    for (name, crop_fraction, display) in [
        ("loading-cloud.svg", 1., [256. * 117.8 / 70.7, 256.]),
        ("elves-parachuting.svg", 0.5, [160., 160.]),
    ] {
        let key = format!("scenes/snowball/img/{name}");
        let image = assets::cpu_decode(&root, &key)?;
        let tree = resvg::usvg::Tree::from_data(
            &std::fs::read(root.join(&key))?,
            &resvg::usvg::Options::default(),
        )?;
        if let Some(path) = &output {
            // CPU decoded images are already top-down. Image.export_png flips
            // GPU readbacks, so encode these bytes directly instead.
            let file = std::fs::File::create(path.join(format!("{name}.png")))?;
            let mut encoder = png::Encoder::new(file, image.width as u32, image.height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(&image.bytes)?;
        }
        let crop = [
            image.width as f64 * crop_fraction,
            image.height as f64 * crop_fraction,
        ];
        rows.push(serde_json::json!({
            "file":key,"svg_size":[tree.size().width(),tree.size().height()],
            "decoded_size":[image.width,image.height],"source_crop_size":crop,
            "display_size_desktop":display,
            "samples_per_display_pixel_1x":[crop[0]/display[0],crop[1]/display[1]],
            "samples_per_display_pixel_2x":[crop[0]/(display[0]*2.),crop[1]/(display[1]*2.)]
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "method":"Actual native CPU SVG decoder with source CSS desktop display/crop sizes. No window, GPU, OS input or browser rendering. Sample ratios establish bitmap undersampling, not browser/native pixel parity or a diagnosis of all texture clarity.",
            "images":rows
        }))?
    );
    Ok(())
}
