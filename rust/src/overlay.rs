//! santa-overlay.js/scss and santa-gameloader's disabled scene filter.
use crate::{
    assets::Assets,
    chrome::Chrome,
    ui_render::{Button, Shapes},
    village::Village,
};
use macroquad::prelude::*;
use santa_tracker::ui::{Ease, Transition};

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
    below_opacity: Transition<1>,
    recommendations: [String; 2],
    scene: String,
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
            below_opacity: Transition::new([0.]),
            recommendations: ["santaselfie".into(), "jetpack".into()],
            scene: String::new(),
            focus: 0,
            title: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Lobster-Regular.ttf"))
                .map_err(|e| e.to_string())?,
            sounds: vec![],
        })
    }
    pub fn set_recommendations(&mut self, cards: [String; 2]) {
        self.recommendations = cards;
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
        self.draw_at(assets, chrome, village, scene, paused, gameover, get_time())
    }
    pub fn draw_at(
        &mut self,
        assets: &mut Assets,
        chrome: &Chrome,
        village: &mut Village,
        scene: &str,
        paused: bool,
        gameover: bool,
        now: f64,
    ) -> Option<Action> {
        let visible = paused || gameover;
        self.scene = scene.into();
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
            self.below_opacity
                .set(now, [f64::from(visible)], 0.4, Ease::Default);
        }
        let opacity = self.opacity.value(now)[0] as f32;
        if opacity < 0.001 {
            return None;
        }
        let h = screen_height();
        let w = screen_width();
        let layout = santa_tracker::ui::overlay_layout(w, h);
        let shift = self.offset.value(now)[0] as f32 * h;
        let r = layout.hero;
        let hero = Rect::new(r[0], r[1] + shift, r[2], r[3]);
        let center = hero.x + hero.w / 2.;
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
            let text = "Game Over!";
            let widths: Vec<f32> = text
                .chars()
                .map(|c| measure_text(&c.to_string(), Some(&self.title), 42, 1.).width)
                .collect();
            let width = widths.iter().sum::<f32>() - 2. * (widths.len() - 1) as f32;
            let mut x = center - width / 2.;
            for (c, cw) in text.chars().zip(widths) {
                for (dx, dy, color) in [
                    (1., 2., Color::new(0., 0., 0., 0.13 * opacity)),
                    (-0.25, 0., Color::new(1., 1., 1., opacity)),
                    (0.25, 0., Color::new(1., 1., 1., opacity)),
                ] {
                    draw_text_ex(
                        &c.to_string(),
                        x + dx,
                        hero.y + 36.75 + dy,
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
        let r = layout.nav;
        let nav = Rect::new(r[0], r[1] + shift, r[2], r[3]);
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
                center - count as f32 * 28. + i as f32 * 56. + 6.,
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
        let delta = self.below.value(now)[0] as f32;
        let rects = layout.cards.map(|r| {
            Rect::new(
                r[0] + if layout.row {
                    delta * layout.below[2]
                } else {
                    0.
                },
                r[1] + if layout.row {
                    0.
                } else {
                    delta * layout.below[3]
                },
                r[2],
                r[3],
            )
        });
        let card_alpha = opacity * self.below_opacity.value(now)[0] as f32;
        if let Some(route) = village.overlay_cards(assets, &self.recommendations, rects, card_alpha)
        {
            if visible {
                result = Some(Action::Route(route));
            }
        }
        result
    }
}

pub use crate::scene_filter::SceneFilter;
