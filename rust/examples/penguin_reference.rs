//! Capture the production Glider renderer with controlled simulation time.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/penguin_view.rs"]
mod penguin_view;
use macroquad::prelude::*;
fn main() {
    let a: Vec<_> = std::env::args().skip(1).collect();
    let output = a
        .first()
        .cloned()
        .unwrap_or("/tmp/penguin-native.png".into());
    let time = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.);
    let w = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1440);
    let h = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(800);
    macroquad::Window::from_config(
        Conf {
            window_title: "Native Penguin Dash source comparison".into(),
            window_width: w,
            window_height: h,
            sample_count: 4,
            ..Default::default()
        },
        async move {
            let mut assets = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            assets.load_scene("penguindash").unwrap();
            let mut game = santa_tracker::penguindash::PenguinDash::new("test1234");
            game.advance(time);
            let mut view = penguin_view::PenguinView::new("test1234");
            request_new_screen_size(w as f32, h as f32);
            for i in 0..8 {
                view.draw(&assets, &game);
                if i == 7 {
                    assert_eq!((screen_width() as i32, screen_height() as i32), (w, h));
                    capture::save_png(&get_screen_data(), std::path::Path::new(&output)).unwrap();
                }
                next_frame().await;
            }
        },
    )
}
