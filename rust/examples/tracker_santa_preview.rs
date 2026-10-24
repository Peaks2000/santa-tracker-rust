//! Isolated native tracker Santa comparison, not a working Christmas map.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/tracker_santa_view.rs"]
mod tracker_santa_view;
#[path = "../src/ui_render.rs"]
mod ui_render;
use macroquad::prelude::*;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let output = args.first().cloned();
    let w = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1440);
    let heading = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(90.);
    let stop = args.get(3).is_some_and(|s| s == "stop");
    let time = args.get(4).and_then(|s| s.parse().ok());
    macroquad::Window::from_config(
        Conf {
            window_title: "Tracker Santa animation preview".into(),
            window_width: w,
            window_height: 800,
            ..Default::default()
        },
        async move {
            let mut assets = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            let mut view = tracker_santa_view::SantaView::new(&mut assets).unwrap();
            let started = get_time();
            let mut frame = 0;
            loop {
                clear_background(Color::from_rgba(142, 224, 221, 255));
                let (sw, sh) = santa_tracker::tracker_santa::outer_size(screen_width() as f64);
                let ms = time.unwrap_or((get_time() - started) * 1000.);
                view.draw(
                    &assets,
                    vec2(300. + sw as f32 / 2., 200. + sh as f32 / 2.),
                    screen_width() as f64,
                    heading,
                    stop,
                    ms,
                    ms,
                );
                if frame == 7 {
                    if let Some(output) = &output {
                        capture::save_png(&get_screen_data(), std::path::Path::new(output))
                            .unwrap();
                        break;
                    }
                }
                frame += 1;
                next_frame().await;
            }
        },
    );
}
