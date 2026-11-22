//! Native replay of Elf.generateElfTexture's Canvas2D drawing operations.
//! Copyright 2020 Google LLC, Apache-2.0. Palette/cache rules are source behavior.
use resvg::tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect, Transform};
pub const MAJOR: [u32; 6] = [0x84488f, 0xd55040, 0x4db1e9, 0x55ab57, 0x56bbca, 0xbc3eb4];
pub const SKIN: [u32; 4] = [0xe7c241, 0x936644, 0xb8906d, 0xdbbc99];
pub const HAIR: [u32; 4] = [0x2f3030, 0x7c4149, 0xab2923, 0xea9639];
/// Output is top-down straight RGBA, like the source canvas Image. All paints
/// cover an opaque skin/clothes region or align to integer bounds, so the
/// premultiplied raster storage also has the correct straight bytes here.
/// The caller retains the source cache key of (major, minor), ignoring hair/pony.
pub fn elf_rgba(major: u32, minor: u32, hair: u32, ponytail: bool) -> Vec<u8> {
    let mut image = Pixmap::new(512, 512).unwrap();
    let mut paint = Paint::default();
    paint.anti_alias = true;
    let mut rect = |color: u32, x: f32, y: f32, w: f32, h: f32| {
        paint.set_color_rgba8((color >> 16) as u8, (color >> 8) as u8, color as u8, 255);
        image.fill_rect(
            Rect::from_xywh(x, y, w, h).unwrap(),
            &paint,
            Transform::identity(),
            None,
        );
    };
    // A fresh image plus these covering rectangles has the same result as the
    // source canvas clearRect(0,0,hairStart,153) on each uncached color pair.
    let start = if ponytail { 0. } else { 128. };
    rect(hair, start, 0., 256. - start, 153.);
    rect(0xffffff, 256., 0., 256., 153.);
    rect(minor, 0., 153., 512., 174.);
    rect(major, 0., 327., 512., 140.);
    rect(0x333333, 0., 467., 512., 45.);
    // Canvas arc uses continuous coverage at pixel centers; the former native
    // integer-distance tests were shifted by half a pixel and had hard edges.
    // The source appends the same two full circles to its retained path without
    // beginPath. Repeated contours have the same nonzero-winding filled region.
    let mut eyes = PathBuilder::new();
    eyes.push_circle(44., 207., 21.);
    eyes.push_circle(150., 207., 21.);
    paint.set_color_rgba8(51, 51, 51, 255);
    image.fill_path(
        &eyes.finish().unwrap(),
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    paint.set_color_rgba8(249, 212, 72, 255);
    image.fill_rect(
        Rect::from_xywh(252., 460., 60., 52.).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );
    image.data().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_texture_regions_match_and_eye_coverage_improves() {
        use macroquad::prelude::Image;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-elf-texture-errors.json")).unwrap();
        let rows = fixture["frames"].as_array().unwrap();
        assert_eq!(rows.len(), 28);
        for row in rows {
            let parse = |key: &str| {
                u32::from_str_radix(row[key].as_str().unwrap().trim_start_matches('#'), 16).unwrap()
            };
            let rgba = elf_rgba(
                parse("color"),
                parse("skin"),
                parse("hair"),
                row["ponytail"].as_bool().unwrap(),
            );
            let original = Image::from_file_with_format(
                &std::fs::read(root.join(format!(
                    "assets/snowball/reference-textures/{}.png",
                    row["index"]
                )))
                .unwrap(),
                None,
            )
            .unwrap();
            let mut error = 0usize;
            for (i, (a, b)) in rgba
                .chunks_exact(4)
                .zip(original.bytes.chunks_exact(4))
                .enumerate()
            {
                let (x, y) = (i % 512, i / 512);
                if !(23..171).contains(&x) || !(186..228).contains(&y) {
                    assert_eq!(a, b, "non-eye pixel {x},{y} in {}", row["index"]);
                }
                for c in 0..4 {
                    let e = a[c].abs_diff(b[c]);
                    assert!(e <= 40, "eye coverage divergence at {x},{y}");
                    error += e as usize;
                }
            }
            // This guards improvement against actual browser data, not equality:
            // the remaining native/browser eye-AA difference is documented.
            let old_error = row["before"]["rgba_mae_per_255"].as_f64().unwrap() * rgba.len() as f64;
            assert!(
                (error as f64) < old_error / 4.,
                "source coverage regressed for {}",
                row["index"]
            );
        }
    }
}
