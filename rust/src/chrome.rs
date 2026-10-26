//! Shared toolbar based on santa-chrome, santa-button and santa-badge.
use crate::ui_render::{Button, Shapes};
use macroquad::prelude::*;
use santa_tracker::ui::{Counter, Ease, Transition};
use std::collections::HashMap;

pub enum Action {
    Menu,
    Home,
    Mute,
    Pause,
    Restart,
}
#[derive(Clone, Copy)]
pub struct GameHud {
    pub playing: bool,
    pub paused: bool,
    pub score: i64,
    pub countdown: f64,
    pub level: Option<u32>,
    pub badge: bool,
    pub can_pause: bool,
}
fn toolbar_action(game: Option<GameHud>) -> Option<&'static str> {
    game.filter(|g| g.can_pause || !g.playing).map(|g| {
        if !g.playing {
            "restart"
        } else if g.paused {
            "play"
        } else {
            "pause"
        }
    })
}
pub struct Chrome {
    icons: HashMap<String, Texture2D>,
    font: Font,
    label_font: Font,
    countdown_font: Font,
    title: Font,
    buttons: [Button; 3],
    shapes: Shapes,
    end: Texture2D,
    counter: Counter,
    started: f64,
    epoch_ms: u64,
    fixed_clock: bool,
    focused: Option<usize>,
    keyboard_pressed: Option<usize>,
    action_reveal: Transition<1>,
    last_action: &'static str,
    last_level: Option<u32>,
    level_changed: f64,
    badge_alt: Transition<1>,
    badge_width: Transition<2>,
    pub sounds: Vec<&'static str>,
}
impl Chrome {
    pub fn new(clock_ms: Option<u64>) -> Result<Self, String> {
        let paths: HashMap<String, String> =
            serde_json::from_str(include_str!("../data/chrome-icons.json"))
                .map_err(|e| e.to_string())?;
        let mut icons = HashMap::new();
        for (name, path) in paths {
            let data = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="white" d="{path}"/></svg>"#
            );
            let tree = resvg::usvg::Tree::from_data(data.as_bytes(), &Default::default())
                .map_err(|e| e.to_string())?;
            let mut pixmap = resvg::tiny_skia::Pixmap::new(72, 72).ok_or("Icon allocation")?;
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::from_scale(3., 3.),
                &mut pixmap.as_mut(),
            );
            let mut pixels = pixmap.data().to_vec();
            for p in pixels.chunks_exact_mut(4) {
                if p[3] > 0 {
                    p[0] = 255;
                    p[1] = 255;
                    p[2] = 255;
                }
            }
            let texture = Texture2D::from_rgba8(72, 72, &pixels);
            texture.set_filter(FilterMode::Linear);
            icons.insert(name, texture);
        }
        let started = get_time();
        let epoch_ms = clock_ms.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });
        let deadline = deadline(epoch_ms / 1000) * 1000;
        let tree = resvg::usvg::Tree::from_data(
            include_bytes!("../data/countdown-end.svg"),
            &Default::default(),
        )
        .map_err(|e| e.to_string())?;
        let mut pix = resvg::tiny_skia::Pixmap::new(60, 240).ok_or("Counter shape allocation")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(4., 4.),
            &mut pix.as_mut(),
        );
        let mut pixels = pix.data().to_vec();
        for p in pixels.chunks_exact_mut(4) {
            p[0] = 255;
            p[1] = 255;
            p[2] = 255;
        }
        let end = Texture2D::from_rgba8(60, 240, &pixels);
        end.set_filter(FilterMode::Linear);
        Ok(Self {
            icons,
            font: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/GoogleSans-Medium.ttf"))
                .map_err(|e| e.to_string())?,
            label_font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/GoogleSans-Bold.ttf"
            ))
            .map_err(|e| e.to_string())?,
            countdown_font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/Roboto-SemiBold.ttf"
            ))
            .map_err(|e| e.to_string())?,
            title: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Lobster-Regular.ttf"))
                .map_err(|e| e.to_string())?,
            buttons: std::array::from_fn(|_| Button::new(Color::from_hex(0xff3333))),
            shapes: Shapes::new()?,
            end,
            counter: Counter::new(epoch_ms, deadline, started),
            started,
            epoch_ms,
            fixed_clock: clock_ms.is_some(),
            focused: None,
            keyboard_pressed: None,
            action_reveal: Transition::new([0.]),
            last_action: "pause",
            last_level: None,
            level_changed: -10.,
            badge_alt: Transition::new([0.]),
            badge_width: Transition::new([0., 0.]),
            sounds: vec![],
        })
    }
    pub fn icon(&self, key: &str) -> &Texture2D {
        &self.icons[key]
    }
    fn text(&self, text: &str, x: f32, y: f32, size: u16, title: bool) {
        let font = if title { &self.title } else { &self.font };
        let w = measure_text(text, Some(font), size, 1.).width;
        draw_text_ex(
            text,
            x - w / 2.,
            y,
            TextParams {
                font: Some(font),
                font_size: size,
                color: WHITE,
                ..Default::default()
            },
        );
    }
    fn label(&self, text: &str, x: f32, y: f32, size: u16, alpha: f32, spacing: f32) {
        let font = if size == 8 {
            &self.countdown_font
        } else if size == 9 {
            &self.label_font
        } else {
            &self.font
        };
        let widths: Vec<f32> = text
            .chars()
            .map(|c| measure_text(&c.to_string(), Some(font), size, 1.).width)
            .collect();
        let width = widths.iter().sum::<f32>() + spacing * (widths.len().saturating_sub(1)) as f32;
        let mut left = x - width / 2.;
        for (c, width) in text.chars().zip(widths) {
            draw_text_ex(
                &c.to_string(),
                left,
                y,
                TextParams {
                    font: Some(font),
                    font_size: size,
                    color: Color::new(1., 1., 1., alpha),
                    ..Default::default()
                },
            );
            left += width + spacing;
        }
    }
    fn counter_number(&self, value: u64, x: f32, top: f32, height: f32, size: u16, alpha: f32) {
        // CSS line boxes center the font's ascender/descender within line-height.
        let text = format!("{value:02}");
        let metrics = measure_text(&text, Some(&self.title), size, 1.);
        let baseline = top + (height - 8. - size as f32) / 2. + size as f32 * 0.82;
        let widths: Vec<f32> = text
            .chars()
            .map(|c| measure_text(&c.to_string(), Some(&self.title), size, 1.).width)
            .collect();
        let total = widths.iter().sum::<f32>() - widths.len().saturating_sub(1) as f32;
        let mut left = x - total / 2.;
        let _ = metrics;
        for (c, width) in text.chars().zip(widths) {
            draw_text_ex(
                &c.to_string(),
                left,
                baseline,
                TextParams {
                    font: Some(&self.title),
                    font_size: size,
                    color: Color::new(1., 1., 1., alpha),
                    ..Default::default()
                },
            );
            left += width - 1.;
        }
    }
    fn draw_counter(&mut self, now: f64, side: f32, y: f32) -> f32 {
        let elapsed = (now - self.started).max(0.);
        let clock_ms = if self.fixed_clock {
            self.epoch_ms + (elapsed * 1000.) as u64
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        };
        self.counter.update(now, clock_ms);
        let split = self.counter.current;
        if split.count == 0 {
            return 0.;
        }
        let desktop = screen_width() >= 768.;
        let height = if desktop { 60. } else { 52. };
        let visible = split.visible(screen_width() as f64);
        let values = split.values();
        let previous = self.counter.previous.values();
        let widths: Vec<(usize, f32)> = visible
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .map(|(i, _)| {
                (
                    i,
                    if i == 0 && split.days >= 100 {
                        if desktop {
                            52.
                        } else {
                            48.
                        }
                    } else if desktop {
                        44.
                    } else {
                        40.
                    },
                )
            })
            .collect();
        let inner =
            widths.iter().map(|(_, w)| *w).sum::<f32>() + (widths.len().saturating_sub(1)) as f32;
        let edge = height / 4.;
        let total = inner + 2. * (edge - 14.);
        let left = screen_width() - side - total;
        let top = y + 22. - height / 2.;
        let bar = Color::from_hex(0xff3333);
        let draw_bg = |offset: Vec2, c: Color| {
            draw_rectangle(
                left + edge - 14. + 6. + offset.x,
                top + offset.y,
                (inner - 12.).max(0.),
                height,
                c,
            );
            for (x, flip) in [(left - 7., false), (left + edge - 14. + inner - 7., true)] {
                draw_texture_ex(
                    &self.end,
                    x + offset.x,
                    top + offset.y,
                    c,
                    DrawTextureParams {
                        dest_size: Some(vec2(edge, height)),
                        flip_x: flip,
                        ..Default::default()
                    },
                );
            }
        };
        draw_bg(vec2(3., 8.), Color::new(0., 0., 0., 0.1));
        draw_bg(Vec2::ZERO, bar);
        let phase = self.counter.phase(now) as f32;
        let animating = now - self.counter.tick_at < 0.6;
        let mut x = left + edge - 14.;
        for (index, width) in widths {
            if x > left + edge - 14. + 0.5 {
                draw_rectangle(
                    x - 1.,
                    top + 7.,
                    1.,
                    height - 14.,
                    Color::new(0., 0., 0., 0.25),
                );
            }
            let center = x + width / 2.;
            let size = if desktop { 26 } else { 22 };
            if animating && self.counter.changed[index] {
                self.counter_number(
                    values[index],
                    center,
                    top + (1. - phase) * height / 2.,
                    height,
                    size,
                    phase,
                );
                self.counter_number(
                    previous[index],
                    center,
                    top - phase * height / 2.,
                    height,
                    size,
                    1. - phase,
                );
            } else {
                self.counter_number(values[index], center, top, height, size, 1.);
            }
            self.label(
                ["DAYS", "HRS", "MIN", "SEC"][index],
                center,
                top + height - 8.,
                8,
                0.8,
                1.,
            );
            x += width + 1.;
        }
        total
    }
    pub fn draw(&mut self, route: &str, game: Option<GameHud>, muted: bool) -> Option<Action> {
        let now = get_time();
        for y in 0..64 {
            draw_rectangle(
                0.,
                y as f32,
                screen_width(),
                1.,
                Color::new(0., 0., 0., 0.2 * (1. - y as f32 / 64.)),
            );
        }
        let (x, y) = toolbar_origin();
        let first = navigation_icon(route);
        let action = toolbar_action(game);
        if let Some(action) = action {
            self.last_action = action;
        }
        self.action_reveal
            .set(now, [f64::from(action.is_some())], 0.25, Ease::InOut);
        let enabled = if action.is_some() { 3 } else { 2 };
        if is_key_pressed(KeyCode::Tab) {
            self.focused = Some(match self.focused {
                Some(index)
                    if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) =>
                {
                    (index + enabled - 1) % enabled
                }
                Some(index) => (index + 1) % enabled,
                None => 0,
            });
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            self.focused = None;
        }
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
            self.keyboard_pressed = self.focused;
        }
        let released = is_key_released(KeyCode::Space) || is_key_released(KeyCode::Enter);
        let key_held = is_key_down(KeyCode::Space) || is_key_down(KeyCode::Enter);
        let names = [
            first,
            if muted { "unmute" } else { "mute" },
            self.last_action,
        ];
        let mut result = None;
        for (i, name) in names.iter().enumerate() {
            let rect = toolbar_rect(i);
            let usable = i < enabled;
            let interact = self.buttons[i].update(
                rect,
                now,
                usable,
                self.keyboard_pressed == Some(i) && key_held,
                self.keyboard_pressed == Some(i) && released,
            );
            if interact.entered {
                self.sounds.push("generic_button_over");
            }
            if interact.pressed
                || self.keyboard_pressed == Some(i)
                    && (is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter))
            {
                self.sounds.push("generic_button_click");
            }
            self.buttons[i].theme(
                if muted && i == 1 {
                    Color::from_hex(0xad00ad)
                } else {
                    Color::from_hex(0xff3333)
                },
                now,
            );
            let visibility = if i == 2 {
                self.action_reveal.value(now)[0] as f32
            } else {
                1.
            };
            if visibility > 0. {
                // Header button disabled state translates up 100px and fades.
                let mut draw_rect = rect;
                draw_rect.y -= 100. * (1. - visibility);
                let delta = self.buttons[i].draw(
                    &self.shapes,
                    draw_rect,
                    now,
                    self.focused == Some(i),
                    visibility,
                );
                draw_texture_ex(
                    &self.icons[*name],
                    draw_rect.x + 10. + delta.x,
                    draw_rect.y + 10. + delta.y,
                    Color::new(1., 1., 1., visibility),
                    DrawTextureParams {
                        dest_size: Some(vec2(24., 24.)),
                        ..Default::default()
                    },
                );
            }
            if interact.clicked {
                result = Some(match *name {
                    "home" => Action::Home,
                    "menu" => Action::Menu,
                    "mute" | "unmute" => Action::Mute,
                    "restart" => Action::Restart,
                    _ => Action::Pause,
                });
            }
        }
        if released {
            self.keyboard_pressed = None;
        }
        let counter_width = self.draw_counter(now, x, y);
        if let Some(game) = game.filter(|g| g.badge) {
            if game.level != self.last_level {
                self.last_level = game.level;
                self.level_changed = now;
            }
            let alt = game.level.is_some() && (now - self.level_changed < 2. || game.score <= 0);
            self.badge_alt
                .set(now, [f64::from(alt)], 0.4, Ease::Default);
            let time = game.countdown.clamp(0., 599.).floor() as u32;
            let show_right = game.level.is_some() || game.score != 0;
            self.badge_width.set(
                now,
                [
                    if time > 0 {
                        if show_right {
                            57.
                        } else {
                            56.
                        }
                    } else {
                        0.
                    },
                    if show_right { 56. } else { 0. },
                ],
                0.5,
                Ease::InOut,
            );
            let widths = self.badge_width.value(now);
            let width = (widths[0] + widths[1]) as f32;
            // The original flex toolbar gives each grow item half the remaining
            // space after all three buttons (including the disabled action).
            let group = (screen_width() - 2. * x - 3. * 56. - counter_width - width) / 2.;
            let left = x + 3. * 56. + group;
            let top = y + 2.;
            self.shapes.rounded(
                Rect::new(left + 3., top + 8., width, 40.),
                4.,
                Color::new(0., 0., 0., 0.1),
                0.,
            );
            self.shapes.rounded(
                Rect::new(left, top, width, 40.),
                4.,
                Color::from_hex(0x6f00ff),
                0.,
            );
            if widths[0] > 0.1 {
                self.label(
                    &format!("{}:{:02}", time / 60, time % 60),
                    left + widths[0] as f32 / 2.,
                    top + 21.,
                    17,
                    1.,
                    -1.,
                );
                self.text("TIME", left + widths[0] as f32 / 2., top + 34., 9, false);
            }
            if show_right && widths[0] > 0.1 && widths[1] > 0.1 {
                draw_rectangle(
                    left + widths[0] as f32 - 1.,
                    top + 6.,
                    1.,
                    28.,
                    Color::new(0., 0., 0., 0.25),
                );
            }
            if widths[1] > 0.1 {
                let middle = left + widths[0] as f32 + 28.;
                let t = self.badge_alt.value(now)[0] as f32;
                let score = if game.score < 10000 {
                    game.score.to_string()
                } else if game.score < 1000000 {
                    format!("{}k", game.score / 1000)
                } else if game.score < 1000000000 {
                    format!("{}m", game.score / 1000000)
                } else {
                    "?".into()
                };
                // Clip the two sliding labels to the original .item box.
                unsafe {
                    get_internal_gl().quad_gl.scissor(Some((
                        (left + widths[0] as f32) as i32,
                        top as i32,
                        widths[1] as i32,
                        40,
                    )));
                }
                self.label(&score, middle + 56. * t, top + 21., 17, 1. - t, -1.);
                self.label("SCORE", middle + 56. * t, top + 34., 9, 1. - t, 0.);
                if let Some(level) = game.level {
                    self.label(
                        &format!(
                            "{level}·{}",
                            match route {
                                "codelab" => 14,
                                "codeboogie" => 9,
                                _ => 10,
                            }
                        ),
                        middle,
                        top + 21. - 40. * (1. - t),
                        17,
                        t,
                        -1.,
                    );
                    self.label("LEVEL", middle, top + 34. - 40. * (1. - t), 9, t, 0.);
                }
                unsafe {
                    get_internal_gl().quad_gl.scissor(None);
                }
            }
        }
        result
    }
}

// entrypoint.js sets showHome from the route, independently of score/action.
// The native village route corresponds to the original empty top-level route.
fn navigation_icon(route: &str) -> &'static str {
    if route == "modvil" {
        "menu"
    } else {
        "home"
    }
}

fn leap(year: u64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
pub fn deadline(now: u64) -> u64 {
    let mut year = 1970;
    let mut start = 0;
    while start + (if leap(year) { 366 } else { 365 }) * 86400 <= now {
        start += (if leap(year) { 366 } else { 365 }) * 86400;
        year += 1;
    }
    let day = 334 + u64::from(leap(year)) + 23;
    start + day * 86400 + 10 * 3600
}
#[cfg(test)]
pub fn countdown(now: u64) -> u64 {
    deadline(now).saturating_sub(now)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coding_toolbar_action_matches_original_host_without_pause() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/coding-host-reference.json")).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            for row in case["actions"].as_array().unwrap() {
                let gameover = row["state"]["gameover"].as_bool().unwrap();
                let hud = GameHud {
                    playing: !gameover,
                    paused: false,
                    score: 0,
                    countdown: 0.,
                    level: None,
                    badge: true,
                    can_pause: false,
                };
                assert_eq!(
                    toolbar_action(Some(hud)),
                    row["host"]["action"].as_str(),
                    "{} {}",
                    case["scene"],
                    row["action"]
                );
            }
        }
    }
    #[test]
    fn activity_navigation_is_independent_of_hud() {
        assert_eq!(navigation_icon("modvil"), "menu");
        for route in [
            "penguindash",
            "jamband",
            "elfmaker",
            "santaselfie",
            "runner",
            "jetpack",
            "elfski",
            "presentdrop",
            "boatload",
            "glider",
        ] {
            assert_eq!(navigation_icon(route), "home", "{route}");
        }
    }
    #[test]
    fn countdown_targets_original_utc_takeoff() {
        assert_eq!(countdown(1798106400), 0);
        assert_eq!(countdown(1798106399), 1);
    }
}

/// Hit area shared with the native scenes so the toolbar does not consume
/// the entire top band of the game (Santa's scalp is interactive there).
pub fn toolbar_origin() -> (f32, f32) {
    if screen_width() < 768. {
        (10., 10.)
    } else {
        (screen_width() * 0.05, 20.)
    }
}
pub fn toolbar_rect(index: usize) -> Rect {
    let (x, y) = toolbar_origin();
    let spacing = if screen_width() <= 400. { 48. } else { 56. };
    Rect::new(
        x + index as f32 * spacing + if screen_width() <= 400. { 0. } else { 2. },
        y,
        44.,
        44.,
    )
}
pub fn over_controls(mouse: Vec2, count: usize) -> bool {
    (0..count).any(|i| toolbar_rect(i).contains(mouse))
}
