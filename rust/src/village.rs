use crate::{assets::Assets, ui_render::Shapes};
use santa_tracker::ui::{CardPlayer, Ease, Transition};
struct CardMotion {
    lift: Transition<1>,
    player: CardPlayer,
    hover: bool,
    auto_until: f64,
    last_seen: f64,
}
impl Default for CardMotion {
    fn default() -> Self {
        Self {
            lift: Transition::new([0.]),
            player: CardPlayer::default(),
            hover: false,
            auto_until: 0.,
            last_seen: 0.,
        }
    }
}
use macroquad::prelude::*;
use santa_tracker::{
    animation::Clip,
    village_data::{sample, Card, Keyframe, VillageData},
};
use std::collections::{HashMap, HashSet};
use thorvg::Thorvg;

struct AnimatedTexture<'a> {
    clip: Clip<'a>,
    texture: Option<Texture2D>,
    last_frame: f32,
    width: u32,
    height: u32,
}
pub struct Village<'a> {
    engine: &'a Thorvg,
    pub data: VillageData,
    pub scroll: f32,
    pub menu: bool,
    menu_scroll: f32,
    time: f32,
    animations: HashMap<String, AnimatedTexture<'a>>,
    failed: HashSet<String>,
    pub font: Font,
    title_font: Font,
    google_logo: Texture2D,
    clouds: [Texture2D; 3],
    weather: santa_tracker::weather::Weather,
    weather_time: f32,
    pressed: Option<Vec2>,
    drag_y: Option<f32>,
    drag_start: Option<Vec2>,
    pub notice: Option<String>,
    pub sounds: Vec<&'static str>,
    shapes: Shapes,
    play_motion: Transition<4>,
    play_ripple: Transition<2>,
    play_hover: bool,
    play_active: bool,
    family_motion: Transition<3>,
    family_hover: bool,
    cards: HashMap<String, CardMotion>,
    random: santa_tracker::random::Random,
    next_twiddle: f64,
    draw_alpha: f32,
    menu_opacity: Transition<1>,
    menu_changed: f64,
    last_menu: bool,
    pending_route: Option<(String, f64)>,
}
fn color(s: &str) -> Color {
    let n = u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap_or(0);
    Color::from_rgba((n >> 16) as u8, (n >> 8) as u8, n as u8, 255)
}
fn image(t: &Texture2D, r: Rect) {
    draw_texture_ex(
        t,
        r.x,
        r.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(r.size()),
            ..Default::default()
        },
    );
}
fn cover(t: &Texture2D, r: Rect) {
    let scale = (r.w / t.width()).max(r.h / t.height());
    let sw = r.w / scale;
    let sh = r.h / scale;
    draw_texture_ex(
        t,
        r.x,
        r.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(r.size()),
            source: Some(Rect::new(
                (t.width() - sw) / 2.,
                (t.height() - sh) / 2.,
                sw,
                sh,
            )),
            ..Default::default()
        },
    );
}
fn clip(rect: Option<Rect>) {
    unsafe {
        get_internal_gl().quad_gl.scissor(rect.map(|r| {
            (
                r.x.max(0.) as i32,
                r.y.max(0.) as i32,
                r.w.max(0.) as i32,
                r.h.max(0.) as i32,
            )
        }));
    }
}
pub(crate) fn rounded(r: Rect, radius: f32, c: Color) {
    let d = radius.min(r.w / 2.).min(r.h / 2.);
    draw_rectangle(r.x + d, r.y, r.w - 2. * d, r.h, c);
    draw_rectangle(r.x, r.y + d, r.w, r.h - 2. * d, c);
    for (x, y) in [
        (r.x + d, r.y + d),
        (r.right() - d, r.y + d),
        (r.x + d, r.bottom() - d),
        (r.right() - d, r.bottom() - d),
    ] {
        draw_poly(x, y, 96, d, 0., c);
    }
}
pub(crate) fn css_ease(x: f32) -> f32 {
    fn bezier(t: f32, a: f32, b: f32) -> f32 {
        3. * (1. - t) * (1. - t) * t * a + 3. * (1. - t) * t * t * b + t * t * t
    }
    let mut lo = 0.;
    let mut hi = 1.;
    for _ in 0..24 {
        let t = (lo + hi) * 0.5;
        if bezier(t, 0.25, 0.25) < x {
            lo = t;
        } else {
            hi = t;
        }
    }
    bezier((lo + hi) * 0.5, 0.1, 1.)
}
fn svg_texture(bytes: &[u8], width: u16, height: u16) -> Result<Texture2D, String> {
    let tree =
        resvg::usvg::Tree::from_data(bytes, &Default::default()).map_err(|e| e.to_string())?;
    let mut pixels =
        resvg::tiny_skia::Pixmap::new(width as u32, height as u32).ok_or("SVG allocation")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(
            width as f32 / tree.size().width(),
            height as f32 / tree.size().height(),
        ),
        &mut pixels.as_mut(),
    );
    let mut rgba = pixels.data().to_vec();
    for p in rgba.chunks_exact_mut(4) {
        if p[3] != 0 {
            for i in 0..3 {
                p[i] = (u32::from(p[i]) * 255 / u32::from(p[3])).min(255) as u8;
            }
        }
    }
    Ok(Texture2D::from_rgba8(width, height, &rgba))
}
impl<'a> Village<'a> {
    pub fn new(engine: &'a Thorvg) -> Result<Self, String> {
        let tree = resvg::usvg::Tree::from_data(
            include_bytes!("../data/google-logo.svg"),
            &Default::default(),
        )
        .map_err(|e| e.to_string())?;
        let mut pixels = resvg::tiny_skia::Pixmap::new(124, 76).ok_or("Logo allocation")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(2., 2.),
            &mut pixels.as_mut(),
        );
        let mut rgba = pixels.data().to_vec();
        for p in rgba.chunks_exact_mut(4) {
            if p[3] != 0 {
                for i in 0..3 {
                    p[i] = (u32::from(p[i]) * 255 / u32::from(p[3])).min(255) as u8;
                }
            }
        }
        let google_logo = Texture2D::from_rgba8(124, 76, &rgba);
        Ok(Self {
            engine,
            data: VillageData::original(),
            scroll: 0.,
            menu: false,
            menu_scroll: 0.,
            time: 0.,
            animations: HashMap::new(),
            failed: HashSet::new(),
            font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/GoogleSans-Regular.ttf"
            ))
            .map_err(|e| e.to_string())?,
            title_font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/Lobster-Regular.ttf"
            ))
            .map_err(|e| e.to_string())?,
            google_logo,
            clouds: [
                svg_texture(include_bytes!("../data/cloud-0.svg"), 230, 139)?,
                svg_texture(include_bytes!("../data/cloud-1.svg"), 197, 118)?,
                svg_texture(include_bytes!("../data/cloud-2.svg"), 118, 71)?,
            ],
            weather: santa_tracker::weather::Weather::new("village-weather"),
            weather_time: 0.,
            pressed: None,
            drag_y: None,
            drag_start: None,
            notice: None,
            sounds: vec!["music_start_village", "village_start"],
            shapes: Shapes::new()?,
            play_motion: Transition::new([-5., 3., 5., 13.]),
            play_ripple: Transition::new([1., 0.05]),
            play_hover: false,
            play_active: false,
            family_motion: Transition::new([0., 2., 2.]),
            family_hover: false,
            cards: HashMap::new(),
            random: santa_tracker::random::Random::seeded("village-ui"),
            next_twiddle: 8.,
            draw_alpha: 1.,
            menu_opacity: Transition::new([0.]),
            menu_changed: 0.,
            last_menu: false,
            pending_route: None,
        })
    }
    fn text(&self, s: &str, x: f32, y: f32, size: f32, title: bool, c: Color, center: bool) {
        let font = if title { &self.title_font } else { &self.font };
        let pixels = size.ceil() as u16;
        let font_scale = size / pixels as f32;
        let width = if center {
            measure_text(s, Some(font), pixels, font_scale).width / 2.
        } else {
            0.
        };
        draw_text_ex(
            s,
            x - width,
            y,
            TextParams {
                font: Some(font),
                font_size: pixels,
                font_scale,
                color: Color {
                    a: c.a * self.draw_alpha,
                    ..c
                },
                ..Default::default()
            },
        );
    }
    fn texture<'b>(&mut self, assets: &'b mut Assets, key: &str) -> Option<&'b Texture2D> {
        if self.failed.contains(key) {
            return None;
        }
        if !assets.has(key) {
            if let Err(error) = assets.load(key) {
                eprintln!("{error}");
                self.failed.insert(key.into());
                return None;
            }
        }
        Some(assets.texture(key))
    }
    fn animation(&mut self, assets: &Assets, key: &str, r: Rect, ratio: Option<f32>) {
        if self.failed.contains(key) {
            return;
        }
        if !self.animations.contains_key(key) {
            match Clip::load(self.engine, &assets.root.join(key)) {
                Ok(clip) => {
                    self.animations.insert(
                        key.into(),
                        AnimatedTexture {
                            clip,
                            texture: None,
                            last_frame: -1.,
                            width: 0,
                            height: 0,
                        },
                    );
                }
                Err(error) => {
                    eprintln!("{error}");
                    self.failed.insert(key.into());
                    return;
                }
            }
        }
        let anim = self.animations.get_mut(key).unwrap();
        let frame = if let Some(ratio) = ratio {
            ratio.clamp(0., 1.) * (anim.clip.frames - 0.001)
        } else {
            (self.time / anim.clip.duration).fract() * anim.clip.frames
        };
        let width = r.w.ceil().clamp(1., 1920.) as u32;
        let height = r.h.ceil().clamp(1., 1920.) as u32;
        if (anim.last_frame - frame).abs() >= 0.5
            || anim.width != width
            || anim.height != height
            || anim.texture.is_none()
        {
            match anim.clip.rasterize(width, height, frame) {
                Ok(bytes) => {
                    if anim.width != width || anim.height != height || anim.texture.is_none() {
                        let t = Texture2D::from_rgba8(width as u16, height as u16, &bytes);
                        t.set_filter(FilterMode::Linear);
                        anim.texture = Some(t);
                    } else {
                        anim.texture.as_ref().unwrap().update(&Image {
                            bytes,
                            width: width as u16,
                            height: height as u16,
                        });
                    }
                    anim.width = width;
                    anim.height = height;
                    anim.last_frame = frame;
                }
                Err(error) => {
                    eprintln!("{key}: {error}");
                    self.failed.insert(key.into());
                    return;
                }
            }
        }
        if let Some(texture) = &anim.texture {
            draw_texture_ex(
                texture,
                r.x,
                r.y,
                Color::new(1., 1., 1., self.draw_alpha),
                DrawTextureParams {
                    dest_size: Some(r.size()),
                    ..Default::default()
                },
            );
        }
    }
    pub fn overlay_cards(
        &mut self,
        assets: &mut Assets,
        ids: &[String; 2],
        rects: [Rect; 2],
        opacity: f32,
    ) -> Option<String> {
        self.time += get_frame_time();
        self.pressed = if is_mouse_button_released(MouseButton::Left) {
            Some(Vec2::from(mouse_position()))
        } else {
            None
        };
        let mut result = None;
        for (id, rect) in ids.iter().zip(rects) {
            let card = Card {
                id: id.clone(),
                title: self.data.scene_names.get(id).cloned().unwrap_or(id.clone()),
                x: 0.,
                y: 0.,
            };
            result = result.or(self.card(assets, &card, rect, false, opacity, true));
        }
        result
    }
    fn clicked(&self, r: Rect) -> bool {
        self.pressed.is_some_and(|p| r.contains(p))
    }
    fn sprite(
        &mut self,
        assets: &mut Assets,
        s: &santa_tracker::village_data::Sprite,
        scale: f32,
        base: f32,
        width: f32,
        mobile: bool,
    ) {
        let time = self.time - s.delay;
        let key = if time >= 0. && s.duration > 0. {
            self.data
                .frames
                .get(&s.animation)
                .map(|t| sample(t, (time / s.duration).fract(), &s.ease))
                .unwrap_or_default()
        } else {
            Keyframe::default()
        };
        let x = base + ((if s.center { width / scale / 2. } else { 0. }) + s.x + key.x) * scale;
        let y = ((if mobile { s.mobile_y } else { s.y }) + key.y) * scale - self.scroll;
        let w = s.width * scale;
        let h = s.height * scale;
        let viewport = Rect::new(
            base,
            0.,
            width,
            if mobile { 720. } else { 1050. } - self.scroll,
        );
        if s.name.starts_with("waterspout-") {
            let r = Rect::new(
                base + width / 2. - 55. * scale,
                (607. + if mobile { 150. } else { 0. }) * scale - self.scroll,
                117. * scale,
                132. * scale,
            );
            let top = r.y.max(0.);
            let bottom = r.bottom().min(viewport.h);
            clip(Some(Rect::new(r.x, top, r.w, (bottom - top).max(0.))));
        }
        if let Some(t) = self.texture(assets, &s.asset) {
            draw_texture_ex(
                t,
                x + (1. - key.sx) * w / 2.,
                y + (1. - key.sy) * h / 2.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w * key.sx.abs(), h * key.sy.abs())),
                    rotation: key.rotation.to_radians(),
                    flip_x: key.sx < 0.,
                    flip_y: key.sy < 0.,
                    ..Default::default()
                },
            );
        }
        if s.name.starts_with("waterspout-") {
            clip(Some(viewport));
        }
        if s.name == "elfchair" {
            let angle = key.rotation.to_radians();
            let delta = vec2(35. - s.width / 2., 55. - s.height / 2.);
            let delta = vec2(
                delta.x * angle.cos() - delta.y * angle.sin(),
                delta.x * angle.sin() + delta.y * angle.cos(),
            ) * scale;
            if let Some(t) = self.texture(assets, "scenes/modvil/img/village/elfchair-wheel.svg") {
                draw_texture_ex(
                    t,
                    x + w / 2. + delta.x - 15. * scale,
                    y + h / 2. + delta.y - 15. * scale,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(30. * scale, 30. * scale)),
                        rotation: angle + self.time / 0.5 * std::f32::consts::TAU,
                        ..Default::default()
                    },
                );
            }
        }
    }
    fn hero(&mut self, assets: &mut Assets, width: f32, base: f32, height: f32) {
        let stops = [
            (0., Color::from_hex(0x00c1f2)),
            (0.1771, Color::from_hex(0xb4e5e6)),
            (0.4167, Color::from_hex(0xfff8bd)),
            (1., Color::from_hex(0xfff8bd)),
        ];
        for y in 0..height.ceil() as i32 {
            let p = y as f32 / height;
            let (a, b) = stops
                .windows(2)
                .find(|p2| p <= p2[1].0)
                .map(|p2| (p2[0], p2[1]))
                .unwrap();
            let t = ((p - a.0) / (b.0 - a.0)).clamp(0., 1.);
            let c = Color::new(
                a.1.r + (b.1.r - a.1.r) * t,
                a.1.g + (b.1.g - a.1.g) * t,
                a.1.b + (b.1.b - a.1.b) * t,
                1.,
            );
            draw_rectangle(base, y as f32 - self.scroll, width, 1.1, c);
        }
        self.weather_time += get_frame_time();
        while self.weather_time >= 1. / 60. || self.weather.flakes.is_empty() {
            self.weather.step(width as f64, height as f64);
            self.weather_time -= 1. / 60.;
        }
        clip(Some(Rect::new(
            base,
            0.,
            width,
            (height - self.scroll).max(0.),
        )));
        for f in &self.weather.flakes {
            let alpha = if f.y > 0.9 {
                1. - (f.y - 0.9) / 0.1
            } else {
                1.
            };
            draw_poly(
                base + f.x as f32 * width,
                f.y as f32 * height - self.scroll,
                32,
                f.d as f32,
                0.,
                Color::new(1., 1., 1., (f.opacity * alpha) as f32),
            );
        }
        for c in &self.weather.clouds {
            draw_texture_ex(
                &self.clouds[c.image],
                base + (c.x as f32 * width) - c.width as f32 / 2.,
                c.y as f32 * height - self.scroll - c.height as f32 / 2.,
                Color::new(1., 1., 1., c.opacity as f32),
                DrawTextureParams {
                    dest_size: Some(vec2(c.width as f32, c.height as f32)),
                    ..Default::default()
                },
            );
        }
        let mobile = screen_width() <= 1024.;
        let scale = if mobile { 0.6 } else { 1. };
        clip(Some(Rect::new(
            base,
            0.,
            width,
            (height - self.scroll).max(0.),
        )));
        for layer in 1..=4 {
            let key = format!("scenes/modvil/img/village/{layer}.svg");
            if let Some(t) = self.texture(assets, &key) {
                let w = 2040. * scale;
                let h = t.height() * w / t.width();
                image(
                    t,
                    Rect::new(base + (width - w) / 2., height - h - self.scroll, w, h),
                );
            }
            if layer == 3 {
                let r = Rect::new(
                    base + width / 2. - 477. * scale,
                    (494. + if mobile { 150. } else { 0. }) * scale - self.scroll,
                    84. * scale,
                    133. * scale,
                );
                let glitch = self.time.rem_euclid(15.) < 5.;
                if let Some(t) =
                    self.texture(assets, "scenes/modvil/img/village/elf-glitch-front.svg")
                {
                    image(t, r);
                }
                if glitch {
                    let t = self.time.rem_euclid(15.);
                    let phase = (t / 3.).rem_euclid(2.);
                    let phase = if phase > 1. { 2. - phase } else { phase };
                    for (track, dx, dy, flip) in [
                        ("glitch-anim-1-horizontal", 3., 0., false),
                        ("glitch-anim-2-horizontal", -3., 0., false),
                        ("glitch-anim-3-horizontal", 0., -7., true),
                    ] {
                        if let Some(frames) = self.data.frames.get(track) {
                            let key = sample(frames, phase, "linear");
                            let (top, bottom) = if flip {
                                (
                                    1. - key.clip_top.max(key.clip_bottom),
                                    1. - key.clip_top.min(key.clip_bottom),
                                )
                            } else {
                                (
                                    key.clip_top.min(key.clip_bottom),
                                    key.clip_top.max(key.clip_bottom),
                                )
                            };
                            let bounds = Rect::new(
                                r.x + dx * scale,
                                (r.y + dy * scale + top * r.h).max(0.),
                                r.w,
                                ((bottom - top) * r.h).max(0.),
                            );
                            clip(Some(bounds));
                            if let Some(tex) = self
                                .texture(assets, "scenes/modvil/img/village/elf-glitch-front.svg")
                            {
                                draw_texture_ex(
                                    tex,
                                    r.x + dx * scale,
                                    r.y + dy * scale,
                                    WHITE,
                                    DrawTextureParams {
                                        dest_size: Some(r.size()),
                                        flip_x: flip,
                                        flip_y: flip,
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                    }
                    clip(Some(Rect::new(
                        base,
                        0.,
                        width,
                        (height - self.scroll).max(0.),
                    )));
                    // The fifth child has an unresolved flash animation in the
                    // original stylesheet, so it remains the unmodified image.
                    if let Some(t) =
                        self.texture(assets, "scenes/modvil/img/village/elf-glitch-front.svg")
                    {
                        image(t, r);
                    }
                }
            }
            for s in self
                .data
                .sprites
                .clone()
                .iter()
                .filter(|s| s.layer == layer)
            {
                self.sprite(assets, s, scale, base, width, mobile);
            }
        }
        clip(None);
        let center = base + width / 2.;
        if let Some(t) = self.texture(assets, "scenes/modvil/img/ornament.svg") {
            let w = 250. * t.width() / t.height();
            image(t, Rect::new(center - w / 2., -self.scroll, w, 250.));
        }
        let title_size = if screen_width() < 768. { 44. } else { 72. };
        image(
            &self.google_logo,
            Rect::new(center - 31., 60. - self.scroll, 62., 38.),
        );
        let origin = vec3(center, 131. - self.scroll, 0.);
        let skew = Mat4::from_cols(
            vec4(1., 0., 0., 0.),
            vec4((-4f32).to_radians().tan(), 1., 0., 0.),
            vec4(0., 0., 1., 0.),
            vec4(0., 0., 0., 1.),
        );
        let transform = Mat4::from_translation(origin)
            * Mat4::from_rotation_z((-3f32).to_radians())
            * skew
            * Mat4::from_translation(-origin);
        unsafe {
            get_internal_gl().quad_gl.push_model_matrix(transform);
        }
        self.text(
            &self.data.strings["santatracker"],
            center,
            160. - self.scroll,
            title_size,
            true,
            Color::new(0., 0., 0., 0.25),
            true,
        );
        self.text(
            &self.data.strings["santatracker"],
            center,
            158. - self.scroll,
            title_size,
            true,
            WHITE,
            true,
        );
        unsafe {
            get_internal_gl().quad_gl.pop_model_matrix();
        }
        self.text(
            "Explore, play and learn with Santa's",
            center,
            204. - self.scroll,
            21.6,
            false,
            Color::new(0., 0., 0., 0.33),
            true,
        );
        self.text(
            "elves all December long",
            center,
            225.6 - self.scroll,
            21.6,
            false,
            Color::new(0., 0., 0., 0.33),
            true,
        );
        self.text(
            "Explore, play and learn with Santa's",
            center,
            203. - self.scroll,
            21.6,
            false,
            WHITE,
            true,
        );
        self.text(
            "elves all December long",
            center,
            224.6 - self.scroll,
            21.6,
            false,
            WHITE,
            true,
        );
        let r = Rect::new(center - 60., 261.171875 - self.scroll, 120., 120.);
        let mouse = Vec2::from(mouse_position());
        let now = self.time as f64;
        let hover = !self.menu && r.contains(mouse);
        if hover && !self.play_hover {
            self.sounds.push("generic_button_over");
        }
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            self.sounds.push("generic_button_click");
        }
        self.play_hover = hover;
        let down = self.drag_start.is_some_and(|p| r.contains(p))
            && is_mouse_button_down(MouseButton::Left);
        let active = down || self.play_active;
        self.play_motion.set(
            now,
            if active {
                [0., 0., 0., 0.]
            } else if hover {
                [-7., 3., 7., 18.]
            } else {
                [-5., 3., 5., 13.]
            },
            if active { 0.15 } else { 0.25 },
            Ease::InOut,
        );
        self.play_ripple.set(
            now,
            if active { [25., 0.] } else { [1., 0.05] },
            if active { 0.3 } else { 0. },
            Ease::Out,
        );
        let m = self.play_motion.value(now);
        let ripple = self.play_ripple.value(now);
        let c = r.center();
        self.shapes.circle(c, 76., Color::new(0., 0., 0., 0.05), 0.);
        self.shapes.circle(
            c,
            65. + ripple[0] as f32,
            Color::new(0., 0., 0., ripple[1] as f32),
            0.,
        );
        self.shapes.circle(c, 65., WHITE, 0.);
        let front = c + Vec2::splat(m[0] as f32);
        self.shapes.circle(
            front + Vec2::splat(m[3] as f32),
            60.,
            Color::new(0., 0., 0., 0.1),
            0.,
        );
        self.shapes.circle(
            front + Vec2::splat(m[2] as f32),
            60.,
            Color::from_hex(0xe70058),
            0.,
        );
        self.shapes.circle(
            front + Vec2::splat(m[1] as f32),
            60. + (m[1] / 3.) as f32,
            Color::from_hex(0xe70058),
            0.,
        );
        self.shapes
            .circle(front, 60., Color::from_hex(0xff0160), 0.);
        let angle = (-5f32).to_radians();
        let o = vec3(center + m[0] as f32, r.y + 60. + m[0] as f32, 0.);
        unsafe {
            get_internal_gl().quad_gl.push_model_matrix(
                Mat4::from_translation(o)
                    * Mat4::from_rotation_z(angle)
                    * Mat4::from_translation(-o),
            );
        }
        self.text(
            &self.data.strings["play"],
            center + m[0] as f32,
            r.y + 71. + m[0] as f32,
            32.,
            true,
            WHITE,
            true,
        );
        unsafe {
            get_internal_gl().quad_gl.pop_model_matrix();
        }
        if !self.menu && self.clicked(r) {
            self.pressed = None;
            self.play_active = true;
            self.pending_route = Some(("runner".into(), now + 0.3));
        }
        let family = Rect::new(
            base + if screen_width() <= 768. { 12. } else { 32. },
            (screen_height() - 32.).min(height - self.scroll - 32.) - 40.,
            139.,
            40.,
        );
        let hover = !self.menu && family.contains(mouse);
        if hover && !self.family_hover {
            self.sounds.push("generic_button_over");
        }
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            self.sounds.push("generic_button_click");
        }
        self.family_hover = hover;
        let down = self.drag_start.is_some_and(|p| family.contains(p))
            && is_mouse_button_down(MouseButton::Left);
        self.family_motion.set(
            now,
            if down {
                [2., 0., 0.]
            } else if hover {
                [-2., 4., 3.]
            } else {
                [0., 2., 2.]
            },
            0.2,
            Ease::Default,
        );
        let motion = self.family_motion.value(now);
        let mut family = family;
        family.y += motion[0] as f32;
        self.shapes.rounded(
            Rect::new(
                family.x + motion[2] as f32,
                family.y + motion[1] as f32 + motion[2] as f32,
                family.w,
                family.h,
            ),
            20.,
            Color::new(0., 0., 0., 0.1),
            0.,
        );
        self.shapes.rounded(
            Rect::new(family.x, family.y + motion[1] as f32, family.w, family.h),
            20.,
            Color::from_hex(0x4b7eb1),
            0.,
        );
        self.shapes
            .rounded(family, 20., Color::from_hex(0x6bb4fd), 0.);
        if let Some(t) = self.texture(assets, "scenes/modvil/img/family-icon.png") {
            image(t, Rect::new(family.x + 5., family.y + 4., 32., 32.));
        }
        self.text(
            "Family Guide",
            family.x + 45.,
            family.y + 26.,
            14.,
            false,
            WHITE,
            false,
        );
        if !self.menu && self.clicked(family) {
            self.pressed = None;
            self.notice = Some("familyguide".into());
        }
    }
    fn card(
        &mut self,
        assets: &mut Assets,
        card: &Card,
        mut rect: Rect,
        menu: bool,
        opacity: f32,
        overlay: bool,
    ) -> Option<String> {
        self.draw_alpha = opacity;
        let now = self.time as f64;
        let hover = rect.contains(Vec2::from(mouse_position())) && (menu || !self.menu);
        let key = format!("{}:{}", if menu { "menu" } else { "village" }, card.id);
        let state = self.cards.entry(key.clone()).or_default();
        state.last_seen = now;
        if hover && !state.hover {
            self.sounds.push("menu_over");
        } else if !hover && state.hover {
            self.sounds.push("menu_out");
        }
        state.hover = hover;
        state
            .lift
            .set(now, [if hover { -4. } else { 0. }], 0.16, Ease::Default);
        let lift = state.lift.value(now)[0] as f32;
        rect.y += lift;
        let radius = if overlay {
            if screen_width() <= 400. || screen_height() <= 400. {
                10.
            } else {
                12.
            }
        } else if menu {
            if screen_width() < 768. {
                20.
            } else {
                30.
            }
        } else {
            20.
        };
        let h = (-lift / 4.).clamp(0., 1.);
        self.shapes.rounded(
            Rect::new(rect.x, rect.y + 2. + 4. * h, rect.w, rect.h),
            radius,
            Color::new(0., 0., 0., 0.3 * opacity),
            8. + 4. * h,
        );
        self.shapes.rounded(
            rect,
            radius,
            Color {
                a: opacity,
                ..self
                    .data
                    .colors
                    .get(&card.id)
                    .map(|s| color(s))
                    .unwrap_or(Color::from_hex(0x2a57ad))
            },
            0.,
        );
        let size = if overlay {
            if screen_width() <= 400. || screen_height() <= 400. {
                15.36
            } else {
                19.2
            }
        } else if !menu
            && screen_width() <= 600.
            && ["likealight", "yulelog", "penguinproof"].contains(&card.id.as_str())
        {
            16.8
        } else if menu {
            if screen_width() < 768. {
                16.8
            } else {
                24.
            }
        } else if screen_width() > 1024. {
            31.2
        } else {
            26.4
        };
        let line_height = size;
        let art_top = (rect.h - rect.h * 0.6 - size * 1.5) / 2.;
        let art = Rect::new(
            rect.x + rect.w * 0.075,
            rect.y + art_top,
            rect.w * 0.85,
            rect.h * 0.6,
        );
        let animation = format!("img/card/{}.json", card.id);
        if assets.root.join(&animation).is_file() {
            let duration = if let Some(anim) = self.animations.get(&animation) {
                anim.clip.duration as f64
            } else {
                std::fs::read_to_string(assets.root.join(&animation))
                    .ok()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                    .map(|v| {
                        (v["op"].as_f64().unwrap_or(60.) - v["ip"].as_f64().unwrap_or(0.))
                            / v["fr"].as_f64().unwrap_or(60.)
                    })
                    .unwrap_or(1.)
            };
            let state = self.cards.get_mut(&key).unwrap();
            state
                .player
                .set(now, hover || now < state.auto_until, duration);
            let frame = state.player.frame(now, duration);
            self.animation(assets, &animation, art, Some(frame));
        } else {
            let preferred = format!("img/scenes/{}_1x.png", card.id);
            let key = if assets.root.join(&preferred).is_file() {
                preferred
            } else {
                "img/unknown.png".into()
            };
            if let Some(t) = self.texture(assets, &key) {
                self.shapes.cover(t, rect, radius, opacity, true);
            }
        }
        let words = card.title.split_whitespace().collect::<Vec<_>>();
        let mut lines = vec![String::new()];
        for word in words {
            let current = lines.last().unwrap();
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            if !current.is_empty()
                && measure_text(&candidate, Some(&self.title_font), size as u16, 1.).width
                    > rect.w - 24.
            {
                lines.push(word.into());
            } else {
                *lines.last_mut().unwrap() = candidate;
            }
        }
        for (i, line) in lines.iter().enumerate() {
            self.text(
                line,
                rect.x + rect.w / 2.,
                art.bottom() + size * 1.05 + i as f32 * line_height,
                size,
                true,
                WHITE,
                true,
            );
        }
        self.draw_alpha = 1.;
        if (menu || !self.menu) && self.clicked(Rect::new(rect.x, rect.y - lift, rect.w, rect.h)) {
            self.pressed = None;
            self.sounds.push("generic_button_click");
            Some(card.id.clone())
        } else {
            None
        }
    }
    pub fn input(&mut self, total: f32) {
        let old_time = self.time;
        self.time += get_frame_time();
        for (seconds, event) in [
            (5., "village_top_skier"),
            (14., "village_top_bikeride"),
            (8., "village_top_surprise"),
            (5., "village_top_fountain"),
        ] {
            if (old_time / seconds).floor() < (self.time / seconds).floor() && !self.menu {
                self.sounds.push(event);
            }
        }
        let now = self.time as f64;
        if now >= self.next_twiddle {
            self.next_twiddle = now + 8.;
            let mut visible: Vec<String> = self
                .cards
                .iter()
                .filter(|(_, s)| now - s.last_seen < 0.2)
                .map(|(k, _)| k.clone())
                .collect();
            visible.sort();
            let mut factor = 0.5;
            for key in visible {
                if self.random.next() > factor {
                    self.cards.get_mut(&key).unwrap().auto_until = now + 1.5 + self.random.next();
                    break;
                }
                factor *= 0.5;
            }
        }
        if self.menu != self.last_menu {
            self.last_menu = self.menu;
            self.menu_changed = now;
            self.sounds
                .push(if self.menu { "nav_open" } else { "nav_close" });
            if self.menu {
                self.menu_scroll = 0.;
            }
        }
        self.menu_opacity
            .set(now, [f64::from(self.menu)], 0.25, Ease::Sidebar);
        self.pressed = None;
        let mouse = Vec2::from(mouse_position());
        if is_mouse_button_pressed(MouseButton::Left) {
            self.drag_y = Some(mouse.y);
            self.drag_start = Some(mouse);
        }
        let mut drag = 0.;
        if let Some(last) = self.drag_y {
            if is_mouse_button_down(MouseButton::Left) {
                drag = last - mouse.y;
                self.drag_y = Some(mouse.y);
            }
            if is_mouse_button_released(MouseButton::Left) {
                if self.drag_start.is_some_and(|p| p.distance(mouse) < 5.) {
                    self.pressed = Some(mouse);
                }
                self.drag_y = None;
                self.drag_start = None;
            }
        }
        let (_, wheel) = mouse_wheel();
        let scroll = if self.menu {
            &mut self.menu_scroll
        } else {
            &mut self.scroll
        };
        *scroll = (*scroll - wheel * 60. + drag).max(0.);
        if !self.menu {
            *scroll = scroll.min((total - screen_height()).max(0.));
        }
        if is_key_pressed(KeyCode::PageDown) {
            *scroll += screen_height() * 0.9;
        }
        if is_key_pressed(KeyCode::PageUp) {
            *scroll = (*scroll - screen_height() * 0.9).max(0.);
        }
        if is_key_pressed(KeyCode::Home) {
            *scroll = 0.;
        }
    }
    pub fn draw(&mut self, assets: &mut Assets) -> Option<String> {
        let width = screen_width().min(1920.);
        let base = (screen_width() - width) / 2.;
        let hero = if screen_width() <= 1024. { 720. } else { 1050. };
        let h = (width * 0.9).max(800.);
        let total = hero
            + 72.
            + self
                .data
                .modules
                .iter()
                .map(|m| {
                    let mh = if m.id == "movies" {
                        (width * 0.55555555).max(800.)
                    } else if m.id == "footer" {
                        (width * 0.694).max(800.)
                    } else {
                        h
                    };
                    mh + if m.no_divider { 0. } else { 72. }
                })
                .sum::<f32>();
        self.input(total);
        let mut route = None;
        clear_background(Color::from_hex(0xe9f9fd));
        if self.scroll < hero {
            self.hero(assets, width, base, hero);
        }
        let mut y = hero - self.scroll;
        if let Some(t) = self.texture(assets, "scenes/modvil/img/village/divider.svg") {
            let tw = t.width() * 72. / t.height();
            let mut x = base;
            while x < base + width {
                image(t, Rect::new(x, y, tw, 72.));
                x += tw;
            }
        }
        y += 72.;
        for module in self.data.modules.clone() {
            let width = width.max(414.);
            let h = if module.id == "movies" {
                (width * 0.55555555).max(800.)
            } else if module.id == "footer" {
                (width * 0.694).max(800.)
            } else {
                (width * 0.9).max(800.)
            };
            let rect = Rect::new(base, y, width, h);
            if y < screen_height() && y + h > 0. {
                clip(Some(Rect::new(
                    base,
                    y.max(0.),
                    width,
                    (y + h).min(screen_height()) - y.max(0.),
                )));
                draw_rectangle(base, y, width, h, color(&module.color));
                for (idx, part) in module.parts.iter().enumerate() {
                    let key = format!("scenes/modvil/img/modules/{}/{idx}", module.id);
                    match part.as_str() {
                        "static-png" | "static-svg" => {
                            let key = format!("{key}.{}", part.trim_start_matches("static-"));
                            if let Some(t) = self.texture(assets, &key) {
                                cover(t, rect);
                            }
                        }
                        "loop" => self.animation(assets, &format!("{key}.json"), rect, None),
                        "scroll" => {
                            let top = (self.scroll + y - screen_height()).max(0.);
                            let bottom = (self.scroll + y + h).min(total - screen_height());
                            let ratio = if bottom > top {
                                (self.scroll - top) / (bottom - top)
                            } else {
                                0.
                            };
                            self.animation(assets, &format!("{key}.json"), rect, Some(ratio));
                        }
                        "cards" => {
                            for (i, card) in module.cards.iter().enumerate() {
                                let [x, cy, w, ch] = santa_tracker::ui::card_rect(
                                    &module.id,
                                    i,
                                    module.cards.len(),
                                    (card.x, card.y),
                                    width,
                                    screen_width(),
                                    y,
                                );
                                let r = Rect::new(base + x, cy, w, ch);
                                if r.y < screen_height() && r.bottom() > 0. {
                                    route = route
                                        .or_else(|| self.card(assets, card, r, false, 1., false));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                clip(None);
            }
            y += h;
            if !module.no_divider {
                if y < screen_height() && y + 72. > 0. {
                    let key = format!("scenes/modvil/img/modules/{}/divider.svg", module.id);
                    if let Some(t) = self.texture(assets, &key) {
                        let tw = t.width() * 72. / t.height();
                        let mut x = base;
                        while x < base + width {
                            image(t, Rect::new(x, y, tw, 72.));
                            x += tw;
                        }
                    }
                }
                y += 72.;
            }
        }
        if self.menu || self.menu_opacity.value(self.time as f64)[0] > 0.001 {
            route = self.draw_menu(assets).or(route);
        }
        if self
            .pending_route
            .as_ref()
            .is_some_and(|(_, at)| self.time as f64 >= *at)
        {
            let (target, _) = self.pending_route.take().unwrap();
            self.play_active = false;
            route = Some(target);
        }
        route.or_else(|| self.notice.take())
    }
    fn draw_menu(&mut self, assets: &mut Assets) -> Option<String> {
        draw_rectangle(
            0.,
            0.,
            screen_width(),
            screen_height(),
            Color::new(
                72. / 255.,
                3. / 255.,
                184. / 255.,
                0.95 * self.menu_opacity.value(self.time as f64)[0] as f32,
            ),
        );
        let small = screen_width() < 768.;
        let size = if small { 130. } else { 200. };
        let cols = ((screen_width() / (size + 20.)).floor() as usize).clamp(2, 6);
        let nav: serde_json::Value =
            serde_json::from_str(include_str!("../data/navigation.json")).unwrap();
        let videos = nav["videos"].as_array().unwrap();
        let mut order = 0usize;
        let mut available = std::collections::VecDeque::new();
        let mut cards = Vec::new();
        for id in nav["cards"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(serde_json::Value::as_str)
        {
            let wide = videos.iter().any(|v| v == id);
            let mut slot = order;
            if wide {
                order += 2;
                if slot % cols + 1 == cols {
                    available.push_back(slot);
                    slot += 1;
                    order += 1;
                }
            } else if let Some(previous) = available.pop_front() {
                slot = previous;
            } else {
                order += 1;
            }
            cards.push((
                slot,
                wide,
                Card {
                    id: id.into(),
                    title: self.data.scene_names.get(id).cloned().unwrap_or(id.into()),
                    x: 0.,
                    y: 0.,
                },
            ));
        }
        cards.sort_by_key(|(slot, _, _)| *slot);
        let mut result = None;
        for (i, wide, card) in &cards {
            let i = *i;
            let x = (screen_width() - cols as f32 * (size + 20.)) / 2.
                + 10.
                + (i % cols) as f32 * (size + 20.);
            let reveal = if self.menu {
                Ease::Out.at((self.time as f64 - self.menu_changed - 0.2) / 0.2) as f32
            } else {
                0.
            };
            let y = 90. + (i / cols) as f32 * (size + 20.) - self.menu_scroll + 32. * (1. - reveal);
            if reveal <= 0. {
                continue;
            }
            if y < screen_height() && y + size > 0. {
                if let Some(route) = self.card(
                    assets,
                    card,
                    Rect::new(x, y, if *wide { size * 2. + 20. } else { size }, size),
                    true,
                    reveal,
                    false,
                ) {
                    result = Some(route);
                }
            }
        }
        result
    }
}
