use crate::assets::Assets;
use macroquad::prelude::*;
use santa_tracker::ui::{Ease, Transition};
use santa_tracker::{
    jetpack::{Jetpack, Kind},
    village_data::ease_in_out,
};

pub struct JetpackView {
    font: Font,
    title: Font,
    touch: Option<Vec2>,
    fire: Transition<1>,
    last_time: f64,
}
impl JetpackView {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/OpenSans-ExtraBold.ttf"
            ))
            .map_err(|e| e.to_string())?,
            title: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Lobster-Regular.ttf"))
                .map_err(|e| e.to_string())?,
            touch: None,
            fire: Transition::new([1.]),
            last_time: 0.,
        })
    }
    pub fn dimensions() -> (f64, f64) {
        let scale = (screen_width() / 980.).min(1.);
        (
            (screen_width() / scale) as f64,
            (screen_height() / scale) as f64,
        )
    }
    pub fn input(&mut self, block_mouse: bool) -> (f64, f64) {
        let mut input = (
            f64::from(is_key_down(KeyCode::Right)) - f64::from(is_key_down(KeyCode::Left)),
            f64::from(is_key_down(KeyCode::Down)) - f64::from(is_key_down(KeyCode::Up)),
        );
        if !block_mouse
            && is_mouse_button_pressed(MouseButton::Left)
            && !crate::chrome::over_controls(Vec2::from(mouse_position()), 3)
        {
            self.touch = Some(Vec2::from(mouse_position()));
        }
        if let Some(start) = self.touch {
            if !block_mouse && is_mouse_button_down(MouseButton::Left) {
                let delta = (Vec2::from(mouse_position()) - start) / 70.;
                input = (delta.x as f64, delta.y as f64);
            } else {
                self.touch = None;
            }
        }
        input
    }
    pub fn draw(&mut self, assets: &Assets, game: &Jetpack) {
        let scale = (screen_width() / 980.).min(1.);
        let left = (screen_width() - 980.) / 2.;
        let anchor = vec2(left + 490. * (1. - scale), 0.);
        let time = game.time as f32;
        if game.time < self.last_time {
            self.fire = Transition::new([1.]);
        }
        self.last_time = game.time;
        self.fire
            .set(game.time, [game.player.fire_scale], 0.2, Ease::Default);
        let fire_scale = self.fire.value(game.time)[0] as f32;
        let colors = [
            0x90d4f5, 0xad95f5, 0xf59393, 0xebca46, 0xed5b37, 0xc93c3c, 0x9719d1, 0x4d2dc2,
            0x51057a, 0x04266e, 0x04266e,
        ];
        clear_background(Color::from_hex(
            colors[(game.background_level as usize).min(10)],
        ));
        if let Some(progress) = game.transition {
            let c = Color::from_hex(colors[((game.background_level + 1) as usize).min(10)]);
            let y = (-1. + progress as f32) * screen_height() * 2.;
            for row in 0..(screen_height() * 2.).ceil() as i32 {
                let alpha = if row as f32 > screen_height() {
                    1. - (row as f32 - screen_height()) / screen_height()
                } else {
                    1.
                };
                draw_rectangle(
                    0.,
                    y + row as f32,
                    screen_width(),
                    1.1,
                    Color::new(c.r, c.g, c.b, alpha),
                );
            }
            let cloud = assets.scene("jetpack", "levelclouds.svg");
            let w = if screen_width() >= 1025. {
                3000.
            } else if screen_width() >= 661. {
                2000.
            } else {
                1150.
            };
            draw_texture_ex(
                cloud,
                (screen_width() - w) / 2.,
                y + screen_height() * 1.5
                    - 0.75
                        * if screen_width() > 1024. {
                            400.
                        } else if screen_width() > 660. {
                            280.
                        } else {
                            160.
                        },
                Color::new(1., 1., 1., 0.3),
                DrawTextureParams {
                    dest_size: Some(vec2(w, w * cloud.height() / cloud.width())),
                    ..Default::default()
                },
            );
            let size = if screen_width() > 1024. {
                400.
            } else if screen_width() > 660. {
                280.
            } else {
                160.
            };
            let text = (game.background_level + 2).to_string();
            let width = measure_text(&text, Some(&self.title), size as u16, 1.).width;
            draw_text_ex(
                &text,
                (screen_width() - width) / 2.,
                y + screen_height() * 1.5 - 0.75 * size + size,
                TextParams {
                    font: Some(&self.title),
                    font_size: size as u16,
                    color: Color::new(1., 1., 1., 0.3),
                    ..Default::default()
                },
            );
        }
        let draw = |name: &str,
                    x: f32,
                    y: f32,
                    w: f32,
                    h: f32,
                    source: Option<Rect>,
                    rotation: f32,
                    flip: bool,
                    alpha: f32,
                    pivot: Option<Vec2>| {
            draw_texture_ex(
                assets.scene("jetpack", name),
                anchor.x + x * scale,
                anchor.y + y * scale,
                Color::new(1., 1., 1., alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(w * scale, h * scale)),
                    source,
                    rotation,
                    flip_x: flip,
                    pivot: pivot.map(|p| anchor + p * scale),
                    ..Default::default()
                },
            );
        };
        for e in game.entities.iter().filter(|e| e.kind == Kind::Cloud) {
            let (w, h, ml, mt) = match e.cloud_type {
                0 => (140., 68., -70., 34.),
                1 => (130., 81., -65., 41.),
                _ => (135., 81., -68., 41.),
            };
            let size = 1. - e.cloud_type as f32 * 0.25;
            draw(
                "clouds.png",
                e.x as f32 + ml + (1. - size) * w / 2.,
                e.y as f32 + mt + (1. - size) * h / 2.,
                w * size,
                h * size,
                Some(Rect::new(e.cloud_type as f32 * 150., 0., w, h)),
                0.,
                false,
                1.,
                None,
            );
        }
        for e in game.entities.iter().filter(|e| e.kind != Kind::Cloud) {
            if e.hit {
                let elapsed = (game.time - e.hit_at) as f32;
                let alpha = 1. - Ease::Default.at(((elapsed - 0.5) / 1.5) as f64) as f32;
                let text = &e.message;
                let size = 40. * scale;
                let width = measure_text(text, Some(&self.font), size as u16, 1.).width;
                draw_text_ex(
                    text,
                    anchor.x + e.x as f32 * scale - width / 2.,
                    e.y as f32 * scale,
                    TextParams {
                        font: Some(&self.font),
                        font_size: size as u16,
                        color: if e.kind == Kind::Danger {
                            Color::new(0.91, 0., 0., alpha)
                        } else {
                            Color::new(1., 1., 1., alpha)
                        },
                        ..Default::default()
                    },
                );
                continue;
            }
            let style = e.style.split_whitespace().last().unwrap_or("");
            let index = style
                .chars()
                .last()
                .unwrap_or('1')
                .to_digit(10)
                .unwrap_or(1) as f32
                - 1.;
            let age = (game.time - e.born) as f32;
            let (name, source, w, h, mx, my, rotation, flip) = if style.starts_with("item-present")
            {
                let (w, h, mx, my) = match index as u32 {
                    0 => (49., 110., -25., -94.),
                    1 => (49., 122., -25., -95.),
                    2 => (46., 148., -23., -128.),
                    _ => (49., 103., -25., -89.),
                };
                let phase = (age / 2.).fract();
                let phase = if phase < 0.5 {
                    phase * 2.
                } else {
                    (1. - phase) * 2.
                };
                let angle = (-3. + ease_in_out(phase) * 6.).to_radians();
                (
                    "items.png",
                    Rect::new(index * 100., 200., w, h),
                    w,
                    h,
                    mx,
                    my,
                    angle,
                    false,
                )
            } else if style.starts_with("item-ball") {
                let (w, h, mx, my) = if index == 0. || index == 3. {
                    (43., 41., -22., -21.)
                } else {
                    (37., 51., -19., -26.)
                };
                (
                    "items.png",
                    Rect::new(index * 100., 100., w, h),
                    w,
                    h,
                    mx,
                    my,
                    0.,
                    false,
                )
            } else if style.starts_with("item-candy") {
                let h = if index == 3. { 18. } else { 35. };
                (
                    "items.png",
                    Rect::new(index * 100., 0., 35., h),
                    35.,
                    h,
                    -18.,
                    if index == 3. { 9. } else { -18. },
                    (1. - age.fract()) * std::f32::consts::TAU,
                    false,
                )
            } else if e.kind == Kind::Clock {
                (
                    "clock.png",
                    Rect::new(0., 0., 70., 116.),
                    70.,
                    116.,
                    -36.,
                    -90.,
                    0.,
                    false,
                )
            } else if index < 2. {
                let (x, w, h) = if index == 0. {
                    (0., 90., 118.)
                } else {
                    (130., 45., 63.)
                };
                (
                    "danger.png",
                    Rect::new(x, 0., w, h),
                    w,
                    h,
                    -56.,
                    -90.,
                    0.,
                    false,
                )
            } else {
                let frame = ((age / 0.6).fract() * 4.).floor();
                (
                    "birds.png",
                    Rect::new(0., frame * 100., 86., 100.),
                    86.,
                    100.,
                    -43.,
                    -50.,
                    0.,
                    index == 3.,
                )
            };
            draw(
                name,
                e.x as f32 + mx,
                e.y as f32 + my,
                w,
                h,
                Some(source),
                rotation,
                flip,
                1.,
                None,
            );
        }
        let p = &game.player;
        let center = vec2(p.x as f32, p.y as f32);
        let rot = (p.rotation as f32).to_radians();
        let w = if p.hit { 180. } else { 86. };
        let h = if p.hit { 215. } else { 216. };
        let mx = if p.hit { -94. } else { -43. };
        let my = if p.hit { -111. } else { -104. };
        let pivot = center
            + vec2(
                mx + (w + if p.hit { 4. } else { 0. }) / 2.,
                my + (h + if p.hit { 8. } else { 0. }) * 0.48,
            );
        let parent = Mat4::from_translation((anchor + pivot * scale).extend(0.))
            * Mat4::from_rotation_z(rot)
            * Mat4::from_translation(-(anchor + pivot * scale).extend(0.));
        unsafe {
            get_internal_gl().quad_gl.push_model_matrix(parent);
        }
        draw(
            if p.hit {
                "player-hit.svg"
            } else {
                "player.png"
            },
            center.x + mx,
            center.y + my,
            w,
            h,
            if p.hit {
                None
            } else {
                Some(Rect::new(if p.going_up { 100. } else { 0. }, 0., 86., 216.))
            },
            0.,
            false,
            1.,
            None,
        );
        if p.hit {
            let phase = (p.hit_duration as f32 / 0.6).fract();
            let t = if phase < 0.5 {
                phase * 2.
            } else {
                (1. - phase) * 2.
            };
            draw(
                "player-overlay.svg",
                center.x + mx,
                center.y + my,
                180.,
                215.,
                None,
                0.,
                false,
                0.9 - t * 0.4,
                None,
            );
        }
        let fire_x = center.x + mx + (w - 33.) / 2.;
        let fire_y = center.y + my + 131.;
        let phase = (time / 0.8).fract();
        let phase = if phase < 0.5 {
            phase * 2.
        } else {
            (1. - phase) * 2.
        };
        let orange = (0.95 + phase * 0.1) * fire_scale;
        draw(
            "fire.png",
            fire_x + (33. - 33. * orange) / 2.,
            fire_y,
            33. * orange,
            98. * orange,
            Some(Rect::new(0., 0., 33., 98.)),
            0.,
            false,
            1.,
            None,
        );
        let phase = (time / 1.4).fract();
        let phase = if phase < 0.5 {
            phase * 2.
        } else {
            (1. - phase) * 2.
        };
        let yellow = (0.6 + ease_in_out(phase) * 0.15) * fire_scale;
        draw(
            "fire.png",
            fire_x + 7. + (19. - 19. * yellow) / 2.,
            fire_y + 5. * fire_scale,
            19. * yellow,
            60. * yellow,
            Some(Rect::new(39., 20., 19., 60.)),
            0.,
            false,
            1.,
            None,
        );
        for (i, (x, y, delay)) in [
            (0., 10., 0.1),
            (20., 10., 0.),
            (5., 20., 0.2),
            (15., 15., 0.3),
            (5., 0., 0.4),
            (0., 10., 0.2),
            (20., 15., 0.4),
        ]
        .iter()
        .enumerate()
        {
            if time < *delay {
                continue;
            }
            let phase = ((time - *delay) / 0.5).fract();
            let small = i % 3 == 0;
            let s = if small { 0.7 } else { 1. };
            draw(
                "fire.png",
                center.x + mx + if p.hit { 79. } else { 28. } + x,
                center.y
                    + my
                    + if p.hit { 147. } else { 140. }
                    + y
                    + phase * if small { 36. } else { 48. },
                12. * s,
                12. * s,
                Some(Rect::new(if i % 2 == 0 { 60. } else { 40. }, 0., 12., 12.)),
                phase * std::f32::consts::TAU * 2.,
                false,
                1. - phase,
                None,
            );
        }
        unsafe {
            get_internal_gl().quad_gl.pop_model_matrix();
        }
        if game.finished {
            let target = (game.height as f32 - 412. - game.height as f32 * 0.1) * scale;
            let width = 452f32.min(game.width as f32 / 2.) * scale;
            let progress = (game.goal_time as f32 / 3.).clamp(0., 1.);
            draw_texture_ex(
                assets.scene("jetpack", "goal.png"),
                screen_width() - width,
                -412. * scale + (target + 412. * scale) * progress,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, width * 412. / 452.)),
                    ..Default::default()
                },
            );
        }
    }
}
