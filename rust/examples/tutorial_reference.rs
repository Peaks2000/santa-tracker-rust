//! Render the real native tutorial overlay with a controlled animation clock.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/tutorial_view.rs"]
mod tutorial_view;
mod chrome {
    pub fn over_controls(_: macroquad::prelude::Vec2, _: usize) -> bool {
        false
    }
}
use macroquad::prelude::*;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let name = args.first().cloned().unwrap_or("keys-space".into());
    let elapsed: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.);
    let out = args
        .get(2)
        .cloned()
        .unwrap_or("/tmp/tutorial-native.png".into());
    macroquad::Window::from_config(
        Conf {
            window_title: "Tutorial comparison".into(),
            window_width: 1440,
            window_height: 800,
            sample_count: 4,
            ..Default::default()
        },
        async move {
            let mut a = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            a.load_scene(if name == "elfski.mp4" {
                "elfski"
            } else {
                "boatload"
            })
            .unwrap();
            let mut t = tutorial_view::TutorialView::new().unwrap();
            t.state.queue(&[&name], 0.);
            t.prepare(&a, 0.);
            for frame in 0..2 {
                clear_background(Color::from_hex(0x00abeb));
                t.draw(&a, 1.5 + elapsed, true);
                if frame == 1 {
                    capture::save_png(&get_screen_data(), std::path::Path::new(&out)).unwrap();
                }
                next_frame().await;
            }
        },
    );
}
