//! CPU-only audit of the production Glider decoder; never opens a window.
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
    let mut total_before=0.;
    for name in [
        "glider-sprite.svg",
        "cloud.svg",
        "hit-house-01.svg",
        "hit-house-14.svg",
        "hit-tree-1.svg",
        "treat-candy.svg",
        "present-01.svg",
        "present-05.svg",
        "orb-time-front.svg",
        "orb-time-back.svg",
        "points-100.svg",
        "hit-beam.svg",
    ] {
        let key = format!("scenes/glider/img/{name}");
        let data = std::fs::read(root.join(&key))?;
        let tree = resvg::usvg::Tree::from_data(&data, &Default::default())?;
        let scale = if name == "glider-sprite.svg" { 3. } else { 1. };
        let size = tree.size().to_int_size().scale_by(scale).unwrap();
        let mut reference = resvg::tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(scale, scale),
            &mut reference.as_mut(),
        );
        let raw = Image {
            bytes: reference.data().to_vec(),
            width: size.width() as u16,
            height: size.height() as u16,
        };
        let upload = assets::cpu_decode(&root, &key)?;
        assert_eq!((upload.width, upload.height), (raw.width, raw.height));
        assert_eq!(
            upload.bytes, raw.bytes,
            "keep original associated vector coverage"
        );
        // Reconstruct the actual previous SVG unassociation and two-pass bleed.
        let mut previous = raw.clone();
        for p in previous.bytes.chunks_exact_mut(4) {
            if p[3] > 0 {
                for c in 0..3 {
                    p[c] = (u32::from(p[c]) * 255 / u32::from(p[3])).min(255) as u8;
                }
            }
        }
        santa_tracker::image_alpha::bleed_edges(
            &mut previous.bytes,
            raw.width as usize,
            raw.height as usize,
        );
        let mut before_max = 0f64;
        let mut after_max = 0f64;
        let mut before_sum = 0.;
        let mut after_sum = 0.;
        let mut components = 0usize;
        let mut fractional = 0usize;
        let w = raw.width as usize;
        let h = raw.height as usize;
        // Four half-texel bilinear neighbours, at three subpixel phases.
        // Target: original associated SVG bytes BEFORE interpolation, followed
        // by source-over on two source Glider sky backgrounds and two opacities.
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
                            straight[c] += previous.bytes[offset * 4 + c] as f64 / 255. * weight;
                            premult[c] += upload.bytes[offset * 4 + c] as f64 / 255. * weight;
                            exact[c] += p[c] as f64 / 255. * weight;
                        }
                    }
                    if alpha <= 0. || alpha >= 1. {
                        continue;
                    }
                    fractional += 1;
                    for background in [[0x23, 0x69, 0xba], [0x7d, 0x5e, 0xbc]] {
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
        assert_eq!(after_max, 0., "associated SVG bytes must stay exact");
        assert!(after_sum <= before_sum,"{name}: edge error must not increase");
        total_before+=before_sum;
        rows.push(serde_json::json!({"file":key,"size":[w,h],"fractional_samples":fractional,
            "rgb_components":components,"before_max_error_per_255":before_max,
            "after_max_error_per_255":after_max,"before_mean_error_per_255":before_sum/components as f64,
            "after_mean_error_per_255":after_sum/components as f64}));
    }
    assert!(total_before>0.,"must exercise changed fractional edge colors");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "method":"Actual native CPU decoder bytes compared with independently rendered original associated SVG bytes and analytic bilinear-filtering/source-over. Three subpixel phases, two source Glider sky backgrounds and two opacities. Baseline includes actual prior two-pass edge bleeding. Not a browser/GPU screenshot or minification/mipmap oracle; does not prove whole-scene texture clarity.",
            "images":rows
        }))?
    );
    Ok(())
}
