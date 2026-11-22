//! Convert window-logical UI clipping to active framebuffer physical pixels.
//! Camera-local image-baking coordinates require their own logical extent.
pub fn physical_clip(
    (x, y, w, h): (i32, i32, i32, i32),
    logical: (f32, f32),
    physical: (u32, u32),
) -> (i32, i32, i32, i32) {
    let sx = physical.0 as f64 / logical.0 as f64;
    let sy = physical.1 as f64 / logical.1 as f64;
    let left = (x as f64 * sx).floor() as i32;
    let top = (y as f64 * sy).floor() as i32;
    let width = if w == 0 {
        0
    } else {
        ((x as f64 + w as f64) * sx).ceil() as i32 - left
    };
    let height = if h == 0 {
        0
    } else {
        ((y as f64 + h as f64) * sy).ceil() as i32 - top
    };
    (left, top, width, height)
}
pub fn scissor(clip: Option<(i32, i32, i32, i32)>) {
    use macroquad::prelude::*;
    let logical = (screen_width(), screen_height());
    unsafe {
        let gl = get_internal_gl();
        let clip = clip.map(|clip| {
            let physical = if let Some(pass) = gl.quad_gl.get_active_render_pass() {
                let texture = gl.quad_context.render_pass_texture(pass);
                gl.quad_context.texture_size(texture)
            } else {
                let (w, h) = macroquad::miniquad::window::screen_size();
                (w as u32, h as u32)
            };
            physical_clip(clip, logical, physical)
        });
        gl.quad_gl.scissor(clip);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logical_clips_preserve_identity_and_empty_regions() {
        for rect in [
            (0, 0, 320, 240),
            (-10, 7, 31, 17),
            (3, 5, 0, 12),
            (3, 5, 12, 0),
        ] {
            assert_eq!(physical_clip(rect, (320., 240.), (320, 240)), rect);
        }
        assert_eq!(
            physical_clip((8, 12, 16, 20), (320., 240.), (640, 480)),
            (16, 24, 32, 40)
        );
        assert_eq!(
            physical_clip((3, 5, 8, 12), (320., 240.), (400, 300)),
            (3, 6, 11, 16)
        );
        assert_eq!(
            physical_clip((3, 5, 0, 0), (320., 240.), (400, 300)),
            (3, 6, 0, 0)
        );
    }
}
