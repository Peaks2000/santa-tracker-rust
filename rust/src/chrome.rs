//! Shared toolbar based on santa-chrome, santa-button and santa-badge.
use macroquad::prelude::*;
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
pub struct Chrome {
    icons: HashMap<String, Texture2D>,
    font: Font,
    title: Font,
    pressed: Option<usize>,
}
impl Chrome {
    pub fn new() -> Result<Self, String> {
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
        Ok(Self {
            icons,
            font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/GoogleSans-Regular.ttf"
            ))
            .map_err(|e| e.to_string())?,
            title: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Lobster-Regular.ttf"))
                .map_err(|e| e.to_string())?,
            pressed: None,
        })
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
    pub fn draw(&mut self, game: Option<GameHud>, muted: bool) -> Option<Action> {
        for y in 0..64 {
            draw_rectangle(
                0.,
                y as f32,
                screen_width(),
                1.,
                Color::new(0., 0., 0., 0.2 * (1. - y as f32 / 64.)),
            );
        }
        let small = screen_width() < 768.;
        let x = if small { 10. } else { screen_width() * 0.05 };
        let y = if small { 10. } else { 20. };
        let first = if game.is_some() { "home" } else { "menu" };
        let state = game.filter(|g| g.can_pause).map(|g| {
            if !g.playing {
                "restart"
            } else if g.paused {
                "play"
            } else {
                "pause"
            }
        });
        let mut result = None;
        let mouse = Vec2::from(mouse_position());
        for (i, name) in [
            Some(first),
            Some(if muted { "unmute" } else { "mute" }),
            state,
        ]
        .into_iter()
        .enumerate()
        {
            let Some(name) = name else {
                continue;
            };
            let bx = x + i as f32 * 56. + 2.;
            let by = y + 2.;
            let rect = Rect::new(bx, by, 44., 44.);
            let hover = rect.contains(mouse);
            if hover && is_mouse_button_pressed(MouseButton::Left) {
                self.pressed = Some(i);
            }
            let down = hover && self.pressed == Some(i) && is_mouse_button_down(MouseButton::Left);
            let delta = if down { vec2(0., 0.) } else { vec2(-3., -5.) };
            let c = if muted && i == 1 {
                Color::from_hex(0xad00ad)
            } else {
                Color::from_hex(0xff3333)
            };
            if !down {
                draw_poly(
                    bx + 27.,
                    by + 24.,
                    96,
                    24.,
                    0.,
                    Color::new(0., 0., 0., 0.24),
                );
                draw_poly(
                    bx + 22.,
                    by + 22.,
                    96,
                    22.,
                    0.,
                    Color::new(c.r * 0.7, c.g * 0.7, c.b * 0.7, 1.),
                );
            }
            draw_poly(bx + 22. + delta.x, by + 22. + delta.y, 96, 22., 0., c);
            draw_texture_ex(
                &self.icons[name],
                bx + 10. + delta.x,
                by + 10. + delta.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(24., 24.)),
                    ..Default::default()
                },
            );
            if hover && is_mouse_button_released(MouseButton::Left) && self.pressed == Some(i) {
                result = Some(match name {
                    "home" => Action::Home,
                    "menu" => Action::Menu,
                    "mute" | "unmute" => Action::Mute,
                    "restart" => Action::Restart,
                    _ => Action::Pause,
                });
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            self.pressed = None;
        }
        if let Some(game) = game.filter(|g| g.badge) {
            let cells = if game.score > 0 || game.level.is_some() {
                2
            } else {
                1
            };
            let width = 56. * cells as f32;
            let left = screen_width() / 2. - width / 2.;
            draw_rectangle(left + 3., y + 10., width, 40., Color::new(0., 0., 0., 0.1));
            draw_rectangle(left, y + 2., width, 40., Color::from_hex(0x6f00ff));
            let seconds = game.countdown.ceil().max(0.) as u32;
            self.text(
                &format!("{}:{:02}", seconds / 60, seconds % 60),
                left + 28.,
                y + 23.,
                17,
                false,
            );
            self.text("TIME", left + 28., y + 36., 9, false);
            if cells == 2 {
                draw_rectangle(left + 56., y + 8., 1., 28., Color::new(0., 0., 0., 0.25));
                let score = if game.score < 10000 {
                    game.score.to_string()
                } else if game.score < 1000000 {
                    format!("{}k", game.score / 1000)
                } else {
                    format!("{}m", game.score / 1000000)
                };
                if game.score <= 0 && game.level.is_some() {
                    self.text(
                        &format!("{}·10", game.level.unwrap()),
                        left + 84.,
                        y + 23.,
                        17,
                        false,
                    );
                    self.text("LEVEL", left + 84., y + 36., 9, false);
                } else {
                    self.text(&score, left + 84., y + 23., 17, false);
                    self.text("SCORE", left + 84., y + 36., 9, false);
                }
            }
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let remaining = countdown(now);
        if remaining > 0 && screen_width() > 500. {
            let days = remaining / 86400;
            let values = [
                days,
                (remaining / 3600) % 24,
                (remaining / 60) % 60,
                remaining % 60,
            ];
            let cw = if days >= 100 { 48. } else { 40. };
            let w = cw + 120.;
            let left = screen_width() - x - w;
            draw_rectangle(left + 3., y, w, 60., Color::new(0., 0., 0., 0.1));
            draw_rectangle(left, y - 8., w, 60., Color::from_hex(0xff3333));
            for (i, value) in values.iter().enumerate() {
                let offset = if i == 0 {
                    0.
                } else {
                    cw + (i - 1) as f32 * 40.
                };
                let width = if i == 0 { cw } else { 40. };
                let mid = left + offset + width / 2.;
                self.text(&format!("{value:02}"), mid, y + 23., 22, true);
                self.text(["DAYS", "HRS", "MIN", "SEC"][i], mid, y + 41., 8, false);
                if i > 0 {
                    draw_rectangle(left + offset, y, 1., 44., Color::new(0., 0., 0., 0.25));
                }
            }
        }
        result
    }
}
fn leap(year: u64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
pub fn countdown(now: u64) -> u64 {
    let mut year = 1970;
    let mut start = 0;
    while start + (if leap(year) { 366 } else { 365 }) * 86400 <= now {
        start += (if leap(year) { 366 } else { 365 }) * 86400;
        year += 1;
    }
    let day = 334 + u64::from(leap(year)) + 23;
    (start + day * 86400 + 10 * 3600).saturating_sub(now)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn countdown_targets_original_utc_takeoff() {
        assert_eq!(countdown(1798106400), 0);
        assert_eq!(countdown(1798106399), 1);
    }
}

/// Hit area shared with the native scenes so the toolbar does not consume
/// the entire top band of the game (Santa's scalp is interactive there).
pub fn over_controls(mouse: Vec2, count: usize) -> bool {
    let small = screen_width() < 768.;
    let x = if small { 10. } else { screen_width() * 0.05 };
    let y = if small { 10. } else { 20. };
    (0..count).any(|i| Rect::new(x + i as f32 * 56. + 2., y + 2., 44., 44.).contains(mouse))
}
