//! Santa Selfie's original point/constraint beard simulation, in Rust.
use crate::random::Random;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Tool {
    Hand,
    Clipper,
    Dryer,
    Grow,
    Clean,
    Spray(u8),
    Decoration(u8),
}
#[derive(Clone, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub px: f64,
    pub py: f64,
    pub pinned: bool,
    pub rotation: f64,
    pub draw: bool,
    pub constrain: bool,
    pub constraints: Vec<usize>,
    pub ox: f64,
    pub oy: f64,
    pub original_pinned: bool,
    pub original_draw: bool,
    pub spray: Option<u8>,
    pub decoration: Option<u8>,
}
#[derive(Clone, Copy, Default)]
pub struct Mouse {
    pub x: f64,
    pub y: f64,
    pub px: f64,
    pub py: f64,
    pub down: bool,
    pub scale: f64,
}
pub struct Selfie {
    pub points: Vec<Point>,
    pub tool: Tool,
    pub mouse: Mouse,
    pub paused: bool,
    pub time: f64,
    pub selected_at: f64,
    pub sounds: Vec<&'static str>,
    pub near_beard: bool,
    pub can_decorate: bool,
    pub blink_remaining: f64,
    next_blink: f64,
    random: Random,
    sound_active: [bool; 6],
}
impl Selfie {
    pub fn new(seed: &str) -> Self {
        let mut out = Self {
            points: vec![],
            tool: Tool::Hand,
            mouse: Mouse {
                scale: 1.,
                ..Default::default()
            },
            paused: false,
            time: 0.,
            selected_at: 0.,
            sounds: vec!["selfie_start"],
            near_beard: false,
            can_decorate: true,
            blink_remaining: 0.,
            next_blink: 1.,
            random: Random::seeded(seed),
            sound_active: [false; 6],
        };
        out.reset();
        out.next_blink = (out.random.next() * 8.).max(1.);
        out
    }
    fn point(&mut self, x: f64, y: f64, pinned: bool, draw: bool, constraints: Vec<usize>) {
        self.points.push(Point {
            x,
            y,
            px: x,
            py: y,
            pinned,
            rotation: self.random.next() * std::f64::consts::TAU,
            draw,
            constrain: true,
            constraints,
            ox: x,
            oy: y,
            original_pinned: pinned,
            original_draw: draw,
            spray: None,
            decoration: None,
        });
    }
    pub fn reset(&mut self) {
        self.points.clear();
        for y in 0..=25 {
            for x in 0..=22 {
                let mut constraints = vec![];
                if x > 0 {
                    constraints.push(self.points.len() - 1);
                }
                if y > 0 {
                    constraints.push(x + (y - 1) * 23);
                }
                let px = 380. + x as f64 * 20.;
                let py = 670. + y as f64 * 20.;
                let pinned = ((px - 600.).powi(2) + (py - 700.).powi(2)).sqrt() < 240.;
                let draw = if y > 14 {
                    (x as f64 - 11.).powi(2) + (y as f64 - 14.).powi(2) < 121.
                } else {
                    true
                };
                self.point(px, py, pinned, draw, constraints);
            }
        }
        for y in 1..5 {
            for x in 0..2 {
                self.point(
                    380. + x as f64 * 20.,
                    670. - y as f64 * 20.,
                    true,
                    true,
                    vec![],
                );
                self.point(
                    380. + (x + 21) as f64 * 20.,
                    670. - y as f64 * 20.,
                    true,
                    true,
                    vec![],
                );
            }
        }
        for (start, end, y) in [(3, 11, 3), (3, 12, 2), (4, 12, 1), (5, 11, 0)] {
            for x in start..end {
                self.point(
                    440. + x as f64 * 20.,
                    505. - y as f64 * 20.,
                    true,
                    true,
                    vec![],
                );
            }
        }
    }
    pub fn select(&mut self, tool: Tool) {
        self.selected_at = self.time;
        if self.tool == Tool::Clipper && tool != Tool::Clipper {
            self.sounds.push("selfie_happy");
        }
        self.tool = if tool == self.tool { Tool::Hand } else { tool };
        self.sounds.push("selfie_click");
        if self.tool == Tool::Clipper {
            self.sounds.push("selfie_concerned");
        }
    }
    pub fn pointer(&mut self, x: f64, y: f64, down: bool, scale: f64, released: bool) {
        self.mouse = Mouse {
            x,
            y,
            px: self.mouse.x,
            py: self.mouse.y,
            down,
            scale,
        };
        if !self.can_decorate {
            self.can_decorate = !down;
        }
        if down {
            self.near_beard = false;
            for p in self.points.iter_mut().rev() {
                if !p.draw {
                    continue;
                }
                let distance = (p.ox * scale - x).hypot(p.oy * scale - y);
                if distance < 20. * scale {
                    match self.tool {
                        Tool::Clipper => self.near_beard = true,
                        Tool::Spray(i) => p.spray = Some(i),
                        Tool::Clean => {
                            p.spray = None;
                            p.decoration = None;
                        }
                        Tool::Decoration(i) if self.can_decorate => {
                            p.decoration = Some(i);
                            self.can_decorate = false;
                            self.sounds.push("selfie_item");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        if released && self.tool == Tool::Grow {
            self.grow();
        }
    }
    pub fn update_tool_sounds(&mut self, scene_x: f64, down: bool) {
        let near = down && scene_x > 230.;
        let active = [
            near && self.tool == Tool::Dryer,
            near && self.tool == Tool::Clipper,
            near && matches!(self.tool, Tool::Spray(_)),
            down && self.near_beard && self.tool == Tool::Clipper,
            near && self.tool == Tool::Grow,
            near && self.tool == Tool::Clean,
        ];
        let starts = [
            "selfie_dryer_start",
            "selfie_shave_start",
            "selfie_color_start",
            "selfie_shave_cutting_start",
            "selfie_spray_small",
            "selfie_spray_big",
        ];
        let stops = [
            "selfie_dryer_stop",
            "selfie_shave_stop",
            "selfie_color_stop",
            "selfie_shave_cutting_stop",
        ];
        for i in 0..active.len() {
            if active[i] && !self.sound_active[i] {
                self.sounds.push(starts[i]);
            }
            if !active[i] && self.sound_active[i] && i < 4 {
                self.sounds.push(stops[i]);
            }
        }
        self.sound_active = active;
    }
    pub fn grow(&mut self) {
        for p in self.points.iter_mut().rev() {
            if (p.ox * self.mouse.scale - self.mouse.x)
                .hypot(p.oy * self.mouse.scale - self.mouse.y)
                < 100.
            {
                if !p.draw {
                    p.spray = None;
                    p.decoration = None;
                }
                p.x = p.ox;
                p.y = p.oy;
                p.px = p.ox;
                p.py = p.oy;
                p.pinned = p.original_pinned;
                p.rotation = self.random.next() * std::f64::consts::TAU;
                p.draw = true;
                p.constrain = true;
            }
        }
    }
    pub fn step(&mut self) {
        if self.paused {
            return;
        }
        self.time += 0.016;
        self.next_blink -= 0.016;
        self.blink_remaining = (self.blink_remaining - 0.016).max(0.);
        if self.next_blink <= 0. {
            self.blink_remaining = 0.2;
            self.next_blink = (self.random.next() * 8.).max(1.);
        }
        for _ in 0..3 {
            for i in (0..self.points.len()).rev() {
                if self.points[i].pinned {
                    self.points[i].x = self.points[i].px;
                    self.points[i].y = self.points[i].py;
                    continue;
                }
                let constraints = self.points[i].constraints.clone();
                for j in constraints.into_iter().rev() {
                    if !self.points[i].constrain {
                        continue;
                    }
                    let (a, b) = self.points.split_at_mut(i);
                    let p = &mut b[0];
                    let q = &mut a[j];
                    let dx = p.x - q.x;
                    let dy = p.y - q.y;
                    let distance = dx.hypot(dy);
                    let difference = (20. - distance) / distance;
                    if self.tool == Tool::Clipper && dy > 80. {
                        p.constrain = false;
                        p.pinned = false;
                    }
                    let px = dx * difference * 0.5;
                    let py = dy * difference * 0.5;
                    p.x += px;
                    p.y += py;
                    q.x -= px;
                    q.y -= py;
                }
                let p = &mut self.points[i];
                if p.x > 1199. {
                    p.x = 2398. - p.x;
                } else if p.x < 1. {
                    p.x = 2. - p.x;
                }
                if p.y < 1. {
                    p.y = 2. - p.y;
                } else if p.y > 1399. {
                    p.y = 1399.;
                    p.draw = false;
                }
            }
        }
        for p in self.points.iter_mut().rev() {
            let dx = p.x * self.mouse.scale - self.mouse.x;
            let dy = p.y * self.mouse.scale - self.mouse.y;
            let distance = dx.hypot(dy);
            let mut force_x = 0.;
            if self.mouse.down {
                if self.tool == Tool::Clipper && distance < 20. {
                    p.constrain = false;
                    p.pinned = false;
                }
                if self.tool == Tool::Dryer {
                    force_x = if dx == 0. {
                        f64::NAN
                    } else {
                        dx / dx.abs() * 5500.
                    };
                }
                if self.tool == Tool::Hand && distance < 20. {
                    p.px = p.x - (self.mouse.x - self.mouse.px);
                    p.py = p.y - (self.mouse.y - self.mouse.py);
                }
            }
            let nx = p.x + (p.x - p.px) * 0.8 + force_x / 2. * (0.032 * 0.032);
            let ny = p.y + (p.y - p.py) * 0.8 + 1000. * (0.032 * 0.032);
            p.px = p.x;
            p.py = p.y;
            p.x = nx;
            p.y = ny;
        }
    }
    pub fn save(&self) -> String {
        const ENCODER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let chars = self.points.iter().map(|p| {
            let mut index = usize::from(p.draw && p.constrain);
            if p.draw {
                if let Some(s) = p.spray {
                    index += (s as usize + 1) * 5;
                }
                if let Some(d) = p.decoration {
                    index += d as usize + 1;
                }
            }
            ENCODER[index] as char
        });
        let mut previous = None;
        let mut count = 0;
        let mut out = String::new();
        for c in chars {
            if previous == Some(c) {
                count += 1;
            } else {
                if let Some(p) = previous {
                    out.push_str(&format!("{count}{p}"));
                }
                previous = Some(c);
                count = 1;
            }
        }
        if let Some(p) = previous {
            out.push_str(&format!("{count}{p}"));
        }
        out
    }
    pub fn restore(&mut self, encoded: &str) -> Result<(), String> {
        let mut chars = Vec::with_capacity(self.points.len());
        let mut run = 0usize;
        for c in encoded.chars() {
            if let Some(d) = c.to_digit(10) {
                run = run
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(d as usize))
                    .filter(|v| *v <= self.points.len())
                    .ok_or("Invalid beard data")?;
            } else if c.is_ascii_alphabetic()
                && "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrst".contains(c)
                && run > 0
            {
                if chars.len() + run > self.points.len() {
                    return Err("Beard data is too long".into());
                }
                chars.extend(std::iter::repeat_n(c, run));
                run = 0;
            } else {
                return Err("Invalid beard data".into());
            }
        }
        if run != 0 || chars.len() != self.points.len() {
            return Err("Invalid beard length".into());
        }
        self.reset();
        for (p, c) in self.points.iter_mut().zip(chars) {
            let index = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
                .find(c)
                .unwrap() as i32;
            p.draw = index > 0;
            p.constrain = index > 0;
            let spray = ((index - 1) as f64 / 5.).floor() as i32;
            p.spray = if spray > 0 {
                Some((spray - 1) as u8)
            } else {
                None
            };
            let decoration = (index - 1) % 5;
            p.decoration = if decoration > 0 {
                Some((decoration - 1) as u8)
            } else {
                None
            };
        }
        Ok(())
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"save":self.save(),"points":self.points.iter().map(|p|serde_json::json!({"x":p.x,"y":p.y,"px":p.px,"py":p.py,"pinned":p.pinned,"draw":p.draw,"constrain":p.constrain,"rotation":p.rotation,"spray":p.spray,"decoration":p.decoration})).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn beard_roundtrip_and_invalid_restore_is_atomic() {
        let mut game = Selfie::new("test");
        game.points[10].spray = Some(7);
        game.points[10].decoration = Some(3);
        let saved = game.save();
        game.reset();
        game.restore(&saved).unwrap();
        assert_eq!(game.save(), saved);
        assert_eq!(game.points[10].spray, Some(7));
        let before = game.snapshot();
        for invalid in ["645z", "99999999999999999999999A", "644B", "645B1A"] {
            assert!(game.restore(invalid).is_err());
            assert_eq!(game.snapshot(), before);
        }
    }
    #[test]
    fn continuous_tool_audio_starts_once_and_stops_on_release() {
        let mut game = Selfie::new("test");
        game.tool = Tool::Dryer;
        game.sounds.clear();
        game.update_tool_sounds(500., true);
        game.update_tool_sounds(600., true);
        game.update_tool_sounds(600., false);
        assert_eq!(game.sounds, vec!["selfie_dryer_start", "selfie_dryer_stop"]);
    }
}
