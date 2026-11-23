//! Native state translation of jetpack/js/controls.js (Google, Apache-2.0).
#[derive(Clone, Copy, serde::Deserialize)]
pub struct Touch {
    pub id: i64,
    pub x: f64,
    pub y: f64,
}
#[derive(Default)]
pub struct Controls {
    pub keys: [bool; 4], // left, up, right, down
    pub touch_id: Option<i64>,
    pub start: [Option<f64>; 2],
    pub current: [Option<f64>; 2],
    pub nub: Option<[f64; 2]>,
    pub visible: bool,
    pub started_in_gui: bool,
    pub input: [f64; 2],
    pub dismissed: Vec<&'static str>,
    leftright_pressed: bool,
    updown_pressed: bool,
    touch_started: bool,
}
impl Controls {
    // Original reset is deliberately empty: keys and touches survive restart.
    pub fn reset(&mut self) {}
    pub fn key_down(&mut self, code: u32) {
        if (37..=40).contains(&code) {
            self.keys[(code - 37) as usize] = true;
        }
        if !self.leftright_pressed && (self.keys[0] || self.keys[2]) {
            self.dismissed
                .extend(["keys-leftright", "spacenav-leftright"]);
            self.leftright_pressed = true;
        }
        if !self.updown_pressed && (self.keys[1] || self.keys[3]) {
            self.dismissed.extend(["keys-updown", "spacenav-updown"]);
            self.updown_pressed = true;
        }
    }
    pub fn key_up(&mut self, code: u32) {
        if (37..=40).contains(&code) {
            self.keys[(code - 37) as usize] = false;
        }
    }
    pub fn touch_start(&mut self, touch: Touch, gui: bool) -> bool {
        self.started_in_gui = gui;
        if self.touch_id.is_some() || gui {
            return false;
        }
        self.show_touch(touch);
        if !self.touch_started {
            self.dismissed.push("touch-updown");
            self.touch_started = true;
        }
        true
    }
    pub fn touch_move(&mut self, changed: &[Touch], target: &[Touch]) {
        if let Some(touch) = self.current_touch(changed, target) {
            self.current = [Some(touch.x), Some(touch.y)];
        }
    }
    pub fn touch_end(&mut self, changed: &[Touch], target: &[Touch]) {
        if self.current_touch(changed, target).is_some() {
            self.hide_touch();
        }
    }
    fn current_touch(&mut self, changed: &[Touch], target: &[Touch]) -> Option<Touch> {
        let id = self.touch_id?;
        if let Some(touch) = changed.iter().find(|touch| touch.id == id) {
            return Some(*touch);
        }
        if !target.iter().any(|touch| touch.id == id) {
            self.hide_touch();
        }
        None
    }
    pub fn show_touch(&mut self, touch: Touch) {
        self.touch_id = Some(touch.id);
        self.start = [Some(touch.x), Some(touch.y)];
        self.visible = true;
        self.nub = None;
        // Source doesn't set touchCurrent until the first move.
    }
    pub fn hide_touch(&mut self) {
        self.touch_id = None;
        self.current = [None, None];
        self.visible = false;
        self.nub = None;
    }
    pub fn raw_input(&self) -> [f64; 2] {
        if let Some(x) = self.current[0] {
            [
                (x - self.start[0].unwrap()) / 70.,
                (self.current[1].unwrap() - self.start[1].unwrap()) / 70.,
            ]
        } else {
            [
                f64::from(self.keys[2]) - f64::from(self.keys[0]),
                f64::from(self.keys[3]) - f64::from(self.keys[1]),
            ]
        }
    }
    pub fn frame(&mut self) {
        let [mut x, mut y] = self.raw_input();
        let squared = x * x + y * y;
        if squared > 1. {
            let length = squared.sqrt();
            x /= length;
            y /= length;
        }
        if self.current[0].is_some() {
            self.nub = Some([x * 70., y * 70.]);
        }
        self.input = [x, y];
    }
}

/// CSS touchpad and nub rectangles in viewport pixels (outside scene scale).
/// Sprite crops are pad [79,0,255,258], nub [0,0,79,85].
pub fn widget_layout(
    viewport_width: f64,
    origin: [f64; 2],
    translation: [f64; 2],
) -> [[f64; 4]; 2] {
    let scale = if viewport_width >= 668. { 0.8 } else { 0.6 };
    let x = origin[0] - 125. + 255. / 2. * (1. - scale);
    let y = origin[1] - 125. + 258. / 2. * (1. - scale);
    [
        [x, y, 255. * scale, 258. * scale],
        [
            x + (255. * 49.02 / 100. - 35. + translation[0]) * scale,
            y + (258. * 48.45 / 100. - 35. + translation[1]) * scale,
            79. * scale,
            85. * scale,
        ],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn touchpad_rectangles_follow_source_css_and_original_nub_states() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/jetpack-touchpad-layout.json")).unwrap();
        assert_eq!(fixture["image"], serde_json::json!([334, 258]));
        assert_eq!(fixture["pad_source"], serde_json::json!([79, 0, 255, 258]));
        assert_eq!(fixture["nub_source"], serde_json::json!([0, 0, 79, 85]));
        for case in fixture["cases"].as_array().unwrap() {
            let origin: [f64; 2] = serde_json::from_value(case["start"].clone()).unwrap();
            let nub: [f64; 2] = serde_json::from_value(case["nub"].clone()).unwrap();
            let rects = widget_layout(case["width"].as_f64().unwrap(), origin, nub);
            for (actual, expected) in rects.iter().zip([&case["pad"], &case["nub_rect"]]) {
                for (a, b) in actual.iter().zip(expected.as_array().unwrap()) {
                    assert!((a - b.as_f64().unwrap()).abs() < 1e-10, "{case}");
                }
            }
        }
    }

    #[test]
    fn original_controls_event_sequences_and_frame_vectors() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../data/jetpack-controls-reference.json")).unwrap();
        for case in reference["cases"].as_array().unwrap() {
            let mut c = Controls::default();
            for frame in case["frames"].as_array().unwrap() {
                let a = frame["action"].as_array().unwrap();
                match a[0].as_str().unwrap() {
                    "down" => c.key_down(a[1].as_u64().unwrap() as u32),
                    "up" => c.key_up(a[1].as_u64().unwrap() as u32),
                    "reset" => c.reset(),
                    "frame" => c.frame(),
                    "start" => {
                        c.touch_start(
                            serde_json::from_value(a[1].clone()).unwrap(),
                            a[2].as_bool().unwrap(),
                        );
                    }
                    "move" | "end" => {
                        let changed: Vec<Touch> = serde_json::from_value(a[1].clone()).unwrap();
                        let target: Vec<Touch> = serde_json::from_value(a[2].clone()).unwrap();
                        if a[0] == "move" {
                            c.touch_move(&changed, &target)
                        } else {
                            c.touch_end(&changed, &target)
                        }
                    }
                    _ => unreachable!(),
                }
                let output = serde_json::json!({"keys":c.keys,"id":c.touch_id,"start":c.start,"current":c.current,
                    "nub":c.nub,"visible":c.visible,"gui":c.started_in_gui,"input":c.input,"dismissed":c.dismissed});
                // DOM preventDefault itself belongs to the platform adapter.
                let mut expected = frame["output"].clone();
                expected.as_object_mut().unwrap().remove("prevented");
                fn equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
                    match (a, b) {
                        (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                            a.as_f64() == b.as_f64()
                        }
                        (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b))
                        }
                        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                            a.len() == b.len()
                                && a.iter().all(|(k, v)| b.get(k).is_some_and(|b| equal(v, b)))
                        }
                        _ => a == b,
                    }
                }
                assert!(
                    equal(&output, &expected),
                    "{} {a:?}: {output} != {expected}",
                    case["name"]
                );
                c.dismissed.clear();
            }
        }
    }
}
