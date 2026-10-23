//! CPU-only diagnostic of the exact native character raster paths.
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
    let g = santa_tracker::elfmaker::ElfMaker::new("test1234");
    let svg = g.svg("-40 -100 400 560", None);
    let shadow = elfmaker_view::shadow(&svg, 560. / 630.);
    for (mode, s) in [
        ("straight + filter", &shadow),
        ("premult + filter", &shadow),
        ("premult plain", &svg),
    ] {
        let start = std::time::Instant::now();
        for _ in 0..20 {
            std::hint::black_box(if mode.starts_with("straight") {
                elfmaker_view::raster(s, 450, 630)
            } else {
                elfmaker_view::raster_premult(s, 450, 630)
            });
        }
        println!(
            "{mode}: {:.3} ms",
            start.elapsed().as_secs_f64() * 1000. / 20.
        );
    }
}
