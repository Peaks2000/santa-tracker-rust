//! CPU-only audit of the production HUD decoder; never opens a window.
#![allow(dead_code)]
mod assets {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assets.rs"));
    pub fn cpu_decode(root: &std::path::Path, key: &str) -> Result<Image, String> {
        decode_image(root, key)
    }
}
use macroquad::prelude::Image;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../static");
    let mut rows = vec![];
    for name in [
        "item-frame.png",
        "powerup-1.png",
        "sleigh.png",
        "parachute-button.png",
    ] {
        let key = format!("scenes/snowball/img/{name}");
        let raw = Image::from_file_with_format(&std::fs::read(root.join(&key))?, None)?;
        let upload = assets::cpu_decode(&root, &key)?;
        let mut before_max = 0f64;
        let mut after_max = 0f64;
        let mut before_sum = 0.;
        let mut after_sum = 0.;
        let mut components = 0usize;
        let mut fractional = 0usize;
        let w = raw.width as usize;
        let h = raw.height as usize;
        // Four half-texel bilinear neighbours, at three subpixel phases.
        // Target: exact float premultiplication BEFORE interpolation, followed
        // by source-over on two source HUD backgrounds and two opacities.
        for y in 0..h - 1 {
            for x in 0..w - 1 {
                let offsets = [
                    y * w + x,
                    y * w + x + 1,
                    (y + 1) * w + x,
                    (y + 1) * w + x + 1,
                ];
                for (fx, fy) in [(0.25, 0.75), (0.5, 0.5), (0.75, 0.25)] {
                    let weights = [
                        (1. - fx) * (1. - fy),
                        fx * (1. - fy),
                        (1. - fx) * fy,
                        fx * fy,
                    ];
                    let mut alpha = 0.;
                    let mut straight = [0.; 3];
                    let mut premult = [0.; 3];
                    let mut exact = [0.; 3];
                    for (offset, weight) in offsets.into_iter().zip(weights) {
                        let p = &raw.bytes[offset * 4..offset * 4 + 4];
                        let a = p[3] as f64 / 255.;
                        alpha += a * weight;
                        for c in 0..3 {
                            straight[c] += p[c] as f64 / 255. * weight;
                            premult[c] += upload.bytes[offset * 4 + c] as f64 / 255. * weight;
                            exact[c] += p[c] as f64 / 255. * a * weight;
                        }
                    }
                    if alpha <= 0. || alpha >= 1. {
                        continue;
                    }
                    fractional += 1;
                    for background in [[0x71, 0xa7, 0xdb], [0x56, 0xad, 0x57]] {
                        for opacity in [1., 0.35] {
                            for c in 0..3 {
                                let dest = background[c] as f64 / 255. * (1. - alpha * opacity);
                                let target = exact[c] * opacity + dest;
                                let before =
                                    ((straight[c] * alpha * opacity + dest) - target).abs() * 255.;
                                let after = ((premult[c] * opacity + dest) - target).abs() * 255.;
                                before_max = before_max.max(before);
                                after_max = after_max.max(after);
                                before_sum += before;
                                after_sum += after;
                                components += 1;
                            }
                        }
                    }
                }
            }
        }
        assert!(after_max <= 0.5000001, "premultiplied byte-rounding bound");
        assert!(after_sum < before_sum);
        rows.push(serde_json::json!({"file":key,"size":[w,h],"fractional_samples":fractional,
            "rgb_components":components,"before_max_error_per_255":before_max,
            "after_max_error_per_255":after_max,"before_mean_error_per_255":before_sum/components as f64,
            "after_mean_error_per_255":after_sum/components as f64}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "method":"Actual native CPU decoder bytes compared with analytic exact-float premultiplication-before-bilinear-filtering/source-over. Three subpixel phases, two source HUD backgrounds and two opacities. Not a browser/GPU screenshot or minification/mipmap oracle; does not prove whole-scene texture clarity.",
            "images":rows
        }))?
    );
    Ok(())
}
