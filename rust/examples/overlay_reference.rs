//! Render the production pause/game-over view on a neutral background.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/chrome.rs"]
mod chrome;
#[path = "../src/overlay.rs"]
mod overlay;
#[path = "../src/scene_filter.rs"]
mod scene_filter;
#[path = "../src/ui_render.rs"]
mod ui_render;
#[path = "../src/village.rs"]
mod village;
use macroquad::prelude::*;
fn main() {
    let a: Vec<_> = std::env::args().skip(1).collect();
    let paused = a.first().is_none_or(|s| s == "pause");
    let out = a
        .get(1)
        .cloned()
        .unwrap_or("/tmp/overlay-native.png".into());
    let w = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1440);
    let h = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(800);
    macroquad::Window::from_config(
        Conf {
            window_title: "Overlay comparison".into(),
            window_width: w,
            window_height: h,
            sample_count: 4,
            ..Default::default()
        },
        async move {
            let mut assets = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            let engine = thorvg::Thorvg::init(0).unwrap();
            let mut village = village::Village::new(&engine).unwrap();
            let chrome = chrome::Chrome::new(None).unwrap();
            let mut overlay = overlay::Overlay::new().unwrap();
            for frame in 0..2 {
                clear_background(Color::from_hex(0x666666));
                overlay.draw_at(
                    &mut assets,
                    &chrome,
                    &mut village,
                    "runner",
                    paused,
                    !paused,
                    frame as f64,
                );
                if frame == 1 {
                    capture::save_png(&get_screen_data(), std::path::Path::new(&out)).unwrap();
                }
                next_frame().await;
            }
        },
    );
}
