//! CPU-only check of the production Elf Maker SVG raster path. No window/device.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/elf_shadow.rs"]
mod elf_shadow;
#[path = "../src/elfmaker_view.rs"]
mod elfmaker_view;
#[path = "../src/ui_render.rs"]
mod ui_render;
mod chrome {
    pub fn over_controls(_: macroquad::prelude::Vec2, _: usize) -> bool {
        false
    }
}
fn main() {
    let game = santa_tracker::elfmaker::ElfMaker::new("test1234");
    let svg = game.svg("-40 -100 400 560", None);
    let mut rows = vec![];
    let mut base_coverage = 0.;
    for density in [1., 1.25, 1.5, 2., 3.] {
        let (w, h) = santa_tracker::raster_density::size(450., 630., density);
        let image = elfmaker_view::raster_premult(&svg, w, h);
        assert_eq!((image.width as u32, image.height as u32), (w, h));
        assert!(image
            .bytes
            .chunks_exact(4)
            .all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]));
        let alpha = image
            .bytes
            .chunks_exact(4)
            .map(|p| p[3] as f64 / 255.)
            .sum::<f64>();
        let coverage = alpha / (w * h) as f64;
        if density == 1. {
            let original = elfmaker_view::raster_premult(&svg, 450, 630);
            assert_eq!(image.bytes, original.bytes);
            base_coverage = coverage;
        }
        assert!(
            (coverage - base_coverage).abs() < 0.001,
            "geometry/coverage changed at{density}x"
        );
        rows.push(serde_json::json!({"density":density,"logical":[450,630],"physical":[w,h],"pixels":w*h,"alpha_coverage":coverage,"logical_shadow_step":santa_tracker::raster_density::logical_texel(450.,630.)}));
    }
    let report = serde_json::json!({"scope":"Production CPU vector raster dimensions, normalized coverage and associated-alpha check. No GPU/filter/shadow output, browser/native screenshot or whole-mode parity measured.","seed":"test1234","one_x_character_bitmap_unchanged":true,"photo_export_remains":[640,1000],"rows":rows});
    let text = serde_json::to_string_pretty(&report).unwrap() + "\n";
    if let Some(path) = std::env::args().nth(1) {
        std::fs::write(path, &text).unwrap();
    }
    print!("{text}");
}
