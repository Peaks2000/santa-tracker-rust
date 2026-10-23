//! Verify a final opacity seal leaves RGB unchanged while fixing surface alpha.
#![allow(dead_code)]
#[path = "../src/ui_render.rs"]
mod ui_render;
use macroquad::prelude::*;
fn main() {
    macroquad::Window::new("Opaque surface check", async {
        let seal = ui_render::OpaqueSurface::new().unwrap();
        let shapes = (0..64)
            .map(|_| ui_render::Shapes::new().unwrap())
            .collect::<Vec<_>>();
        clear_background(Color::from_hex(0x326e9d));
        draw_circle(160., 160., 100., Color::new(1., 0.3, 0.1, 0.4));
        shapes[0].rounded(Rect::new(10., 10., 40., 40.), 4., RED, 0.);
        shapes[63].rounded(Rect::new(60., 10., 40., 40.), 4., BLUE, 0.);
        let before = get_screen_data();
        let pixel = |x: usize, y: usize| {
            let i = ((before.height as usize - 1 - y) * before.width as usize + x) * 4;
            before.bytes[i..i + 3].to_vec()
        };
        assert!(pixel(30, 30)
            .iter()
            .zip([230u8, 41, 56])
            .all(|(a, b)| a.abs_diff(b) <= 1));
        assert!(pixel(80, 30)
            .iter()
            .zip([0u8, 120, 242])
            .all(|(a, b)| a.abs_diff(b) <= 1));
        seal.draw();
        let after = get_screen_data();
        let low = before.bytes.chunks_exact(4).filter(|p| p[3] < 255).count();
        for (a, b) in before
            .bytes
            .chunks_exact(4)
            .zip(after.bytes.chunks_exact(4))
        {
            assert_eq!(&a[..3], &b[..3]);
            assert_eq!(b[3], 255);
        }
        println!("RGB unchanged; {low} translucent surface pixels sealed opaque.");
    });
}
