//! Capture production native views at a controlled clock.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/elf_shadow.rs"]
mod elf_shadow;
#[path = "../src/elfmaker_view.rs"]
mod elfmaker_view;
#[path = "../src/jamband_view.rs"]
mod jamband_view;
#[path = "../src/ui_render.rs"]
mod ui_render;
mod chrome {
    pub fn over_controls(_: macroquad::prelude::Vec2, _: usize) -> bool {
        false
    }
}
use macroquad::prelude::*;
fn main() {
    let a: Vec<_> = std::env::args().skip(1).collect();
    let mode = a.first().cloned().unwrap_or("elfmaker".into());
    let out = a.get(1).cloned().unwrap_or("/tmp/new-scene.png".into());
    let w = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1440);
    let advance: f64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.);
    let band = a.get(4).cloned().unwrap_or_default();
    macroquad::Window::from_config(
        Conf {
            window_title: "New scene source comparison".into(),
            window_width: w,
            window_height: 800,
            sample_count: 4,
            ..Default::default()
        },
        async move {
            let mut assets = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            assets.load_scene(&mode).unwrap();
            let mut maker = santa_tracker::elfmaker::ElfMaker::new("test1234");
            let mut mv = elfmaker_view::ElfMakerView::new().unwrap();
            let mut jam = santa_tracker::jamband::Jamband::new("test1234");
            let mut jv = jamband_view::JambandView::new().unwrap();
            if !band.is_empty() {
                jam.restore(&band);
            }
            jam.advance(advance);
            maker.advance(advance);
            println!("Elf state: {}", maker.save());
            request_new_screen_size(w as f32, 800.);
            for i in 0..8 {
                if mode == "elfmaker" {
                    mv.draw(&assets, &mut maker, false)
                } else {
                    jv.draw(&assets, &mut jam, false)
                }
                if i == 7 {
                    assert_eq!(
                        (screen_width() as i32, screen_height() as i32),
                        (w, 800),
                        "Matched capture viewport"
                    );
                    capture::save_png(&get_screen_data(), std::path::Path::new(&out)).unwrap();
                }
                next_frame().await;
            }
        },
    )
}
