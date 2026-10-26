//! Actual native sky renderer at neighboring subpixel transition positions.
#![allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/capture.rs"]
mod capture;
#[path = "../src/chrome.rs"]
mod chrome;
#[path = "../src/jetpack_view.rs"]
mod jetpack_view;
#[path = "../src/ui_render.rs"]
mod ui_render;
use macroquad::prelude::*;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let prefix = args.first().expect("output prefix").clone();
    macroquad::Window::from_config(
        Conf {
            window_title: "Jetpack gradient comparison".into(),
            window_width: 1440,
            window_height: 800,
            ..Default::default()
        },
        async move {
            let reference: serde_json::Value =
                serde_json::from_str(include_str!("../data/jetpack-background-reference.json"))
                    .unwrap();
            let opaque = ui_render::OpaqueSurface::new().unwrap();
            let mut samples = Vec::new();
            for level in [0, 4, 8] {
                for fraction in [0.35, 0.45, 0.55, 0.65] {
                    let progress = 0.5 + fraction / (2. * screen_height() as f64);
                    jetpack_view::JetpackView::draw_sky(level, Some(progress));
                    opaque.draw();
                    let image = get_screen_data();
                    capture::save_png(
                        &image,
                        std::path::Path::new(&format!("{prefix}-{level}-{fraction}.png")),
                    )
                    .unwrap();
                    let pixels = image.bytes.chunks_exact(4).collect::<Vec<_>>();
                    assert_eq!(
                        (image.width, image.height),
                        (1440, 800),
                        "comparison requires the reference viewport"
                    );
                    let case = reference["cases"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|c| c["level"] == level && c["fraction"].as_f64() == Some(fraction))
                        .unwrap();
                    let expected = case["rgb_column_hex"].as_str().unwrap();
                    for y in 0..800 {
                        // Native GL readback is bottom-up; source PNG rows
                        // and the exported reference column are top-down.
                        let pixel = pixels[(799 - y) * 1440 + 100];
                        for channel in 0..3 {
                            let at = (y * 3 + channel) * 2;
                            let source = u8::from_str_radix(&expected[at..at + 2], 16).unwrap();
                            assert!(pixel[channel].abs_diff(source)<=2,"level {level} fraction {fraction} y={y} channel={channel}: {} vs source {source}",pixel[channel]);
                        }
                    }
                    let points = [100, 400, 700].map(|y| {
                        let p =
                            pixels[(image.height as usize - 1 - y) * image.width as usize + 100];
                        serde_json::json!({"x":100,"y":y,"rgb":[p[0],p[1],p[2]]})
                    });
                    samples.push(serde_json::json!({"level":level,"progress":progress,"fraction":fraction,"points":points}));
                    next_frame().await;
                }
            }
            std::fs::write(format!("{prefix}.json"),serde_json::to_string_pretty(&serde_json::json!({"width":screen_width(),"height":screen_height(),"cases":samples})).unwrap()).unwrap();
        },
    );
}
