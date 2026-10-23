//! Native PNG export with reported filesystem errors.
use macroquad::prelude::Image;
use std::{io::BufWriter, path::Path};
pub fn save_png(image: &Image, path: &Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(
        BufWriter::new(file),
        image.width as u32,
        image.height as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    // OpenGL readback starts at the bottom row.
    let stride = image.width as usize * 4;
    let bytes: Vec<u8> = image
        .bytes
        .chunks_exact(stride)
        .rev()
        .flatten()
        .copied()
        .collect();
    writer.write_image_data(&bytes).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}
