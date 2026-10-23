//! Render original Lottie art without opening a window or running JavaScript.
//! cargo run --example lottie_reference -- img/card/jetpack.json 238 72 0 /tmp/card.png
use std::{fs::File, io::BufWriter, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err("Expected asset-relative path, width, height, frame and output PNG".into());
    }
    let engine = thorvg::Thorvg::init(0)?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static");
    let mut clip = santa_tracker::animation::Clip::load(&engine, &root.join(&args[0]))?;
    let (width, height) = (args[1].parse()?, args[2].parse()?);
    let mut rgba = clip.rasterize(width, height, args[3].parse()?)?;
    // PNG stores straight alpha; keep the native renderer premultiplied.
    for p in rgba.chunks_exact_mut(4) {
        if p[3] > 0 {
            for c in 0..3 {
                p[c] = (u32::from(p[c]) * 255 / u32::from(p[3])).min(255) as u8;
            }
        }
    }
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(&args[4])?), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&rgba)?;
    Ok(())
}
