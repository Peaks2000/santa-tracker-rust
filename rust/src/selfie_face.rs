//! Source face classes and their selected CSS nose/mouth transitions.
use crate::ui::{Ease, Transition};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Deserialize)]
pub struct Input {
    pub clipper: bool,
    pub dryer: bool,
    pub down: bool,
    pub rel_x: f64,
    pub x: f64,
    pub y: f64,
    pub nose_x: f64,
    pub nose_y: f64,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Targets {
    pub dryer_left: bool,
    pub dryer_right: bool,
    pub dryer_center: bool,
    pub alarmed: bool,
    pub shaving: bool,
    pub nose_scale: f64,
    pub mouth_left: f64,
    pub mouth_width: f64,
    pub mouth_height: f64,
    pub mouth_duration: f64,
}
pub fn targets(i: Input) -> Targets {
    let left = i.dryer && i.down && i.rel_x < -0.3;
    let right = i.dryer && i.down && i.rel_x > 0.3;
    let center = i.dryer && i.down && i.rel_x.abs() < 0.3;
    let dx = i.x - i.nose_x;
    let dy = i.y - 86. - i.nose_y;
    let shaving = i.clipper && i.down && (dx * dx + dy * dy).sqrt() < 120.;
    let (mouth_left, mouth_width, mouth_height) = if i.clipper {
        (250.5, 57., 57.)
    } else if left {
        (304., 40., 40.)
    } else if right {
        (234., 40., 40.)
    } else if center {
        (259., 40., 40.)
    } else {
        (234., 90., 45.)
    };
    Targets {
        dryer_left: left,
        dryer_right: right,
        dryer_center: center,
        alarmed: i.clipper,
        shaving,
        nose_scale: if shaving { 0.7 } else { 1. },
        mouth_left,
        mouth_width,
        mouth_height,
        mouth_duration: if left || right || center { 0.3 } else { 0. },
    }
}
pub struct Motion {
    nose: Transition<1>,
    mouth: Transition<1>,
    initialized: bool,
}
impl Default for Motion {
    fn default() -> Self {
        Self {
            nose: Transition::new([1.]),
            mouth: Transition::new([234.]),
            initialized: false,
        }
    }
}
pub struct Frame {
    pub target: Targets,
    pub nose_scale: f64,
    pub mouth_left: f64,
}
impl Motion {
    pub fn update(&mut self, now: f64, scale: f64, input: Input) -> Frame {
        let target = targets(input);
        if !self.initialized {
            self.mouth = Transition::new([234. * scale]);
            self.initialized = true;
        }
        self.nose.set(now, [target.nose_scale], 0.3, Ease::Default);
        if target.mouth_duration == 0. {
            // Removing transition:left cancels even when the destination left
            // is unchanged (e.g. dryer-right to the base smile).
            self.mouth = Transition::new([target.mouth_left * scale]);
        } else {
            self.mouth.set(
                now,
                [target.mouth_left * scale],
                target.mouth_duration,
                Ease::Default,
            );
        }
        Frame {
            nose_scale: self.nose.value(now)[0],
            mouth_left: self.mouth.value(now)[0],
            target,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> Input {
        Input {
            clipper: false,
            dryer: false,
            down: false,
            rel_x: 0.,
            x: 0.,
            y: 86.,
            nose_x: 0.,
            nose_y: 0.,
        }
    }
    #[test]
    fn original_face_method_and_compiled_css_targets_match() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/selfie-face-reference.json")).unwrap();
        for case in data["cases"].as_array().unwrap() {
            let input = serde_json::from_value(case["input"].clone()).unwrap();
            let expected: Targets = serde_json::from_value(case["output"].clone()).unwrap();
            assert_eq!(targets(input), expected, "{}", case["input"]);
        }
    }
    #[test]
    fn nose_reversal_and_mouth_exit_preserve_selected_css_timing() {
        let mut m = Motion::default();
        let mut i = input();
        m.update(0., 1., i);
        i.clipper = true;
        i.down = true;
        assert_eq!(m.update(1., 1., i).nose_scale, 1.);
        let frame = m.update(1.15, 1., i);
        let e = Ease::Default.at(0.5);
        assert!((frame.nose_scale - (1. - 0.3 * e)).abs() < 1e-12);
        i.down = false;
        let before = m.update(1.15, 1., i).nose_scale;
        assert!((m.update(1.15 + 0.3 * e, 1., i).nose_scale - 1.).abs() < 1e-12);
        assert!(before < 1.);
        i.clipper = false;
        i.dryer = true;
        i.down = true;
        i.rel_x = -1.;
        let start = m.update(2., 1., i).mouth_left;
        assert_eq!(start, 250.5);
        assert!((m.update(2.15, 1., i).mouth_left - (start + (304. - start) * e)).abs() < 1e-12);
        i.down = false;
        assert_eq!(m.update(2.16, 1., i).mouth_left, 234.);
        i.down = true;
        i.rel_x = -1.;
        m.update(3., 1., i);
        m.update(3.3, 1., i);
        i.rel_x = 1.;
        m.update(3.4, 1., i);
        assert!(m.update(3.5, 1., i).mouth_left > 234.);
        i.down = false;
        assert_eq!(m.update(3.5, 1., i).mouth_left, 234.);
    }
}
