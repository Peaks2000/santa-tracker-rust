//! Owned GPU check of physical target allocation and logical presentation.
#![allow(dead_code)]
#[path = "../src/scene_filter.rs"]
mod scene_filter;
use macroquad::prelude::*;
fn main() {
    macroquad::Window::from_config(
        Conf {
            window_title: "Scene density diagnostic".into(),
            window_width: 320,
            window_height: 240,
            high_dpi: true,
            sample_count: 4,
            ..Default::default()
        },
        run(),
    );
}
fn pattern() {
    clear_background(Color::from_rgba(17, 34, 51, 255));
    draw_rectangle(8., 12., 16., 20., RED);
    draw_rectangle(screen_width() - 32., screen_height() - 36., 16., 20., GREEN);
    santa_tracker::surface::scissor(Some((
        screen_width() as i32 / 2,
        screen_height() as i32 / 2,
        12,
        16,
    )));
    draw_rectangle(0., 0., screen_width(), screen_height(), BLUE);
    santa_tracker::surface::scissor(None);
}
fn pixel(image: &Image, x: u32, y: u32) -> [u8; 4] {
    let i = ((image.height as u32 - 1 - y) * image.width as u32 + x) as usize * 4;
    image.bytes[i..i + 4].try_into().unwrap()
}
fn close(a: [u8; 4], b: [u8; 4]) {
    for i in 0..4 {
        assert!((a[i] as i16 - b[i] as i16).abs() <= 1, "{a:?} vs {b:?}");
    }
}
async fn run() {
    let mut filter = scene_filter::SceneFilter::new().unwrap();
    let mut rows = vec![];
    for ratio in [1., 2., 1.25, 1., 2.] {
        filter.begin_with_pixel_ratio(ratio);
        pattern();
        filter.finish_live(false);
        let texture = filter.scene_texture();
        let image = texture.get_texture_data();
        let (w, h) = (screen_width(), screen_height());
        assert_eq!(
            (image.width, image.height),
            ((w * ratio) as u16, (h * ratio) as u16)
        );
        let red = pixel(&image, (16. * ratio) as u32, (20. * ratio) as u32);
        let green = pixel(
            &image,
            ((w - 24.) * ratio) as u32,
            ((h - 26.) * ratio) as u32,
        );
        // Macroquad's named RED/GREEN are their original RGBA palette values.
        close(
            red,
            [
                (RED.r * 255.) as u8,
                (RED.g * 255.) as u8,
                (RED.b * 255.) as u8,
                255,
            ],
        );
        close(
            green,
            [
                (GREEN.r * 255.) as u8,
                (GREEN.g * 255.) as u8,
                (GREEN.b * 255.) as u8,
                255,
            ],
        );
        let presented = get_screen_data();
        let actual_ratio = screen_dpi_scale();
        let presentation = pixel(
            &presented,
            ((w - 24.) * actual_ratio) as u32,
            ((h - 26.) * actual_ratio) as u32,
        );
        close(presentation, green);
        let blue = pixel(
            &image,
            ((w / 2. + 6.) * ratio) as u32,
            ((h / 2. + 8.) * ratio) as u32,
        );
        close(
            blue,
            [
                (BLUE.r * 255.) as u8,
                (BLUE.g * 255.) as u8,
                (BLUE.b * 255.) as u8,
                255,
            ],
        );
        let outside = pixel(
            &image,
            ((w / 2. + 16.) * ratio) as u32,
            ((h / 2. + 8.) * ratio) as u32,
        );
        close(outside, [17, 34, 51, 255]);
        rows.push(serde_json::json!({"ratio":ratio,"logical":[w,h],"target":[image.width,image.height],"red":red,"green":green,"presented_green":presentation,"clip_blue":blue,"clip_outside":outside}));
        next_frame().await;
    }
    let mut disabled = scene_filter::SceneFilter::new().unwrap();
    for _ in 0..80 {
        disabled.begin_with_pixel_ratio(2.);
        pattern();
        disabled.finish_live(true);
        next_frame().await;
    }
    let mut disabled_rows = vec![];
    for ratio in [2., 1.25, 1., 2.] {
        // Read after this frame's draw; next_frame starts a cleared framebuffer.
        disabled.begin_with_pixel_ratio(ratio);
        pattern();
        // The non-live path must invalidate its cached image when density changes.
        disabled.finish(true);
        let image = get_screen_data();
        let scale = screen_dpi_scale();
        let x = (screen_width() / 2. + (screen_width() - 24. - screen_width() / 2.) * 1.05) * scale;
        let y =
            (screen_height() / 2. + (screen_height() - 26. - screen_height() / 2.) * 1.05) * scale;
        let green = pixel(&image, x as u32, y as u32);
        assert!(
            green[1] > green[0] + 20,
            "disabled high-density image must fill logical bounds: {green:?}"
        );
        disabled_rows.push(serde_json::json!({"ratio":ratio,"green":green}));
        next_frame().await;
    }
    let path = std::env::args()
        .nth(1)
        .unwrap_or("/tmp/scene-filter-density.json".into());
    std::fs::write(&path,serde_json::to_string_pretty(&serde_json::json!({"method":"Actual production SceneFilter and framebuffer-aware logical scissor helper with GPU readback. Explicit ratios exercise physical target sizes, logical geometry/clipping/blit, resize/reallocation, opaque enabled presentation and disabled cached fades. Actual OS DPI is recorded; override tests do not certify native monitor/DPI event delivery or every scene/font/UI path.","os_ratio":screen_dpi_scale(),"enabled":rows,"disabled":disabled_rows})).unwrap()).unwrap();
    println!("Scene density GPU checks passed: {path}");
}
