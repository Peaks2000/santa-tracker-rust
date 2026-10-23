//! Render the actual native Runner with the source comparison actions.
//! cargo run --manifest-path rust/Cargo.toml --example runner_reference -- idle /tmp/idle.png
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/render.rs"]
mod render;
mod chrome {
    pub fn over_controls(_: macroquad::prelude::Vec2, _: usize) -> bool {
        false
    }
}
use macroquad::prelude::*;
fn main() {
    let mut args = std::env::args().skip(1);
    let action = args.next().unwrap_or("idle".into());
    let output = args.next().unwrap_or("runner-reference.png".into());
    let width = args.next().and_then(|s| s.parse().ok()).unwrap_or(1440);
    let height = args.next().and_then(|s| s.parse().ok()).unwrap_or(800);
    macroquad::Window::from_config(
        Conf {
            window_title: "Runner reference check".into(),
            window_width: width,
            window_height: height,
            sample_count: 4,
            ..Default::default()
        },
        async move {
            let mut a = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            a.load_scene("runner").unwrap();
            let mut g = santa_tracker::runner::Runner::new("test1234");
            g.immediate_levels = true;
            if action != "idle" {
                g.advance(3.);
            }
            if action == "jump" || action == "slide" {
                g.jump();
                g.advance(0.2);
            }
            if action == "slide" {
                g.player.y = 0.;
                g.player.y_speed = 0.;
                g.player.state = santa_tracker::runner::State::Running;
                g.down();
            }
            let mut view = render::View::new();
            for frame in 0..2 {
                view.runner(&a, &g);
                if frame == 1 {
                    capture::save_png(&get_screen_data(), std::path::Path::new(&output)).unwrap();
                }
                next_frame().await;
            }
        },
    );
}
