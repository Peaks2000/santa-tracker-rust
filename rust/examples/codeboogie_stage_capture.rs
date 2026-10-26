//! Scene-only fidelity diagnostic; no alternate gameplay implementation.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/coding_view.rs"]
mod coding_view;
#[path = "../src/ui_render.rs"]
mod ui_render;
use macroquad::prelude::*;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let output = args.first().expect("output PNG path").clone();
    let width = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1440);
    let height = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(800);
    let watching = args.get(3).is_none_or(|s| s != "normal");
    macroquad::Window::from_config(
        Conf {
            window_title: "Code Boogie stage comparison".into(),
            window_width: width,
            window_height: height,
            ..Default::default()
        },
        async move {
            let mut assets = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            assets.load_scene("codeboogie").unwrap();
            let mut view = coding_view::CodingView::new(true).unwrap();
            view.prepare_stage_capture(6, watching);
            for _ in 0..8 {
                view.draw_stage_capture(&mut assets);
                next_frame().await;
            }
            view.draw_stage_capture(&mut assets);
            capture::save_png(&get_screen_data(), std::path::Path::new(&output)).unwrap();
        },
    );
}
