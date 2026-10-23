use crate::assets::Assets;
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
    hovered: String,
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
            hovered: String::new(),
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
                color: c,
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
            image(texture, r);
        }
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
        self.weather_time += get_frame_time().min(0.1);
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
            for s in self
                .data
                .sprites
                .clone()
                .iter()
                .filter(|s| s.layer == layer)
            {
                self.sprite(assets, s, scale, base, width, mobile);
            }
            if layer == 3 {
                let r = Rect::new(
                    base + width / 2. - 477. * scale,
                    (494. + if mobile { 150. } else { 0. }) * scale - self.scroll,
                    84. * scale,
                    133. * scale,
                );
                if let Some(t) =
                    self.texture(assets, "scenes/modvil/img/village/elf-glitch-front.svg")
                {
                    image(t, r);
                }
            }
        }
        clip(None);
        let center = base + width / 2.;
        if let Some(t) = self.texture(assets, "scenes/modvil/img/ornament.svg") {
            image(t, Rect::new(center - 225., -self.scroll, 450., 250.));
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
        draw_poly(
            r.x + 60.,
            r.y + 60.,
            128,
            65.,
            0.,
            Color::new(1., 1., 1., 0.1),
        );
        draw_poly(
            r.x + 60.,
            r.y + 60.,
            128,
            65.,
            0.,
            Color::new(0., 0., 0., 0.05),
        );
        draw_poly(
            r.x + 68.,
            r.y + 68.,
            128,
            60.,
            0.,
            Color::new(0., 0., 0., 0.1),
        );
        draw_poly(r.x + 60., r.y + 60., 128, 65., 0., WHITE);
        draw_poly(
            r.x + 60.,
            r.y + 60.,
            128,
            61.,
            0.,
            Color::from_hex(0xe70058),
        );
        draw_poly(
            r.x + 55.,
            r.y + 55.,
            128,
            60.,
            0.,
            Color::from_hex(0xff0160),
        );
        let angle = (-5f32).to_radians();
        let o = vec3(center - 5., r.y + 55., 0.);
        unsafe {
            get_internal_gl().quad_gl.push_model_matrix(
                Mat4::from_translation(o)
                    * Mat4::from_rotation_z(angle)
                    * Mat4::from_translation(-o),
            );
        }
        self.text(
            &self.data.strings["play"],
            center - 5.,
            r.y + 66.,
            32.,
            true,
            WHITE,
            true,
        );
        unsafe {
            get_internal_gl().quad_gl.pop_model_matrix();
        }
        if self.clicked(r) {
            self.pressed = None;
            self.notice = Some("runner".into());
        }
        let family = Rect::new(
            base + if screen_width() <= 768. { 12. } else { 32. },
            (screen_height() - 32.).min(height - self.scroll - 32.) - 40.,
            139.,
            40.,
        );
        rounded(
            Rect::new(family.x, family.y + 2., family.w, family.h),
            20.,
            Color::from_hex(0x4a7eaf),
        );
        rounded(family, 20., Color::from_hex(0x6bb4fd));
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
        if self.clicked(family) {
            self.pressed = None;
            self.notice = Some("familyguide".into());
        }
    }
    fn card(&mut self, assets: &mut Assets, card: &Card, mut rect: Rect) -> Option<String> {
        let hover = rect.contains(Vec2::from(mouse_position()));
        if hover {
            rect.y -= 4.;
            if self.hovered != card.id {
                self.hovered = card.id.clone();
                self.sounds.push("menu_over");
            }
        }
        rounded(
            Rect::new(rect.x, rect.y + 3., rect.w, rect.h),
            20.,
            Color::new(0., 0., 0., 0.2),
        );
        rounded(
            rect,
            20.,
            self.data
                .colors
                .get(&card.id)
                .map(|s| color(s))
                .unwrap_or(Color::from_hex(0x2a57ad)),
        );
        let art = Rect::new(
            rect.x + rect.w * 0.075,
            rect.y + rect.h * 0.06,
            rect.w * 0.85,
            rect.h * 0.6,
        );
        let animation = format!("img/card/{}.json", card.id);
        if assets.root.join(&animation).is_file() {
            self.animation(
                assets,
                &animation,
                art,
                Some(if hover { (self.time * 0.5).fract() } else { 0. }),
            );
        } else {
            let key = format!("img/scenes/{}_1x.png", card.id);
            if let Some(t) = self.texture(assets, &key) {
                cover(t, Rect::new(rect.x, rect.y, rect.w, rect.h * 0.72));
            }
        }
        let size = if rect.w <= 220. { 24. } else { 30. };
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
                rect.y + rect.h * 0.76 + i as f32 * size,
                size,
                true,
                WHITE,
                true,
            );
        }
        if self.clicked(rect) {
            self.pressed = None;
            self.sounds.push("generic_button_click");
            Some(card.id.clone())
        } else {
            None
        }
    }
    pub fn input(&mut self, total: f32) {
        self.time += get_frame_time();
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
        let total = hero + 72. + self.data.modules.len() as f32 * (h + 72.);
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
                                let w = (width * 0.3).max(280.).min(420.);
                                let ch = w * 0.75;
                                let r = if module.id == "movies" {
                                    Rect::new(
                                        base + width / 2.
                                            - (module.cards.len() as f32 * w
                                                + (module.cards.len() - 1) as f32 * 20.)
                                                / 2.
                                            + i as f32 * (w + 20.),
                                        y + h / 2. - ch / 2.,
                                        w,
                                        ch,
                                    )
                                } else {
                                    Rect::new(
                                        base + width * card.x - w / 2.,
                                        y + h * card.y - ch / 2.,
                                        w,
                                        ch,
                                    )
                                };
                                if r.y < screen_height() && r.bottom() > 0. {
                                    route = route.or_else(|| self.card(assets, card, r));
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
        if self.menu {
            route = route.or_else(|| self.draw_menu(assets));
        }
        route.or_else(|| self.notice.take())
    }
    fn draw_menu(&mut self, assets: &mut Assets) -> Option<String> {
        draw_rectangle(
            0.,
            0.,
            screen_width(),
            screen_height(),
            Color::new(72. / 255., 3. / 255., 184. / 255., 0.95),
        );
        let small = screen_width() < 768.;
        let size = if small { 130. } else { 200. };
        let cols = ((screen_width() / (size + 20.)).floor() as usize).clamp(2, 6);
        let mut cards = self
            .data
            .modules
            .iter()
            .flat_map(|m| m.cards.clone())
            .collect::<Vec<_>>();
        cards.insert(
            0,
            Card {
                id: "modvil".into(),
                title: "Santa's Village".into(),
                x: 0.,
                y: 0.,
            },
        );
        let mut result = None;
        for (i, card) in cards.iter().enumerate() {
            let x = (screen_width() - cols as f32 * (size + 20.)) / 2.
                + 10.
                + (i % cols) as f32 * (size + 20.);
            let y = 100. + (i / cols) as f32 * (size + 20.) - self.menu_scroll;
            if y < screen_height() && y + size > 0. {
                if let Some(route) = self.card(assets, card, Rect::new(x, y, size, size)) {
                    result = Some(route);
                }
            }
        }
        result
    }
}
