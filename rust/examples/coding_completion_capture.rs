//! Production coding view + disabled scene filter + shared game-over/chrome.
//! Calls completion callbacks directly; no fake gameplay or input certification.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/chrome.rs"]
mod chrome;
#[path = "../src/coding_view.rs"]
mod coding_view;
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
    let args: Vec<_> = std::env::args().skip(1).collect();
    let scene = args.first().cloned().unwrap_or("codeboogie".into());
    assert!(scene == "codeboogie" || scene == "codelab");
    let output = args
        .get(1)
        .cloned()
        .unwrap_or("/tmp/coding-completion.png".into());
    let width = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1440);
    let height = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(800);
    macroquad::Window::from_config(
        Conf {
            window_title: "Coding completion comparison".into(),
            window_width: width,
            window_height: height,
            ..Default::default()
        },
        async move {
            let mut assets = assets::Assets::new(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
            );
            assets.load_scene(&scene).unwrap();
            let engine = thorvg::Thorvg::init(0).unwrap();
            let mut village = village::Village::new(&engine).unwrap();
            let mut chrome = chrome::Chrome::new(None).unwrap();
            let mut overlay = overlay::Overlay::new().unwrap();
            overlay.set_recommendations(["santaselfie".into(), "jetpack".into()]);
            let mut filter = scene_filter::SceneFilter::new().unwrap();
            let mut view = coding_view::CodingView::new(scene == "codeboogie").unwrap();
            view.prepare_completion_capture();
            let opaque = ui_render::OpaqueSurface::new().unwrap();
            for frame in 0..121 {
                // Controlled live scene clock under a fully disabled host.
                // The production loop advances from wall frames instead.
                view.advance(1. / 60.);
                filter.begin();
                view.draw(&mut assets, false, true);
                filter.finish_live(view.host.gameover);
                overlay.draw(
                    &mut assets,
                    &chrome,
                    &mut village,
                    &scene,
                    false,
                    view.host.gameover,
                );
                chrome.draw(
                    &scene,
                    Some(chrome::GameHud {
                        playing: !view.host.gameover,
                        paused: false,
                        score: 0,
                        countdown: 0.,
                        level: Some((view.level_index + 1) as u32),
                        badge: true,
                        can_pause: false,
                    }),
                    true,
                );
                opaque.draw();
                if frame == 90 {
                    capture::save_png(
                        &get_screen_data(),
                        std::path::Path::new(&format!("{output}.before.png")),
                    )
                    .unwrap();
                }
                if frame == 120 {
                    capture::save_png(&get_screen_data(), std::path::Path::new(&output)).unwrap();
                }
                next_frame().await;
            }
            println!(
                "{} {}x{} {:?}",
                scene,
                screen_width(),
                screen_height(),
                view.host
            );
        },
    );
}
