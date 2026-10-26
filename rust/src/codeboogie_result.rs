//! Original Code Boogie result flex layout and typography. Google LLC,
//! Apache-2.0. The DOM/CSS reference stays outside the native application.
use crate::ui::{Ease, Transition};
use serde::Deserialize;

pub struct Panel {
    pub spec: Option<Spec>,
    pub visible: bool,
    pub opacity: Transition<1>,
    pub title: Transition<1>,
    pub body: Transition<1>,
    pub buttons: Transition<1>,
}
impl Default for Panel {
    fn default() -> Self {
        Self {
            spec: None,
            visible: false,
            opacity: Transition::new([0.]),
            title: Transition::new([0.]),
            body: Transition::new([0.]),
            buttons: Transition::new([0.]),
        }
    }
}
impl Panel {
    pub fn show(&mut self, now: f64, mut spec: Spec) {
        spec.show_code = false;
        self.spec = Some(spec);
        self.set_visible(now, true);
    }
    pub fn hide(&mut self, now: f64) {
        self.set_visible(now, false);
    }
    fn set_visible(&mut self, now: f64, visible: bool) {
        if self.visible == visible {
            return;
        }
        self.visible = visible;
        let value = if visible { 1. } else { 0. };
        // `visibility 0 0.3s` has a unitless time. Chromium rejects the
        // entire root transition declaration; only the child fades run.
        self.opacity.set(now, [value], 0., Ease::Default);
        self.title.set_delayed(
            now,
            [value],
            0.4,
            if visible { 0.1 } else { 0.3 },
            Ease::Default,
        );
        self.body.set_delayed(now, [value], 0.4, 0.4, Ease::Default);
        self.buttons
            .set_delayed(now, [value], 0.4, 0.8, Ease::Default);
    }
    pub fn composed(&self, _now: f64) -> bool {
        self.spec.is_some() && self.visible
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Spec {
    pub success: bool,
    pub retry: bool,
    pub message: String,
    pub missing: String,
    pub code: String,
    pub show_code: bool,
    #[serde(rename = "final")]
    pub final_level: bool,
    pub freestyle: bool,
    pub share: String,
}
#[derive(Clone, Copy)]
pub enum Font {
    Regular,
    Bold,
    Lobster,
    Mono,
}
impl Font {
    fn face(self) -> rustybuzz::Face<'static> {
        rustybuzz::Face::from_slice(
            match self {
                Self::Regular => include_bytes!("../assets/fonts/Roboto-Regular.ttf"),
                Self::Bold => include_bytes!("../assets/fonts/Roboto-ExtraBold.ttf"),
                Self::Lobster => include_bytes!("../assets/fonts/Lobster-Regular.ttf"),
                Self::Mono => include_bytes!("../assets/fonts/NotoSansMono.ttf"),
            },
            0,
        )
        .unwrap()
    }
    pub fn width(self, text: &str, size: f64) -> f64 {
        let face = self.face();
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();
        let shaped = rustybuzz::shape(&face, &[], buffer);
        shaped
            .glyph_positions()
            .iter()
            .map(|p| p.x_advance as f64)
            .sum::<f64>()
            * size
            / face.units_per_em() as f64
    }
    pub fn normal_height(self, size: f64) -> f64 {
        let face = self.face();
        let scale = size / face.units_per_em() as f64;
        (face.ascender() as f64 * scale).round() + (-face.descender() as f64 * scale).round()
    }
    pub fn baseline(self, size: f64, line_height: f64) -> f64 {
        let face = self.face();
        let ascent = (face.ascender() as f64 * size / face.units_per_em() as f64).round();
        (line_height - self.normal_height(size)) / 2. + ascent
    }
}
fn px(v: f64) -> f64 {
    (v * 64.).round() / 64.
}
#[derive(Clone, Debug)]
pub struct Text {
    pub lines: Vec<String>,
    pub bounds: [f64; 4],
    pub size: f64,
    pub line_height: f64,
}
impl Text {
    fn new(text: &str, font: Font, size: f64, line_height: f64, max_width: f64) -> Self {
        let words: Vec<_> = text.split_whitespace().collect();
        let full = words.join(" ");
        let width = px(font.width(&full, size)).min(max_width);
        let mut lines = Vec::new();
        let mut current = String::new();
        for word in words {
            let next = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if !current.is_empty() && font.width(&next, size) > width + 0.01 {
                lines.push(current);
                current = word.to_owned();
            } else {
                current = next;
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
        let height = line_height * lines.len() as f64;
        Self {
            lines,
            bounds: [0., 0., width, height],
            size,
            line_height,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Retry,
    Share,
    Continue,
}
#[derive(Clone, Debug)]
pub struct Button {
    pub action: Action,
    pub label: String,
    /// Layout box, before the CSS translate(-3px,-6px).
    pub bounds: [f64; 4],
}
#[derive(Clone)]
pub struct Layout {
    pub title: Text,
    pub details: Option<Text>,
    pub missing: Option<[f64; 4]>,
    pub graphic: Option<[f64; 4]>,
    pub code_label: Option<Text>,
    pub code: Option<[f64; 4]>,
    pub buttons: Vec<Button>,
    pub buttons_box: [f64; 4],
    pub code_link: Option<[f64; 4]>,
    pub button_size: f64,
    pub button_radius: f64,
}
impl Layout {
    pub fn new(width: f64, height: f64, spec: &Spec) -> Self {
        let large = width >= 570. && height >= 500.;
        let padding = if large { 20. } else { 10. };
        let available = width - 2. * padding;
        let title_size = if large { 140. } else { 70. };
        let title = Text::new(
            if spec.success { "Nice moves!" } else { "Oops!" },
            Font::Lobster,
            title_size,
            title_size * 0.8,
            available,
        );
        let details_size = if large { 30. } else { 20. };
        let details = (!spec.show_code && !spec.message.is_empty()).then(|| {
            Text::new(
                &spec.message,
                Font::Bold,
                details_size,
                Font::Bold.normal_height(details_size),
                available.min(530.),
            )
        });
        let missing = (!spec.show_code && !spec.missing.is_empty()).then_some([0., 0., 300., 48.]);
        let graphic =
            (!spec.show_code).then_some([padding, 0., available, if large { 300. } else { 150. }]);
        let code_label = spec.show_code.then(|| {
            Text::new(
                "This is what the blocks you put together look like in JavaScript:",
                Font::Bold,
                if large { 20. } else { 16. },
                32.,
                available.min(530.),
            )
        });
        let code = spec
            .show_code
            .then_some([0., 0., available.min(530.), 180.]);
        let desktop_button = width >= 768.;
        let button_size = if desktop_button { 22. } else { 16. };
        let button_height = if desktop_button { 52. } else { 42. };
        let button_padding = if desktop_button { 24. } else { 16. };
        let mut buttons = Vec::new();
        let mut add = |action, label: &str| {
            buttons.push(Button {
                action,
                label: label.into(),
                bounds: [
                    0.,
                    0.,
                    140f64.max(px(Font::Regular.width(label, button_size)) + button_padding),
                    button_height,
                ],
            })
        };
        if spec.retry {
            add(
                Action::Retry,
                if spec.freestyle {
                    "Keep dancing"
                } else {
                    "Try Again"
                },
            );
        }
        if spec.success && !spec.share.is_empty() {
            add(Action::Share, "Share");
        }
        if spec.success {
            add(
                Action::Continue,
                if spec.final_level {
                    "Finish"
                } else {
                    "Next Dance"
                },
            );
        }
        let gap = px(Font::Regular.width(" ", 12.));
        let max_buttons = available + if large { 30. } else { 10. };
        let ideal = buttons.iter().map(|b| b.bounds[2] + 8.).sum::<f64>()
            + gap * (buttons.len().saturating_sub(1)) as f64;
        let button_width = ideal.min(max_buttons);
        let mut rows: Vec<Vec<usize>> = vec![vec![]];
        let mut row_width = 0.;
        for (i, b) in buttons.iter().enumerate() {
            let next = row_width + if row_width > 0. { gap } else { 0. } + b.bounds[2] + 8.;
            if row_width > 0. && next > button_width + 0.01 {
                rows.push(vec![]);
                row_width = 0.;
            }
            if row_width > 0. {
                row_width += gap;
            }
            row_width += b.bounds[2] + 8.;
            rows.last_mut().unwrap().push(i);
        }
        let link = spec.success && !spec.show_code;
        // [hidden] on the inline-block code link does not override its display
        // rule in Chromium; the empty-code success still retains this row.
        let buttons_height =
            rows.len() as f64 * (button_height + 15.) + if link { 43. } else { 0. };
        let mut s = Self {
            title,
            details,
            missing,
            graphic,
            code_label,
            code,
            buttons,
            buttons_box: [0., 0., button_width, buttons_height],
            code_link: None,
            button_size,
            button_radius: if desktop_button { 8. } else { 5. },
        };
        let mut total = s.title.bounds[3] + 0.4 * title_size + buttons_height;
        if let Some(t) = &s.details {
            total += t.bounds[3] + 10.;
        }
        if let Some(r) = s.missing {
            total += r[3] + 10.;
        }
        if let Some(r) = s.graphic {
            total += r[3] + 30.;
        }
        if let Some(t) = &s.code_label {
            total += t.bounds[3] + 20.;
        }
        if let Some(r) = s.code {
            total += r[3] + 30.;
        }
        let mut deficit = (total - (height - 2. * padding)).max(0.);
        // Text/button min-content sizes freeze. Replaced SVGs can shrink to
        // zero; the graphic has shrink factor four, missing-block factor one.
        if deficit > 0. {
            let graphic_weight = s.graphic.map_or(0., |r| r[3] * 4.);
            let missing_weight = s.missing.map_or(0., |r| r[3]);
            let code_weight = s.code.map_or(0., |r| r[3]);
            let weight = graphic_weight + missing_weight + code_weight;
            if weight > 0. {
                let mut used = 0.;
                if let Some(r) = &mut s.graphic {
                    let d = (deficit * graphic_weight / weight).min(r[3]);
                    r[3] = px(r[3] - d);
                    used += d;
                }
                if let Some(r) = &mut s.missing {
                    let d = (deficit * missing_weight / weight).min(r[3]);
                    r[3] = px(r[3] - d);
                    used += d;
                }
                if let Some(r) = &mut s.code {
                    let min = if large { 38. } else { 18. };
                    let d = (deficit * code_weight / weight).min(r[3] - min);
                    r[3] = px(r[3] - d);
                    used += d;
                }
                deficit -= used;
                total -= used;
                // If the high-shrink graphic hit zero, redistribute its
                // remaining deficit to the other unfrozen flex item.
                if deficit > 0.0001 {
                    if let Some(r) = &mut s.missing {
                        let d = deficit.min(r[3]);
                        r[3] = px(r[3] - d);
                        total -= d;
                    }
                }
            }
        }
        let mut y = px((height - total) / 2.);
        s.title.bounds[0] = px((width - s.title.bounds[2]) / 2.);
        s.title.bounds[1] = y;
        y += s.title.bounds[3] + title_size * 0.4;
        let place = |r: &mut [f64; 4], y: &mut f64, gap: f64| {
            r[0] = px((width - r[2]) / 2.);
            r[1] = *y;
            *y += r[3] + gap;
        };
        if let Some(t) = &mut s.details {
            place(&mut t.bounds, &mut y, 10.);
        }
        if let Some(r) = &mut s.missing {
            place(r, &mut y, 10.);
        }
        if let Some(r) = &mut s.graphic {
            place(r, &mut y, 30.);
        }
        if let Some(t) = &mut s.code_label {
            place(&mut t.bounds, &mut y, 20.);
        }
        if let Some(r) = &mut s.code {
            place(r, &mut y, 30.);
        }
        s.buttons_box[0] = px((width - button_width) / 2.);
        s.buttons_box[1] = y;
        for row in &rows {
            let rw = row
                .iter()
                .map(|i| s.buttons[*i].bounds[2] + 8.)
                .sum::<f64>()
                + gap * row.len().saturating_sub(1) as f64;
            let mut x = px((width - rw) / 2.);
            for &i in row {
                let b = &mut s.buttons[i];
                b.bounds[0] = x + 4.;
                b.bounds[1] = y;
                x += b.bounds[2] + 8. + gap;
            }
            y += button_height + 15.;
        }
        if link {
            let lw = px(Font::Bold.width("View code", 15.)) + 10.;
            s.code_link = Some([px((width - lw) / 2.), y + 15., lw, 28.]);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn result_layout_matches_original_flex_boxes() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../data/codeboogie-result-layout-reference.json"
        ))
        .unwrap();
        for c in fixture["cases"].as_array().unwrap() {
            let spec: Spec = serde_json::from_value(c["spec"].clone()).unwrap();
            let l = Layout::new(
                c["width"].as_f64().unwrap(),
                c["height"].as_f64().unwrap(),
                &spec,
            );
            let mut items = vec![
                (".result__title", Some(l.title.bounds)),
                (".result__details", l.details.map(|t| t.bounds)),
                (".result__missing-block", l.missing),
                (".result__graphic", l.graphic),
                (".result__buttons", Some(l.buttons_box)),
                (".result__code-label", l.code_label.map(|t| t.bounds)),
                (".result__code", l.code),
                (".result__code-link", l.code_link),
            ];
            for b in l.buttons {
                let mut r = b.bounds;
                r[0] -= 3.;
                r[1] -= 6.;
                items.push((
                    match b.action {
                        Action::Retry => ".result__button--retry",
                        Action::Continue => ".result__button--continue",
                        Action::Share => ".result__button--share",
                    },
                    Some(r),
                ));
            }
            for (sel, r) in items {
                if let Some(r) = r {
                    let source = &c["items"][sel];
                    if source.is_null() {
                        continue;
                    }
                    for (i, n) in r.iter().enumerate() {
                        let v = source["box"][i].as_f64().unwrap();
                        assert!(
                            (n - v).abs() < 0.08,
                            "{}x{} {} {sel}[{i}]: {r:?} vs {}",
                            c["width"],
                            c["height"],
                            c["spec"]["name"],
                            source["box"]
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn result_transitions_match_original_delays_and_visibility() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../data/codeboogie-result-phase-reference.json"
        ))
        .unwrap();
        for c in fixture["cases"].as_array().unwrap() {
            let mut p = Panel::default();
            if c["kind"] == "hide" || c["kind"] == "show-during-hide" {
                p.show(-2., Spec::default());
                p.hide(0.);
            } else {
                p.show(0., Spec::default());
            }
            let mut t = c["t"].as_f64().unwrap();
            if let Some(at) = c["reverseAt"].as_f64() {
                if c["kind"] == "show-during-hide" {
                    p.show(at, Spec::default());
                } else {
                    p.hide(at);
                }
                t += at;
            }
            let title = p.title.value(t)[0];
            let values = [
                p.opacity.value(t)[0],
                title,
                p.body.value(t)[0],
                p.body.value(t)[0],
                p.body.value(t)[0],
                p.buttons.value(t)[0],
            ];
            for (i, v) in values.iter().enumerate() {
                let s = &c["items"][i];
                assert!(
                    (v - s["opacity"].as_f64().unwrap()).abs() < 0.00002,
                    "{} t={t} item{i}: {v} vs {s}",
                    c["kind"]
                );
                let y = if i == 1 { -100. * (1. - title) } else { 0. };
                assert!((y - s["y"].as_f64().unwrap()).abs() < 0.0001);
            }
            assert_eq!(
                p.composed(t),
                c["items"][0]["visibility"] == "visible",
                "{} t={t}",
                c["kind"]
            );
        }
    }
}
