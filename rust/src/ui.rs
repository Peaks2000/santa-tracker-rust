//! Native translations of santa-button/card-player transitions and lib/time.js.
//! Original behavior: Copyright 2020 Google LLC, Apache-2.0 (see LICENSE).
use serde::Serialize;

#[derive(Clone, Copy, Debug)]
pub enum Ease {
    Linear,
    Default,
    In,
    Out,
    InOut,
    Sidebar,
    InterludeOut,
    OverlayIn,
    OverlayOut,
}
impl Ease {
    pub fn at(self, x: f64) -> f64 {
        let x = x.clamp(0., 1.);
        if x == 0. || x == 1. {
            return x;
        }
        let (a, b, c, d) = match self {
            Self::Linear => return x,
            Self::Default => (0.25, 0.1, 0.25, 1.),
            Self::In => (0.42, 0., 1., 1.),
            Self::Out => (0., 0., 0.58, 1.),
            Self::InOut => (0.42, 0., 0.58, 1.),
            Self::Sidebar => (0.215, 0.610, 0.355, 1.),
            Self::InterludeOut => (0.645, 0., 0.785, 0.390),
            Self::OverlayIn => (0., 1.52, 0.295, 0.86),
            Self::OverlayOut => (0.83, 0.02, 1., 0.01),
        };
        let cubic = |t: f64, a: f64, b: f64| {
            3. * (1. - t).powi(2) * t * a + 3. * (1. - t) * t * t * b + t * t * t
        };
        let (mut lo, mut hi) = (0., 1.);
        for _ in 0..48 {
            let t = (lo + hi) * 0.5;
            if cubic(t, a, c) < x {
                lo = t;
            } else {
                hi = t;
            }
        }
        cubic((lo + hi) * 0.5, b, d)
    }
}
#[derive(Clone, Debug)]
pub struct Transition<const N: usize> {
    from: [f64; N],
    target: [f64; N],
    start: f64,
    duration: f64,
    ease: Ease,
    reversing_start: [f64; N],
    shortening: f64,
}
impl<const N: usize> Transition<N> {
    pub fn new(value: [f64; N]) -> Self {
        Self {
            from: value,
            target: value,
            start: 0.,
            duration: 0.,
            ease: Ease::Linear,
            reversing_start: value,
            shortening: 1.,
        }
    }
    pub fn end_time(&self) -> f64 {
        self.start + self.duration
    }
    pub fn value(&self, now: f64) -> [f64; N] {
        if now < self.start {
            return self.from;
        }
        let t = if self.duration <= 0. {
            1.
        } else {
            self.ease.at((now - self.start) / self.duration)
        };
        std::array::from_fn(|i| self.from[i] + (self.target[i] - self.from[i]) * t)
    }
    pub fn set(&mut self, now: f64, target: [f64; N], duration: f64, ease: Ease) {
        self.set_delayed(now, target, duration, 0., ease);
    }
    /// CSS positive transition-delay, retaining the current value until start.
    pub fn set_delayed(
        &mut self,
        now: f64,
        target: [f64; N],
        duration: f64,
        delay: f64,
        ease: Ease,
    ) {
        if target == self.target {
            return;
        }
        let value = self.value(now);
        let reversing = now < self.start + self.duration && target == self.reversing_start;
        let shortening = if reversing {
            (self.ease.at((now - self.start) / self.duration) * self.shortening
                + (1. - self.shortening))
                .abs()
                .clamp(0., 1.)
        } else {
            1.
        };
        self.reversing_start = if reversing { self.target } else { value };
        self.from = value;
        self.target = target;
        self.start = now + delay;
        self.duration = duration * shortening;
        self.shortening = shortening;
        self.ease = ease;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Split {
    pub days: u64,
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
    pub count: u8,
}
impl Split {
    pub fn values(self) -> [u64; 4] {
        [self.days, self.hours, self.minutes, self.seconds]
    }
    pub fn from_ms(ms: f64) -> Self {
        let ms = if ms.is_nan() { 0. } else { ms.max(0.) };
        // Preserve the original arithmetic order, including JS floating-point boundaries.
        let days_x = ms / 86400000.;
        let days = days_x.floor() as u64;
        let hours_x = (days_x - days as f64) * 24.;
        let hours = hours_x.floor() as u64;
        let minutes_x = (hours_x - hours as f64) * 60.;
        let minutes = minutes_x.floor() as u64;
        let seconds = ((minutes_x - minutes as f64) * 60.).floor() as u64;
        let count = if days > 0 {
            4
        } else if hours > 0 {
            3
        } else if minutes > 0 {
            2
        } else if seconds > 0 {
            1
        } else {
            0
        };
        Self {
            days,
            hours,
            minutes,
            seconds,
            count,
        }
    }
    pub fn visible(self, width: f64) -> [bool; 4] {
        let huge = self.days >= 300;
        std::array::from_fn(|i| {
            let value = 4 - i as u8;
            if huge && value < 4 {
                return false;
            }
            let order = value as i32 - self.count as i32 + 4;
            order <= 4
                && !(order == 3 && width <= 386.
                    || order == 2 && width <= 422.
                    || order == 1 && width <= 458.)
        })
    }
}
pub struct Counter {
    pub current: Split,
    pub previous: Split,
    pub changed: [bool; 4],
    pub tick_at: f64,
    pub until_ms: u64,
    next_tick: f64,
}
impl Counter {
    pub fn new(now_ms: u64, until_ms: u64, time: f64) -> Self {
        let current = Split::from_ms(until_ms.saturating_sub(now_ms) as f64);
        Self {
            current,
            previous: current,
            changed: [false; 4],
            tick_at: time,
            until_ms,
            next_tick: time + 1.,
        }
    }
    pub fn update(&mut self, time: f64, now_ms: u64) {
        if time < self.next_tick {
            return;
        }
        self.next_tick += ((time - self.next_tick).floor() + 1.).max(1.);
        self.previous = self.current;
        self.current = Split::from_ms(self.until_ms.saturating_sub(now_ms) as f64);
        let old = self.previous.values();
        let new = self.current.values();
        self.changed = std::array::from_fn(|i| old[i] != new[i]);
        self.tick_at = time;
    }
    pub fn phase(&self, time: f64) -> f64 {
        Ease::Default.at((time - self.tick_at) / 0.6)
    }
}
/// Original card player: exponential one-way motion, reversing at the current frame.
pub struct CardPlayer {
    pub active: bool,
    start: Option<f64>,
    held: f32,
}
impl Default for CardPlayer {
    fn default() -> Self {
        Self {
            active: false,
            start: None,
            held: 0.,
        }
    }
}
impl CardPlayer {
    pub fn set(&mut self, time: f64, active: bool, duration: f64) {
        if active == self.active {
            return;
        }
        if let Some(start) = self.start {
            let ratio = (time - start) / duration;
            if ratio < 1. {
                let adjusted = out_expo(ratio);
                let start_at = -(adjusted.max(f64::MIN_POSITIVE)).log2() / 10.;
                self.start = Some(time - start_at * duration);
            } else {
                self.start = Some(time);
            }
        } else {
            self.start = Some(time);
        }
        self.active = active;
    }
    pub fn frame(&mut self, time: f64, duration: f64) -> f32 {
        let Some(start) = self.start else {
            return self.held;
        };
        let ratio = (time - start) / duration.max(0.001);
        if ratio >= 1. {
            self.start = None;
        }
        let raw = out_expo(ratio);
        {
            self.held = (if self.active { raw } else { 1. - raw }) as f32;
            self.held
        }
    }
}
fn out_expo(n: f64) -> f64 {
    if n == 1. {
        n
    } else {
        1. - 2f64.powf(-10. * n)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn countdown_and_card_motion_match_original_javascript() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../data/ui-reference.json")).unwrap();
        for entry in reference["splits"].as_array().unwrap() {
            assert_eq!(
                serde_json::to_value(Split::from_ms(entry["ms"].as_f64().unwrap())).unwrap(),
                entry["state"]
            );
        }
        let mut player = CardPlayer::default();
        let mut samples = reference["samples"].as_array().unwrap().iter();
        for action in reference["actions"].as_array().unwrap() {
            let now = action[1].as_f64().unwrap();
            if action[0] == "active" {
                player.set(now, action[2].as_bool().unwrap(), 2.);
            } else {
                let expected = samples.next().unwrap()["frame"].as_f64().unwrap();
                let actual = player.frame(now, 2.) as f64;
                assert!(
                    (actual - expected).abs() < 1e-6,
                    "card {now}: {actual} != {expected}"
                );
            }
        }
    }
    #[test]
    fn responsive_countdown_and_tick_animation() {
        let mut counter = Counter::new(0, 86401250, 0.);
        assert_eq!(counter.current.values(), [1, 0, 0, 1]);
        counter.update(1., 1000);
        assert_eq!(counter.current.values(), [1, 0, 0, 0]);
        assert_eq!(counter.changed, [false, false, false, true]);
        assert_eq!(counter.phase(1.), 0.);
        assert_eq!(counter.phase(1.6), 1.);
        assert_eq!(counter.current.visible(1440.), [true; 4]);
        assert_eq!(counter.current.visible(375.), [true, false, false, false]);
        assert_eq!(
            Split::from_ms(300. * 86400000.).visible(1440.),
            [true, false, false, false]
        );
        assert_eq!(
            Split::from_ms(45000.).visible(375.),
            [false, false, false, true]
        );
    }
    #[test]
    fn rapid_press_reversal_is_continuous_and_shortens_duration() {
        let mut t = Transition::new([0.]);
        t.set(0., [1.], 0.05, Ease::In);
        let halfway = t.value(0.025)[0];
        t.set(0.025, [0.], 0.25, Ease::In);
        assert_eq!(t.value(0.025)[0], halfway);
        assert!((t.duration - 0.25 * halfway).abs() < 1e-12);
        assert_eq!(t.value(1.), [0.]);
    }
}
/// modvil-module.scss and the host's responsive card rules.
pub fn module_height(id: &str, width: f32) -> f32 {
    let width = width.max(414.);
    (width
        * match id {
            "movies" => 0.55555555,
            "footer" => 0.694,
            _ => 0.9,
        })
    .max(800.)
}
pub fn card_rect(
    module: &str,
    index: usize,
    count: usize,
    position: (f32, f32),
    width: f32,
    screen: f32,
    y: f32,
) -> [f32; 4] {
    let width = width.max(414.);
    let h = module_height(module, width);
    if module == "movies" {
        let w = if screen <= 768. { 240. } else { 280. };
        let ch = if screen <= 768. { 200. } else { 210. };
        let host = ch + 20.;
        if screen <= 1024. {
            [
                (width - w) / 2.,
                y + (h - count as f32 * host) / 2. + index as f32 * host,
                w,
                ch,
            ]
        } else {
            let gap = 52.;
            let group = count as f32 * w + (count - 1) as f32 * gap;
            [
                (width - group) / 2. + index as f32 * (w + gap),
                y + (h - host) / 2.,
                w,
                ch,
            ]
        }
    } else if screen <= 768. {
        let parent = (screen * 0.85).min(420.).max(280.);
        let w = (parent * 0.8).min(280.);
        let x = (width - parent) / 2. + if index % 2 == 1 { parent - w } else { 0. };
        [
            x,
            y + (h - count as f32 * 220.) / 2. + index as f32 * 220.,
            w,
            200.,
        ]
    } else {
        let w = (width * 0.3).max(280.);
        let ch = w * 0.75;
        [
            width * position.0 - w / 2.,
            y + h * position.1 - ch / 2.,
            w,
            ch,
        ]
    }
}
#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn home_layout_matches_browser_at_three_viewports() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../data/home-module-layout-reference.json"))
                .unwrap();
        let data = crate::village_data::VillageData::original();
        for width in [1440., 768., 375.] {
            let mut y = if width <= 1024. { 792. } else { 1122. };
            for measured in reference[(width as u32).to_string()].as_array().unwrap() {
                let module = data
                    .modules
                    .iter()
                    .find(|m| m.id == measured["id"].as_str().unwrap())
                    .unwrap();
                assert!(
                    (y - measured["y"].as_f64().unwrap() as f32).abs() < 0.03,
                    "{} y at {width}",
                    module.id
                );
                for (index, card) in module.cards.iter().enumerate() {
                    let native = card_rect(
                        &module.id,
                        index,
                        module.cards.len(),
                        (card.x, card.y),
                        width,
                        width,
                        y,
                    );
                    let original = &measured["cards"][index]["main"];
                    for (i, key) in ["x", "y", "width", "height"].iter().enumerate() {
                        assert!(
                            (native[i] - original[*key].as_f64().unwrap() as f32).abs() < 0.03,
                            "{} {key} at {width}: {} != {}",
                            card.id,
                            native[i],
                            original[*key]
                        );
                    }
                }
                y += module_height(&module.id, width) + if module.no_divider { 0. } else { 72. };
            }
        }
    }
}

/// santa-overlay.scss's column/row media queries and slotted-card margins.
#[derive(Debug)]
pub struct OverlayLayout {
    pub main: [f32; 4],
    pub hero: [f32; 4],
    pub nav: [f32; 4],
    pub below: [f32; 4],
    pub cards: [[f32; 4]; 2],
    pub row: bool,
}
pub fn overlay_layout(w: f32, h: f32) -> OverlayLayout {
    let compact = w <= 400. || h <= 400.;
    let row = h <= 500. && w > 500.;
    let vertical = w <= 500. || row;
    let (cw, ch, m) = if compact {
        (160., 100., 4.)
    } else {
        (200., 120., 8.)
    };
    let hero_h = if h >= 500. { 218. } else { 136. };
    let main_h = hero_h + 72.;
    let bw = if vertical {
        cw + 2. * m
    } else {
        2. * (cw + 2. * m)
    };
    let bh = if vertical {
        2. * (ch + 2. * m)
    } else {
        ch + 2. * m
    };
    let mx = if row {
        (w - 240. - bw - 32.) / 2.
    } else {
        (w - 240.) / 2.
    };
    let my = if row {
        (h - main_h) / 2.
    } else {
        (h - main_h - bh - 32.) / 2.
    };
    let bx = if row { mx + 240. + 16. } else { (w - bw) / 2. };
    let by = if row {
        (h - bh) / 2.
    } else {
        my + main_h + 16.
    };
    OverlayLayout {
        main: [mx, my, 240., main_h],
        hero: [mx, my, 240., hero_h],
        nav: [mx, my + hero_h, 240., 72.],
        below: [bx, by, bw, bh],
        cards: std::array::from_fn(|i| {
            [
                bx + m
                    + if vertical {
                        0.
                    } else {
                        i as f32 * (cw + 2. * m)
                    },
                by + m
                    + if vertical {
                        i as f32 * (ch + 2. * m)
                    } else {
                        0.
                    },
                cw,
                ch,
            ]
        }),
        row,
    }
}

/// The Selfie toolbox's actual flex wrapping, hidden mobile items and relative offsets.
#[derive(Debug)]
pub struct SelfieToolsLayout {
    pub tray: [f32; 4],
    pub rects: [Option<[f32; 4]>; 16],
    pub plain: bool,
    pub desktop: bool,
}
pub fn selfie_tools_layout(w: f32, h: f32) -> SelfieToolsLayout {
    let desktop = w >= 800. && h >= 690.;
    let plain = w <= 800. || h <= 690.;
    let padding = if plain { 5. } else { 0. };
    let content = if plain {
        320.
    } else {
        (232.3f32 * 64.).floor() / 64.
    };
    let mut row_w = [0f32; 16];
    let mut row_h = [0f32; 16];
    let mut row_of = [usize::MAX; 16];
    let mut widths = [0f32; 16];
    let mut heights = [0f32; 16];
    let mut row = 0;
    for i in 0..16 {
        if !desktop && ![0, 1, 3, 4, 7, 8].contains(&i) {
            continue;
        }
        let (ew, eh) = if desktop {
            if i < 4 {
                (104., 156.)
            } else {
                (52., 78.)
            }
        } else {
            (48., 48.)
        };
        if row_w[row] > 0. && row_w[row] + ew > content {
            row += 1;
        }
        row_of[i] = row;
        widths[i] = ew;
        heights[i] = eh;
        row_w[row] += ew;
        row_h[row] = row_h[row].max(eh + if desktop && i == 12 { 20. } else { 0. });
    }
    let count = row + 1;
    let total = row_h[..count].iter().sum::<f32>();
    let extra = if plain {
        0.
    } else {
        (580. - total) / count as f32
    };
    let tray_h = if plain { total + 2. * padding } else { 580. };
    let tx = if desktop {
        20.
    } else {
        w - content - 2. * padding - if w >= 1024. { 50. } else { 0. }
    };
    let ty = if desktop { h / 2. - 290. } else { h - tray_h };
    let mut x_offsets = [0f32; 16];
    let rects = std::array::from_fn(|i| {
        let row = row_of[i];
        if row == usize::MAX {
            return None;
        }
        let x = tx + padding + content - row_w[row] + x_offsets[row] - 15.;
        x_offsets[row] += widths[i];
        let y = ty
            + padding
            + row_h[..row].iter().sum::<f32>()
            + row as f32 * extra
            + if desktop && i < 2 {
                30.
            } else if desktop && i >= 4 {
                -15.
            } else {
                0.
            };
        Some([x, y, widths[i], heights[i]])
    });
    SelfieToolsLayout {
        tray: [tx, ty, content + 2. * padding, tray_h],
        rects,
        plain,
        desktop,
    }
}
#[cfg(test)]
mod overlay_parity_tests {
    use super::*;
    #[test]
    fn overlay_geometry_matches_original_media_queries() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../data/overlay-layout-reference.json")).unwrap();
        for c in cases.as_array().unwrap() {
            let layout = overlay_layout(
                c["w"].as_f64().unwrap() as f32,
                c["h"].as_f64().unwrap() as f32,
            );
            for (key, r) in [
                ("main", layout.main),
                ("hero", layout.hero),
                ("nav", layout.nav),
                ("below", layout.below),
            ] {
                for (i, a) in r.into_iter().enumerate() {
                    assert!(
                        (a as f64 - c[key][i].as_f64().unwrap()).abs() < 0.02,
                        "{key} {:?} {c}",
                        r
                    );
                }
            }
            for (i, r) in layout.cards.iter().enumerate() {
                for (j, a) in r.iter().enumerate() {
                    assert!((*a as f64 - c["cards"][i][j].as_f64().unwrap()).abs() < 0.02);
                }
            }
        }
    }
}
#[cfg(test)]
mod selfie_tools_parity_tests {
    use super::*;
    #[test]
    fn toolbox_flex_geometry_matches_original_desktop_mobile_and_boundary_rules() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../data/selfie-parity-layout-reference.json"))
                .unwrap();
        for (key, c) in cases.as_object().unwrap() {
            let l = selfie_tools_layout(
                c["w"].as_f64().unwrap() as f32,
                c["h"].as_f64().unwrap() as f32,
            );
            let t = &c[".Tools--scrollable"];
            for (i, k) in ["x", "y", "w", "h"].iter().enumerate() {
                assert!(
                    (l.tray[i] as f64 - t[*k].as_f64().unwrap()).abs() < 0.03,
                    "{key} tray {k}"
                );
            }
            for (i, r) in l.rects.iter().enumerate() {
                match r {
                    None => assert!(c["tools"][i].is_null()),
                    Some(r) => {
                        for (j, a) in r.iter().enumerate() {
                            assert!(
                                (*a as f64 - c["tools"][i][j].as_f64().unwrap()).abs() < 0.03,
                                "{key} tool {i} field {j}: {a} != {}",
                                c["tools"][i][j]
                            );
                        }
                    }
                }
            }
        }
    }
}
