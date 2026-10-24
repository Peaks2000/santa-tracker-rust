//! Original tutorial art and cloud clip, native SVG strips / decoded video atlas.
use crate::assets::Assets;
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};
use santa_tracker::tutorial::Tutorial;
const VERTEX: &str = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;uniform mat4 Model;uniform mat4 Projection;varying vec2 uv;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;}"#;
const FRAGMENT: &str = r#"#version 100
precision highp float;varying vec2 uv;uniform sampler2D Texture;uniform sampler2D Mask;uniform vec4 Source;uniform float Opacity;
float cloud(vec2 p){if(p.x<0.0||p.y<0.0||p.x>1.0||p.y>1.0)return 0.0;return texture2D(Mask,p).a;}
void main(){vec4 c=texture2D(Texture,uv);vec2 local=(uv-Source.xy)/Source.zw;float m=cloud(local);float sh=0.15*cloud(local-vec2(4.0/275.0,12.0/270.0));float a=m+sh*(1.0-m);
vec3 art=c.rgb*c.a+vec3(1.0)*(1.0-c.a);
gl_FragColor=vec4(art*m/max(a,0.000001),a*Opacity);}"#;
pub struct TutorialView {
    pub state: Tutorial,
    route: String,
    material: Material,
    _mask: Texture2D,
    captured: bool,
    shown: bool,
    was_hidden: bool,
    image_origin: f64,
}
impl TutorialView {
    pub fn new() -> Result<Self, String> {
        let tree = resvg::usvg::Tree::from_data(
            include_bytes!("../data/tutorial-mask.svg"),
            &Default::default(),
        )
        .map_err(|e| e.to_string())?;
        let mut p = resvg::tiny_skia::Pixmap::new(275, 270).unwrap();
        resvg::render(&tree, Default::default(), &mut p.as_mut());
        let mask = Texture2D::from_rgba8(275, 270, p.data());
        mask.set_filter(FilterMode::Linear);
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            MaterialParams {
                textures: vec!["Mask".into()],
                uniforms: vec![
                    UniformDesc::new("Source", UniformType::Float4),
                    UniformDesc::new("Opacity", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    alpha_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::One,
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())?;
        material.set_texture("Mask", mask.clone());
        Ok(Self {
            state: Tutorial::new("keys"),
            route: String::new(),
            material,
            _mask: mask,
            captured: false,
            shown: false,
            was_hidden: false,
            image_origin: 0.,
        })
    }
    pub fn route(&mut self, route: &str, now: f64) {
        if self.route != route {
            self.route = route.into();
            self.state.for_scene(route, now);
            self.captured = false;
            self.shown = false;
            self.was_hidden = false;
        }
    }
    pub fn prepare(&mut self, a: &Assets, now: f64) {
        if !self.state.loaded {
            if let Some(name) = self.state.active.clone() {
                self.state
                    .asset_result(&name, a.has(&Self::key(&name)), now);
                self.shown = false;
                self.was_hidden = false;
            }
        }
    }
    fn key(name: &str) -> String {
        if name == "elfski.mp4" {
            "native/tutorial/elfski.png".into()
        } else if name.contains('.') {
            format!("img/tutorial/{name}")
        } else {
            format!("img/tutorial/input/{name}.svg")
        }
    }
    pub fn rect() -> Rect {
        Rect::new(
            screen_width() / 2. - 137.5,
            screen_height() / 2. - 137.,
            275.,
            270.,
        )
    }
    /// Parent tutorial button receives the whole click, rather than the game iframe.
    pub fn button_input(&mut self, now: f64, playing: bool) -> bool {
        let r = Self::rect();
        let m = Vec2::from(mouse_position());
        let inside =
            ((m.x - r.center().x) / 137.5).powi(2) + ((m.y - r.center().y) / 135.).powi(2) <= 1.;
        if playing
            && self.state.visible(now)
            && inside
            && is_mouse_button_pressed(MouseButton::Left)
        {
            self.captured = true;
        }
        let captured = self.captured;
        if self.captured && is_mouse_button_released(MouseButton::Left) {
            self.captured = false;
            if inside && playing && self.state.visible(now) {
                if let Some(name) = self.state.active.clone() {
                    self.state.dismiss(&[&name], now);
                    self.shown = false;
                }
            }
        }
        captured
    }
    pub fn controls(&mut self, scene: &str, now: f64, allowed: bool, mouse_blocked: bool) {
        if !allowed {
            return;
        }
        let any = |keys: &[KeyCode]| keys.iter().any(|k| is_key_pressed(*k));
        let mouse = !mouse_blocked
            && is_mouse_button_pressed(MouseButton::Left)
            && !crate::chrome::over_controls(Vec2::from(mouse_position()), 3);
        match scene {
            "glider" => {
                if any(&[KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down]) {
                    self.state.dismiss(&["keys-arrows", "spacenav-arrows"], now);
                }
                if any(&[KeyCode::Space]) {
                    self.state.dismiss(&["keys-space", "spacenav-space"], now);
                }
            }
            "runner" => {
                if any(&[KeyCode::Up, KeyCode::Down]) {
                    self.state.dismiss(&["keys-updown"], now);
                }
                if mouse {
                    self.state.dismiss(&["touch-updown"], now);
                }
            }
            "boatload" => {
                if any(&[KeyCode::Space, KeyCode::Right]) {
                    self.state.dismiss(&["keys-space"], now);
                }
                if any(&[KeyCode::Up, KeyCode::Down]) {
                    self.state.dismiss(&["keys-updown"], now);
                }
                if mouse {
                    self.state.dismiss(&["touch-updown"], now);
                }
            }
            "presentdrop" => {
                if any(&[KeyCode::Space, KeyCode::Down]) {
                    self.state.dismiss(&["keys-space", "spacenav-space"], now);
                }
                if any(&[KeyCode::Left, KeyCode::Right]) {
                    self.state
                        .dismiss(&["keys-leftright", "spacenav-leftright"], now);
                }
                if mouse {
                    self.state.dismiss(&["touch-leftright"], now);
                }
            }
            "jetpack" => {
                if any(&[KeyCode::Left, KeyCode::Right]) {
                    self.state
                        .dismiss(&["keys-leftright", "spacenav-leftright"], now);
                }
                if any(&[KeyCode::Up, KeyCode::Down]) {
                    self.state.dismiss(&["keys-updown", "spacenav-updown"], now);
                }
                if mouse {
                    self.state.dismiss(&["touch-updown"], now);
                }
            }
            "elfski" => {
                if mouse
                    || get_keys_pressed()
                        .iter()
                        .any(|k| !matches!(k, KeyCode::Escape | KeyCode::F12))
                {
                    self.state.dismiss(&["elfski.mp4"], now);
                }
            }
            _ => {}
        }
    }
    pub fn draw(&mut self, a: &Assets, now: f64, playing: bool) {
        let show = playing && self.state.visible(now);
        if !show {
            if self.shown {
                self.was_hidden = true;
            }
            return;
        }
        let Some(name) = self.state.active.as_ref() else {
            return;
        };
        let key = Self::key(name);
        if !a.has(&key) {
            return;
        }
        let video = name.ends_with(".mp4");
        if !self.shown {
            self.image_origin = now - self.state.animation_time(now);
            self.shown = true;
        } else if self.was_hidden && !video {
            self.image_origin = now;
        }
        self.was_hidden = false;
        let elapsed = if video {
            self.state.animation_time(now)
        } else {
            (now - self.image_origin).max(0.)
        };
        let texture = a.texture(&key);
        let source = if video {
            let frame = ((elapsed * 16.).floor() as usize) % 95;
            // Original H.264 has 54:65 sample aspect. Chromium reports
            // 1432x1300 display pixels although the coded frame is 1432x1080.
            let source_width = 275. * 65. / 54.;
            Rect::new(
                (frame % 10) as f32 * 358. + (358. - source_width) / 2.,
                (frame / 10) as f32 * 270.,
                source_width,
                270.,
            )
        } else {
            let natural_h = texture.height();
            let steps = (texture.width() / (natural_h / 270.) / 275.)
                .round()
                .max(1.) as usize;
            let frame = (elapsed.floor() as usize) % steps;
            Rect::new(
                frame as f32 * texture.width() / steps as f32,
                0.,
                texture.width() / steps as f32,
                natural_h,
            )
        };
        let r = Self::rect();
        self.material.set_uniform(
            "Source",
            vec4(
                source.x / texture.width(),
                source.y / texture.height(),
                source.w / texture.width(),
                source.h / texture.height(),
            ),
        );
        self.material.set_uniform("Opacity", 0.95f32);
        gl_use_material(&self.material);
        draw_texture_ex(
            texture,
            r.x,
            r.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    source.x,
                    source.y,
                    source.w * 279. / 275.,
                    source.h * 282. / 270.,
                )),
                dest_size: Some(vec2(279., 282.)),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
}
