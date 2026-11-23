use crate::assets::Assets;
use macroquad::prelude::*;
use santa_tracker::ui::{Ease, Transition};
use santa_tracker::{
    jetpack::{Jetpack, Kind},
    village_data::ease_in_out,
};

pub struct JetpackView {
    images: crate::ui_render::PremultipliedMaterial,
    layer_text: Material,
    level_layer: Option<LevelLayer>,
    font: Font,
    title: Font,
    touch: Option<Vec2>,
    fire: Transition<1>,
    last_time: f64,
}
struct LevelLayer {
    key: (u32, u32, u32),
    target: RenderTarget,
    size: Vec2,
    top: f32,
}
impl JetpackView {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            images: crate::ui_render::premultiplied_material()?,
            layer_text: crate::ui_render::straight_alpha_source_over_material()?,
            level_layer: None,
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
    fn image(&self, texture: &Texture2D, x: f32, y: f32, tint: Color, params: DrawTextureParams) {
        gl_use_material(&self.images);
        draw_texture_ex(texture, x, y, tint, params);
    }
    fn level_layer(&mut self, assets: &Assets, level: u32, progress: f64) {
        let viewport = screen_width();
        let dpi = screen_dpi_scale();
        let key = (viewport.to_bits(), dpi.to_bits(), level);
        let (cloud_width, font_size) = if viewport >= 1025. {
            (3000., 400.)
        } else if viewport >= 661. {
            (2000., 280.)
        } else {
            (1150., 160.)
        };
        if self
            .level_layer
            .as_ref()
            .is_none_or(|layer| layer.key != key)
        {
            let cloud = assets.scene("jetpack", "levelclouds.svg");
            let cloud_height = cloud_width * cloud.height() / cloud.width();
            let text = level.to_string();
            let metrics = measure_text(&text, Some(&self.title), font_size as u16, 1.);
            // Guard the existing glyph/cloud bounds without moving either.
            let top = (font_size - metrics.offset_y).min(0.).floor() - 1.;
            let bottom = cloud_height
                .max(font_size - metrics.offset_y + metrics.height)
                .ceil()
                + 1.;
            let size = vec2(viewport, bottom - top);
            let target = render_target((size.x * dpi).ceil() as u32, (size.y * dpi).ceil() as u32);
            target.texture.set_filter(FilterMode::Linear);
            push_camera_state();
            let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., size.x, size.y));
            camera.render_target = Some(target.clone());
            set_camera(&camera);
            clear_background(BLANK);
            self.image(
                cloud,
                (viewport - cloud_width) / 2.,
                -top,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(cloud_width, cloud_height)),
                    ..Default::default()
                },
            );
            // Font atlas RGB is white, alpha is glyph coverage; use straight
            // RGB blending plus source-over ALPHA into the associated target.
            gl_use_material(&self.layer_text);
            draw_text_ex(
                &text,
                (viewport - metrics.width) / 2.,
                font_size - top,
                TextParams {
                    font: Some(&self.title),
                    font_size: font_size as u16,
                    color: WHITE,
                    ..Default::default()
                },
            );
            gl_use_default_material();
            pop_camera_state();
            self.level_layer = Some(LevelLayer {
                key,
                target,
                size,
                top,
            });
        }
        let layer = self.level_layer.as_ref().unwrap();
        let y = (-1. + progress as f32) * screen_height() * 2. + screen_height() * 1.5
            - 0.75 * font_size;
        // CSS opacity belongs to the entire ::after subtree, applied once.
        self.image(
            &layer.target.texture,
            0.,
            y + layer.top,
            Color::new(1., 1., 1., 0.3),
            DrawTextureParams {
                dest_size: Some(layer.size),
                flip_y: true,
                ..Default::default()
            },
        );
    }
    pub fn draw_sky(background_level: u32, transition: Option<f64>) {
        let colors = [
            0x90d4f5, 0xad95f5, 0xf59393, 0xebca46, 0xed5b37, 0xc93c3c, 0x9719d1, 0x4d2dc2,
            0x51057a, 0x04266e, 0x04266e,
        ];
        clear_background(Color::from_hex(colors[(background_level as usize).min(10)]));
        if let Some(progress) = transition {
            let c = Color::from_hex(colors[((background_level + 1) as usize).min(10)]);
            let y = (-1. + progress as f32) * screen_height() * 2.;
            let (w, h) = (screen_width(), screen_height());
            // Original CSS: a 200%-height background, solid through 50%,
            // then fading to transparent. Shared edges are rasterized once.
            // The old 1.1px strips overlapped at some subpixel translations
            // and composited the gradient twice, producing a visible flash.
            let transparent = Color::new(c.r, c.g, c.b, 0.);
            let vertices = vec![
                Vertex::new(0., y, 0., 0., 0., c),
                Vertex::new(w, y, 0., 0., 0., c),
                Vertex::new(w, y + h, 0., 0., 0., c),
                Vertex::new(0., y + h, 0., 0., 0., c),
                Vertex::new(w, y + 2. * h, 0., 0., 0., transparent),
                Vertex::new(0., y + 2. * h, 0., 0., 0., transparent),
            ];
            draw_mesh(&Mesh {
                vertices,
                indices: vec![0, 1, 2, 0, 2, 3, 3, 2, 4, 3, 4, 5],
                texture: None,
            });
        }
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
        gl_use_default_material();
        Self::draw_sky(game.background_level, game.transition);
        if let Some(progress) = game.transition {
            self.level_layer(assets, game.background_level + 2, progress);
        }
        // The goal and scene share z-index 2; the later scene paints above it.
        // CSS goal dimensions are viewport pixels, independent of scene scale.
        if game.finished {
            let progress = (game.goal_time / 3.).clamp(0., 1.);
            let top = -412. + (game.goal_target_y + 412.) * progress;
            let [x, y, w, h] = santa_tracker::jetpack::goal_image_rect(screen_width() as f64, top);
            self.image(
                assets.scene("jetpack", "goal.png"),
                x as f32,
                y as f32,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w as f32, h as f32)),
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
            self.image(
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
                gl_use_default_material();
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
        gl_use_default_material();
    }
}
