//! Verify a final opacity seal leaves RGB unchanged while fixing surface alpha.
#![allow(dead_code)]
#[path = "../src/ui_render.rs"]
mod ui_render;
use macroquad::prelude::*;
fn main() {
    macroquad::Window::new("Opaque surface check", async {
        let seal = ui_render::OpaqueSurface::new().unwrap();
        clear_background(Color::from_hex(0x326e9d));
        draw_circle(160., 160., 100., Color::new(1., 0.3, 0.1, 0.4));
        let before = get_screen_data();
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
