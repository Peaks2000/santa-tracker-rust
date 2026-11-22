//! Compare native CPU texture generation with saved actual browser Canvas PNGs.
//! Ordinary main: no window, GPU or input.
use macroquad::prelude::{Color, Image, BLANK, WHITE};
use santa_tracker::snowball_texture::{elf_rgba, HAIR};
use std::path::Path;
fn old_rgba(major: u32, minor: u32, hair: u32, pony: bool) -> Vec<u8> {
    // Immutable pre-change production loop, retained only for before/after evidence.
    let mut im = Image::gen_image_color(512, 512, BLANK);
    for y in 0..512 {
        for x in 0..512 {
            let c = if y < 153 {
                if x >= 256 {
                    WHITE
                } else if pony || x >= 128 {
                    Color::from_hex(hair)
                } else {
                    BLANK
                }
            } else if y < 327 {
                if (x as i32 - 44).pow(2) + (y as i32 - 207).pow(2) < 441
                    || (x as i32 - 150).pow(2) + (y as i32 - 207).pow(2) < 441
                {
                    Color::from_hex(0x333333)
                } else {
                    Color::from_hex(minor)
                }
            } else if y < 467 {
                Color::from_hex(major)
            } else {
                Color::from_hex(0x333333)
            };
            im.set_pixel(
                x,
                y,
                if (252..312).contains(&x) && y >= 460 {
                    Color::from_hex(0xf9d448)
                } else {
                    c
                },
            );
        }
    }
    im.bytes
}
fn metric(actual: &[u8], original: &[u8]) -> serde_json::Value {
    let mut total = 0usize;
    let mut max = 0;
    let mut changed = 0;
    for (a, b) in actual.chunks_exact(4).zip(original.chunks_exact(4)) {
        changed += usize::from(a != b);
        for i in 0..4 {
            let e = a[i].abs_diff(b[i]);
            total += e as usize;
            max = max.max(e);
        }
    }
    serde_json::json!({"different_pixels":changed,"rgba_mae_per_255":total as f64/actual.len() as f64,"max_channel_error":max})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture: serde_json::Value = serde_json::from_slice(&std::fs::read(
        root.join("data/snowball-render-reference.json"),
    )?)?;
    let output = std::env::args().nth(1).map(std::path::PathBuf::from);
    if let Some(p) = &output {
        std::fs::create_dir_all(p)?;
    }
    let mut rows = vec![];
    for (i, row) in fixture["players"].as_array().unwrap().iter().enumerate() {
        let path = root
            .join("assets/snowball/reference-textures")
            .join(row["texture"].as_str().unwrap());
        let original = Image::from_file_with_format(&std::fs::read(path)?, None)?;
        assert_eq!((original.width, original.height), (512, 512));
        let from_hex = |s: &str| u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap();
        let major = from_hex(row["color"].as_str().unwrap());
        let skin = from_hex(row["skin"].as_str().unwrap());
        // The legacy fixture records color/skin but not hair/pony. Infer only
        // these two constant regions of the preserved browser image; no eye
        // pixel/coverage is injected into the native generator.
        let pixel = |x: usize, y: usize| &original.bytes[(y * 512 + x) * 4..(y * 512 + x + 1) * 4];
        let h = pixel(150, 10);
        let hair = (h[0] as u32) << 16 | (h[1] as u32) << 8 | h[2] as u32;
        assert!(HAIR.contains(&hair));
        let pony = pixel(10, 10)[3] > 0;
        let old = old_rgba(major, skin, hair, pony);
        let new = elf_rgba(major, skin, hair, pony);
        if let Some(p) = &output {
            for (name, bytes) in [("before", &old), ("after", &new)] {
                let mut e = png::Encoder::new(
                    std::fs::File::create(p.join(format!("{i}-{name}.png")))?,
                    512,
                    512,
                );
                e.set_color(png::ColorType::Rgba);
                e.set_depth(png::BitDepth::Eight);
                e.write_header()?.write_image_data(bytes)?;
            }
        }
        rows.push(serde_json::json!({"index":i,"color":row["color"],"skin":row["skin"],"hair":format!("#{hair:06x}"),"ponytail":pony,"before":metric(&old,&original.bytes),"after":metric(&new,&original.bytes)}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"method":"Actual current native CPU generator and immutable preceding production loop compared with 28 saved original browser Canvas PNGs. Major/skin are recorded source values; hair/pony inferred from constant non-eye regions, not rerolled or eye-injected. No window/GPU/input; not cache lifecycle, mipmap, renderer or whole-scene parity. Tiny-skia circle coverage can differ from browser Skia.","frames":rows})
        )?
    );
    Ok(())
}
