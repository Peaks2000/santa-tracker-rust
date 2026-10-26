//! Isolated result-panel comparison using the live renderer/model.
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
    let output = args.first().expect("output PNG").clone();
    let width = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1440);
    let height = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(800);
    let name = args.get(3).map(String::as_str).unwrap_or("success");
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../data/codeboogie-result-layout-reference.json"
    ))
    .unwrap();
    let row = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["spec"]["name"] == name);
    let flow = name == "flow";
    let spec = (!flow)
        .then(|| serde_json::from_value(row.expect("result case name")["spec"].clone()).unwrap());
    let time = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(2.);
    macroquad::Window::from_config(
        Conf {
            window_title: "Code Boogie result comparison".into(),
            window_width: width,
            window_height: height,
            ..Default::default()
        },
        async move {
            let mut view = coding_view::CodingView::new(true).unwrap();
            if flow {
                view.prepare_result_flow_capture();
            } else {
                view.prepare_result_capture(spec.unwrap(), time);
            }
            for _ in 0..8 {
                view.draw_result_capture();
                next_frame().await;
            }
            view.draw_result_capture();
            capture::save_png(&get_screen_data(), std::path::Path::new(&output)).unwrap();
            println!("{}x{}", screen_width(), screen_height());
        },
    );
}
