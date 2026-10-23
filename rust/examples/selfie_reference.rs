//! Draw the production Selfie renderer with a controlled pointer/expression.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/selfie_view.rs"]
mod selfie_view;
#[path = "../src/ui_render.rs"]
mod ui_render;
#[path = "../src/village.rs"]
mod village;
mod chrome {
    pub fn over_controls(_: macroquad::prelude::Vec2, _: usize) -> bool {
        false
    }
}
use macroquad::prelude::*;
fn main() {
    let a: Vec<_> = std::env::args().skip(1).collect();
    let action = a.first().cloned().unwrap_or("idle".into());
    let out = a.get(1).cloned().unwrap_or("/tmp/selfie-native.png".into());
    let width = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1440);
    macroquad::Window::from_config(
        Conf {
            window_title: "Selfie comparison".into(),
            window_width: width,
            window_height: 800,
            sample_count: 4,
            ..Default::default()
        },
        async move {
            let mut a = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            a.load_scene("santaselfie").unwrap();
            let mut g = santa_tracker::selfie::Selfie::new("test1234");
            let mouse = if action == "dryer-center" {
                vec2(width as f32 / 2., 400.)
            } else if action == "dryer-left" {
                vec2(0., 400.)
            } else {
                Vec2::ZERO
            };
            if action.starts_with("dryer") {
                g.select(santa_tracker::selfie::Tool::Dryer);
                g.mouse.down = true;
            }
            if action == "blink" {
                g.blink_remaining = 0.2;
            }
            let mut view = selfie_view::SelfieView::new().unwrap();
            for frame in 0..2 {
                view.draw_with_mouse(&a, &g, mouse);
                if frame == 1 {
                    capture::save_png(&get_screen_data(), std::path::Path::new(&out)).unwrap();
                }
                next_frame().await;
            }
        },
    );
}
