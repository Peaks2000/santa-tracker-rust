//! Draw the actual native Gift Slingshot renderer for browser comparisons.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/boatload_view.rs"]
mod boatload_view;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/ui_render.rs"]
mod ui_render;
mod chrome {
    pub fn over_controls(_: macroquad::prelude::Vec2, _: usize) -> bool {
        false
    }
}
use macroquad::prelude::*;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let output = args
        .first()
        .cloned()
        .unwrap_or("/tmp/boatload-native.png".into());
    macroquad::Window::from_config(
        Conf {
            window_title: "Gift Slingshot comparison".into(),
            window_width: 1440,
            window_height: 800,
            sample_count: 4,
            ..Default::default()
        },
        async move {
            let mut a = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            a.load_scene("boatload").unwrap();
            let mut g = santa_tracker::boatload::Boatload::new("test1234", 1440., 800.);
            g.advance(0.5);
            let view = boatload_view::BoatloadView::new().unwrap();
            for frame in 0..2 {
                view.draw(&a, &g);
                if frame == 1 {
                    capture::save_png(&get_screen_data(), std::path::Path::new(&output)).unwrap();
                }
                next_frame().await;
            }
        },
    );
}
