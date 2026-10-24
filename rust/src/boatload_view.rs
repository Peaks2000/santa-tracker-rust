//! Gift Slingshot's original HTML/SCSS stacking and world-unit geometry.
use crate::{assets::Assets, ui_render::Shapes};
use macroquad::prelude::*;
use santa_tracker::boatload::Boatload;
pub struct BoatloadView {
    font: Font,
    shapes: Shapes,
    premult: Material,
    touch: bool,
    space_down: bool,
}
impl BoatloadView {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/OpenSans-ExtraBold.ttf"
            ))
            .map_err(|e| e.to_string())?,
            shapes: Shapes::new()?,
            premult: crate::ui_render::premultiplied_material()?,
            touch: false,
            space_down: false,
        })
    }
    pub fn input(&mut self, g: &mut Boatload, block_mouse: bool) {
        if !g.playing || g.paused {
            return;
        }
        if [KeyCode::Up, KeyCode::Down, KeyCode::Space, KeyCode::Right]
            .iter()
            .any(|k| is_key_pressed(*k) || is_key_released(*k))
        {
            g.keys(is_key_down(KeyCode::Up), is_key_down(KeyCode::Down));
        }
        if (is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Right)) && !self.space_down {
            self.space_down = true;
            g.begin_charge();
        }
        if is_key_released(KeyCode::Space) || is_key_released(KeyCode::Right) {
            self.space_down = false;
            g.release(false);
        }
        if !block_mouse
            && is_mouse_button_pressed(MouseButton::Left)
            && !crate::chrome::over_controls(Vec2::from(mouse_position()), 3)
        {
            self.touch = true;
            g.set_y(mouse_position().1 as f64 / g.scale);
            g.begin_charge();
        }
        if self.touch {
            if !block_mouse && is_mouse_button_down(MouseButton::Left) {
                g.set_y(mouse_position().1 as f64 / g.scale);
            }
            if !block_mouse && is_mouse_button_released(MouseButton::Left) {
                self.touch = false;
                g.release(true);
            }
        }
    }
    pub fn draw(&self, a: &Assets, g: &Boatload) {
        let s = g.scale as f32;
        let base = screen_width() * 0.42 - 440. + 411.6 * (1. - s);
        let at = |x: f32, y: f32| vec2(base + x * s, y * s);
        let rect = |x: f32, y: f32, w: f32, h: f32| {
            let p = at(x, y);
            Rect::new(p.x, p.y, w * s, h * s)
        };
        let image = |name: &str, x: f32, y: f32, w: f32, h: f32| {
            let r = rect(x, y, w, h);
            gl_use_material(&self.premult);
            draw_texture_ex(
                a.scene("boatload", name),
                r.x,
                r.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(r.size()),
                    ..Default::default()
                },
            );
            gl_use_default_material();
        };
        clear_background(Color::from_hex(0xd3e4ed));
        let r = rect(294., 0., 686., g.height as f32);
        draw_rectangle(r.x - 8. * s, 0., r.w + 16. * s, screen_height(), WHITE);
        draw_rectangle(r.x, 0., r.w, screen_height(), Color::from_hex(0x00abeb));
        draw_rectangle(
            r.x,
            0.,
            r.w * 0.03,
            screen_height(),
            Color::from_hex(0x0098d2),
        );
        image("trees-top.svg", 100., 10., 53., 41.);
        image("trees-bottom.svg", 0., g.height as f32 - 50., 180., 55.);
        if screen_width() >= 980. {
            image(
                "trees-left.svg",
                -215.,
                g.height as f32 / 2. - 422.,
                202.,
                844.,
            );
            image(
                "trees-right.svg",
                1020.,
                g.height as f32 / 2. - 411.,
                210.,
                822.,
            );
        }
        let min = g.player.min_y as f32;
        let max = g.player.max_y as f32;
        let rails_h = (max - min) - (max - min) % 20. + 80.;
        let rail = a.scene("boatload", "rail.png");
        let mut y = min - 25.;
        gl_use_material(&self.premult);
        while y < min - 25. + rails_h {
            let h = (min - 25. + rails_h - y).min(rail.height());
            let r = rect(100., y, 63., h);
            draw_texture_ex(
                rail,
                r.x,
                r.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(r.size()),
                    source: Some(Rect::new(0., 0., 63., h)),
                    ..Default::default()
                },
            );
            y += h;
        }
        gl_use_default_material();
        for x in [110., 148.] {
            let r = rect(x, min - 35., 4., rails_h + 10.);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::from_hex(0x939597));
        }
        let py = g.player.y as f32 - 79.;
        image("player-crossbow.svg", 0., py, 228., 182.);
        let pull = -g.player.distance as f32 * 20. * g.player.animation as f32;
        image("player-elf.svg", 128. + pull, py + 61., 44., 44.);
        let scale = 0.07 * g.player.animation as f32 * g.player.distance as f32 + 0.48;
        let rotation =
            (12. * g.player.distance as f32 * g.player.animation as f32 + 14.).to_radians();
        for (top, angle, origin) in [(12., rotation, 0.), (45., -rotation, 110.)] {
            let pivot = at(193., py + top + origin);
            unsafe {
                get_internal_gl().quad_gl.push_model_matrix(
                    Mat4::from_translation(pivot.extend(0.))
                        * Mat4::from_scale(vec3(1., scale, 1.))
                        * Mat4::from_rotation_z(angle)
                        * Mat4::from_translation(-pivot.extend(0.)),
                );
            }
            let r = rect(193., py + top, 5., 110.);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::from_hex(0xed1c24));
            unsafe {
                get_internal_gl().quad_gl.pop_model_matrix();
            }
        }
        for b in g.icebergs.iter().filter(|b| !b.dead) {
            let name = format!("iceberg{}.svg", b.kind + 1);
            let size = a.scene_size("boatload", &name);
            image(&name, b.x as f32, (b.y - b.height) as f32, size.x, size.y);
        }
        // Source onInit overrides the namespaced animation with unresolved
        // 'bubble-left'; opacity still transitions for 600ms. Keep that source quirk.
        for b in &g.bubbles {
            let dim = [8., 5., 4., 8., 4., 5.][b.kind.saturating_sub(1) as usize];
            let p = at(845. + b.x as f32, b.y as f32);
            let alpha = 1. - santa_tracker::ui::Ease::Out.at((b.age / 0.6).clamp(0., 1.)) as f32;
            self.shapes.circle(
                p + vec2(dim * s / 2., dim * s / 2.),
                dim * s / 2.,
                Color::new(124. / 255., 202. / 255., 239. / 255., alpha),
                0.,
            );
        }
        for b in g.boats.iter().filter(|b| !b.dead) {
            let name = format!("boat{}.svg", b.kind + 1);
            let size = a.scene_size("boatload", &name);
            image(
                &name,
                850. - b.width as f32 / 2.,
                b.y as f32,
                size.x,
                size.y,
            );
        }
        for p in g.presents.iter().filter(|p| p.visible) {
            let scale = p.scale as f32;
            let origin = at((p.x + p.add_x) as f32 + 10., p.y as f32 + 10.);
            let alpha = 1. - santa_tracker::ui::Ease::Default.at(p.missed.unwrap_or(0.)) as f32;
            let boxcolor = match p.kind {
                1 => 0xfedf00,
                2 => 0xd94736,
                _ => 0xf68b37,
            };
            let ribbon = if p.kind == 2 { 0xf68b37 } else { 0xd94736 };
            let x = origin.x - 10. * s * scale;
            let y = origin.y - 10. * s * scale;
            let c = |v: u32| Color {
                a: alpha,
                ..Color::from_hex(v)
            };
            draw_rectangle(
                x + 5.098 * s * scale,
                y + 7.098 * s * scale,
                15.123 * s * scale,
                15.122 * s * scale,
                c(boxcolor),
            );
            draw_rectangle(
                x + 11.483 * s * scale,
                y + 7.098 * s * scale,
                2.028 * s * scale,
                15.122 * s * scale,
                c(ribbon),
            );
            draw_rectangle(
                x + 5.099 * s * scale,
                y + 13.516 * s * scale,
                15.121 * s * scale,
                2.027 * s * scale,
                c(ribbon),
            );
        }
        if let Some((x, y, score, start)) = g.score_effect {
            let t = ((g.time - start) / 0.4).clamp(0., 1.) as f32;
            if t < 1. {
                let p = at(x as f32 - 20., y as f32);
                draw_text_ex(
                    &score.to_string(),
                    p.x,
                    p.y,
                    TextParams {
                        font: Some(&self.font),
                        font_size: 48,
                        font_scale: s * (0.5 + 0.5 * t),
                        color: Color::new(1., 1., 1., 1. - t),
                        ..Default::default()
                    },
                );
            }
        }
        if let Some((x, y, start)) = g.splash {
            let t = ((g.time - start) / 0.5).clamp(0., 1.) as f32;
            if t < 1. {
                let p = at(x as f32 + 24., y as f32 + 24.);
                let eased = santa_tracker::ui::Ease::Default.at(t as f64) as f32;
                for (radius, weight) in [(9., 5.), (15., 3.), (20., 2.), (24., 1.)] {
                    draw_circle_lines(
                        p.x,
                        p.y,
                        radius * (1. + 0.3 * eased) * s,
                        weight * (1. - eased) * (1. + 0.3 * eased) * s,
                        Color::from_hex(0x3e9bd2),
                    );
                }
            }
        }
    }
}
