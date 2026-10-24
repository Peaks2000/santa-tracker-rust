//! Draw the actual native Gift Slingshot renderer for browser comparisons.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/boatload_view.rs"]
mod boatload_view;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/scene_filter.rs"]
mod scene_filter;
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
    let w = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1440);
    let h = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(800);
    macroquad::Window::from_config(
        Conf {
            window_title: "Gift Slingshot comparison".into(),
            window_width: w,
            window_height: h,
            sample_count: 4,
            ..Default::default()
        },
        async move {
            let mut a = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            a.load_scene("boatload").unwrap();
            let mut g = santa_tracker::boatload::Boatload::new("test1234", w as f64, h as f64);
            g.advance(0.5);
            let view = boatload_view::BoatloadView::new().unwrap();
            let mut compositor = scene_filter::SceneFilter::new().unwrap();
            for frame in 0..8 {
                compositor.begin();
                view.draw(&a, &g);
                compositor.finish(false);
                if frame == 7 {
                    capture::save_png(&get_screen_data(), std::path::Path::new(&output)).unwrap();
                }
                next_frame().await;
            }
        },
    );
}
