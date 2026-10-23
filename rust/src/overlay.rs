//! santa-overlay.js/scss and santa-gameloader's disabled scene filter.
use crate::{
    assets::Assets,
    chrome::Chrome,
    ui_render::{Button, Shapes},
    village::Village,
};
use macroquad::prelude::*;
use santa_tracker::{
    random::Random,
    ui::{Ease, Transition},
};

pub enum Action {
    Resume,
    Restart,
    Home,
    Route(String),
}
pub struct Overlay {
    shapes: Shapes,
    buttons: [Button; 3],
    visible: bool,
    opacity: Transition<1>,
    offset: Transition<1>,
    below: Transition<1>,
    recommendations: [String; 2],
    scene: String,
    random: Random,
    focus: usize,
    title: Font,
    pub sounds: Vec<&'static str>,
}
impl Overlay {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            shapes: Shapes::new()?,
            buttons: std::array::from_fn(|i| {
                Button::new(Color::from_hex(if i < 2 { 0xad00ad } else { 0xff3333 }))
            }),
            visible: false,
            opacity: Transition::new([0.]),
            offset: Transition::new([-1.]),
            below: Transition::new([0.5]),
            recommendations: ["santaselfie".into(), "jetpack".into()],
            scene: String::new(),
            random: Random::seeded("play-next"),
            focus: 0,
            title: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Lobster-Regular.ttf"))
                .map_err(|e| e.to_string())?,
            sounds: vec![],
        })
    }
    pub fn draw(
        &mut self,
        assets: &mut Assets,
        chrome: &Chrome,
        village: &mut Village,
        scene: &str,
        paused: bool,
        gameover: bool,
    ) -> Option<Action> {
        let now = get_time();
        let visible = paused || gameover;
        if self.scene != scene {
            self.scene = scene.into();
            let nav: serde_json::Value =
                serde_json::from_str(include_str!("../data/navigation.json")).unwrap();
            let mut choices: Vec<String> = nav["cards"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str())
                .filter(|id| *id != scene)
                .map(str::to_owned)
                .collect();
            for slot in &mut self.recommendations {
                let i = ((self.random.next().powi(4) * choices.len() as f64) as usize)
                    .min(choices.len() - 1);
                *slot = choices.remove(i);
            }
        }
        if visible != self.visible {
            self.visible = visible;
            self.focus = 0;
            self.opacity.set(
                now,
                [f64::from(visible)],
                if visible { 0.4 } else { 0.6 },
                Ease::InOut,
            );
            self.offset.set(
                now,
                [if visible { 0. } else { -1. }],
                0.4,
                if visible {
                    Ease::OverlayIn
                } else {
                    Ease::OverlayOut
                },
            );
            self.below
                .set(now, [if visible { 0. } else { 0.5 }], 0.4, Ease::Default);
        }
        let opacity = self.opacity.value(now)[0] as f32;
        if opacity < 0.001 {
            return None;
        }
        let h = screen_height();
        let w = screen_width();
        let compact = w <= 400. || h <= 400.;
        let vertical = w <= 500.;
        let hero_h = if h >= 500. { 218. } else { 136. };
        let (cw, ch, margin) = if compact {
            (160., 100., 4.)
        } else {
            (200., 120., 8.)
        };
        let below_h = if vertical {
            2. * (ch + 2. * margin) + 32.
        } else {
            ch + 2. * margin + 32.
        };
        let main_h = hero_h + 72.;
        let top = (h - main_h - below_h) / 2.;
        let shift = self.offset.value(now)[0] as f32 * h;
        let hero = Rect::new(w / 2. - 120., top + shift, 240., hero_h);
        let file = if paused { "pause" } else { "gameover" };
        let key = format!("img/overlay/{file}.svg");
        if let Err(error) = assets.load(&key) {
            eprintln!("Overlay art: {error}");
            return None;
        }
        let texture = assets.texture(&key);
        let scale = (hero.w / texture.width()).min(hero.h / texture.height());
        let tw = texture.width() * scale;
        let th = texture.height() * scale;
        draw_texture_ex(
            texture,
            hero.x + (hero.w - tw) / 2.,
            hero.bottom() - th,
            Color::new(1., 1., 1., opacity),
            DrawTextureParams {
                dest_size: Some(vec2(tw, th)),
                ..Default::default()
            },
        );
        if gameover {
            let text = "Game Over";
            let widths: Vec<f32> = text
                .chars()
                .map(|c| measure_text(&c.to_string(), Some(&self.title), 42, 1.).width)
                .collect();
            let width = widths.iter().sum::<f32>() - 2. * (widths.len() - 1) as f32;
            let mut x = (w - width) / 2.;
            for (c, cw) in text.chars().zip(widths) {
                for (dx, dy, color) in [
                    (1., 2., Color::new(0., 0., 0., 0.13 * opacity)),
                    (-0.25, 0., Color::new(1., 1., 1., opacity)),
                    (0.25, 0., Color::new(1., 1., 1., opacity)),
                ] {
                    draw_text_ex(
                        &c.to_string(),
                        x + dx,
                        hero.y + 38. + dy,
                        TextParams {
                            font: Some(&self.title),
                            font_size: 42,
                            color,
                            ..Default::default()
                        },
                    );
                }
                x += cw - 2.;
            }
        }
        let nav = Rect::new(w / 2. - 120., top + hero_h + shift, 240., 72.);
        self.shapes
            .rounded(nav, 12., Color::new(1., 1., 1., 0.25 * opacity), 0.);
        let mut result = None;
        let count = if paused { 3 } else { 2 };
        let names = if paused {
            vec!["play", "restart", "home"]
        } else {
            vec!["restart", "home"]
        };
        if visible && is_key_pressed(KeyCode::Tab) {
            self.focus = (self.focus + 1) % count;
        }
        for (i, name) in names.iter().enumerate() {
            let rect = Rect::new(
                w / 2. - count as f32 * 28. + i as f32 * 56. + 6.,
                nav.y + 14.,
                44.,
                44.,
            );
            let keyboard = self.focus == i;
            let event = self.buttons[i].update(
                rect,
                now,
                visible,
                keyboard && (is_key_down(KeyCode::Space) || is_key_down(KeyCode::Enter)),
                keyboard && (is_key_released(KeyCode::Space) || is_key_released(KeyCode::Enter)),
            );
            if event.entered {
                self.sounds.push("generic_button_over");
            }
            if event.pressed {
                self.sounds.push("generic_button_click");
            }
            self.buttons[i].theme(
                Color::from_hex(if *name == "home" { 0xff3333 } else { 0xad00ad }),
                now,
            );
            let delta = self.buttons[i].draw(&self.shapes, rect, now, false, opacity);
            draw_texture_ex(
                chrome.icon(name),
                rect.x + 10. + delta.x,
                rect.y + 10. + delta.y,
                Color::new(1., 1., 1., opacity),
                DrawTextureParams {
                    dest_size: Some(vec2(24., 24.)),
                    ..Default::default()
                },
            );
            if event.clicked {
                result = Some(match *name {
                    "play" => Action::Resume,
                    "restart" => Action::Restart,
                    _ => Action::Home,
                });
            }
        }
        let below_top = top + main_h + 16. + self.below.value(now)[0] as f32 * below_h;
        let rects = std::array::from_fn(|i| {
            if vertical {
                Rect::new(
                    (w - cw) / 2.,
                    below_top + margin + i as f32 * (ch + 2. * margin),
                    cw,
                    ch,
                )
            } else {
                Rect::new(
                    w / 2. - (cw + margin) + i as f32 * (cw + 2. * margin),
                    below_top + margin,
                    cw,
                    ch,
                )
            }
        });
        if let Some(route) = village.overlay_cards(assets, &self.recommendations, rects, opacity) {
            if visible {
                result = Some(Action::Route(route));
            }
        }
        result
    }
}

const VERTEX: &str = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;uniform mat4 Model;uniform mat4 Projection;varying vec2 uv;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;}"#;
const FRAGMENT: &str = r#"#version 100
precision highp float;varying vec2 uv;uniform sampler2D Texture;uniform vec2 Texel;uniform float Amount;
void main(){vec3 sum=vec3(0.0);float total=0.0;for(int x=-2;x<=2;x++){for(int y=-2;y<=2;y++){float weight=exp(-float(x*x+y*y)*0.5);sum+=texture2D(Texture,uv+vec2(float(x),float(y))*Texel*4.0*Amount).rgb*weight;total+=weight;}}vec3 c=sum/total;float gray=dot(c,vec3(0.2126,0.7152,0.0722));c=mix(c,vec3(gray),0.7*Amount)*(1.0-0.5*Amount);gl_FragColor=vec4(c,1.0);}"#;
pub struct SceneFilter {
    target: Option<RenderTarget>,
    filtered: Option<RenderTarget>,
    material: Material,
    amount: Transition<1>,
    cached: f32,
}
impl SceneFilter {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            target: None,
            filtered: None,
            material: load_material(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: FRAGMENT,
                },
                MaterialParams {
                    uniforms: vec![
                        UniformDesc::new("Texel", UniformType::Float2),
                        UniformDesc::new("Amount", UniformType::Float1),
                    ],
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?,
            amount: Transition::new([0.]),
            cached: -1.,
        })
    }
    pub fn begin(&mut self) {
        let w = screen_width() as u32;
        let h = screen_height() as u32;
        if self
            .target
            .as_ref()
            .is_none_or(|t| t.texture.width() != w as f32 || t.texture.height() != h as f32)
        {
            self.target = Some(render_target(w, h));
            self.filtered = Some(render_target(w, h));
            self.cached = -1.;
        }
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., w as f32, h as f32));
        camera.render_target = self.target.clone();
        set_camera(&camera);
        clear_background(BLACK);
    }
    pub fn finish(&mut self, disabled: bool) {
        let now = get_time();
        self.amount.set(
            now,
            [f64::from(disabled)],
            if disabled { 0.75 } else { 0.33 },
            Ease::Default,
        );
        let amount = self.amount.value(now)[0] as f32;
        let source = &self.target.as_ref().unwrap().texture;
        let filtered = self.filtered.as_ref().unwrap();
        if amount > 0.001 && (!disabled || amount < 1. || self.cached != amount) {
            let mut camera =
                Camera2D::from_display_rect(Rect::new(0., 0., screen_width(), screen_height()));
            camera.render_target = Some(filtered.clone());
            set_camera(&camera);
            self.material
                .set_uniform("Texel", vec2(1. / screen_width(), 1. / screen_height()));
            self.material.set_uniform("Amount", amount);
            gl_use_material(&self.material);
            draw_texture_ex(
                source,
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    flip_y: true,
                    ..Default::default()
                },
            );
            gl_use_default_material();
            self.cached = amount;
        }
        set_default_camera();
        let scale = 1. + amount * 0.05;
        let size = vec2(screen_width(), screen_height()) * scale;
        draw_texture_ex(
            if amount > 0.001 {
                &filtered.texture
            } else {
                source
            },
            (screen_width() - size.x) / 2.,
            (screen_height() - size.y) / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                flip_y: true,
                ..Default::default()
            },
        );
    }
}
