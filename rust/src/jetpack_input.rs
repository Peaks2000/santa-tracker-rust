//! Ordered native event adapter for the original Jetpack Controls state.
use crate::jetpack_controls::{Controls, Touch};
use macroquad::miniquad::{EventHandler, KeyCode, KeyMods, MouseButton, TouchPhase};
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Down(u32),
    Up(u32),
    MouseDown(f32, f32),
    MouseMove(f32, f32),
    MouseUp(f32, f32),
    Touch(TouchPhase, u64, f32, f32),
}
#[derive(Default)]
pub struct Events(pub Vec<Event>);
impl EventHandler for Events {
    fn update(&mut self) {}
    fn draw(&mut self) {}
    fn key_down_event(&mut self, key: KeyCode, _mods: KeyMods, _repeat: bool) {
        self.0.push(Event::Down(key_code(key)));
    }
    fn key_up_event(&mut self, key: KeyCode, _mods: KeyMods) {
        self.0.push(Event::Up(key_code(key)));
    }
    fn mouse_button_down_event(&mut self, b: MouseButton, x: f32, y: f32) {
        if b == MouseButton::Left {
            self.0.push(Event::MouseDown(x, y));
        }
    }
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.0.push(Event::MouseMove(x, y));
    }
    fn mouse_button_up_event(&mut self, b: MouseButton, x: f32, y: f32) {
        if b == MouseButton::Left {
            self.0.push(Event::MouseUp(x, y));
        }
    }
    fn touch_event(&mut self, p: TouchPhase, id: u64, x: f32, y: f32) {
        self.0.push(Event::Touch(p, id, x, y));
    }
}
fn key_code(key: KeyCode) -> u32 {
    match key {
        KeyCode::Left => 37,
        KeyCode::Up => 38,
        KeyCode::Right => 39,
        KeyCode::Down => 40,
        _ => 0,
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Owner {
    Mouse,
    Touch,
}
#[derive(Default)]
pub struct Adapter {
    pub controls: Controls,
    touches: Vec<Touch>,
    owner: Option<Owner>,
}
impl Adapter {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn apply(
        &mut self,
        events: &[Event],
        dpi: f32,
        enabled: bool,
        playing: bool,
        mut gui: impl FnMut(f64, f64) -> bool,
    ) -> [f64; 2] {
        if !enabled {
            self.touches.clear();
            self.owner = None;
            return [0., 0.];
        }
        for (i, event) in events.iter().enumerate() {
            // Macroquad emits a mouse event immediately before its matching
            // Touch event. Consume the real touch once, without global changes
            // to other scenes' mouse emulation setting.
            let emulated = matches!((event,events.get(i+1)),
                (Event::MouseDown(x,y),Some(Event::Touch(TouchPhase::Started,_,tx,ty))) |
                (Event::MouseMove(x,y),Some(Event::Touch(TouchPhase::Moved,_,tx,ty))) |
                (Event::MouseUp(x,y),Some(Event::Touch(TouchPhase::Ended,_,tx,ty))) if x==tx && y==ty);
            if emulated {
                continue;
            }
            match *event {
                Event::Down(code) => self.controls.key_down(code),
                Event::Up(code) => self.controls.key_up(code),
                Event::Touch(phase, id, x, y) => {
                    let point = Touch {
                        id: id as i64,
                        x: x as f64 / dpi as f64,
                        y: y as f64 / dpi as f64,
                    };
                    if let Some(index) = self.touches.iter().position(|t| t.id == point.id) {
                        if matches!(phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                            self.touches.remove(index);
                        } else {
                            self.touches[index] = point;
                        }
                    } else if !matches!(phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                        self.touches.push(point);
                    }
                    if self.owner == Some(Owner::Mouse) {
                        continue;
                    }
                    match phase {
                        TouchPhase::Started => {
                            if self
                                .controls
                                .touch_start(point, !playing || gui(point.x, point.y))
                            {
                                self.owner = Some(Owner::Touch);
                            }
                        }
                        TouchPhase::Moved => self.controls.touch_move(&[point], &self.touches),
                        TouchPhase::Ended => self.controls.touch_end(&[point], &self.touches),
                        // Original registers no touchcancel handler. Registry
                        // removal lets its next getCurrentTouch recover loss.
                        TouchPhase::Cancelled => {}
                    }
                }
                Event::MouseDown(x, y) => {
                    if self.owner != Some(Owner::Touch) {
                        let point = Touch {
                            id: 0,
                            x: x as f64 / dpi as f64,
                            y: y as f64 / dpi as f64,
                        };
                        if self
                            .controls
                            .touch_start(point, !playing || gui(point.x, point.y))
                        {
                            self.owner = Some(Owner::Mouse);
                        }
                    }
                }
                Event::MouseMove(x, y) => {
                    if self.owner == Some(Owner::Mouse) {
                        let p = Touch {
                            id: 0,
                            x: x as f64 / dpi as f64,
                            y: y as f64 / dpi as f64,
                        };
                        self.controls.touch_move(&[p], &[p]);
                    }
                }
                Event::MouseUp(x, y) => {
                    if self.owner == Some(Owner::Mouse) {
                        self.controls.touch_end(
                            &[Touch {
                                id: 0,
                                x: x as f64 / dpi as f64,
                                y: y as f64 / dpi as f64,
                            }],
                            &[],
                        );
                    }
                }
            }
            if self.controls.touch_id.is_none() {
                self.owner = None;
            }
        }
        if playing {
            self.controls.frame();
        }
        // Core performs source normalization once, after this raw vector.
        self.controls.raw_input()
    }
    pub fn release_missing_mouse(&mut self) -> bool {
        if self.owner == Some(Owner::Mouse) {
            self.controls.hide_touch();
            self.owner = None;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        use serde_json::Value::*;
        match (a, b) {
            (Number(a), Number(b)) => a.as_f64() == b.as_f64(),
            (Array(a), Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b)),
            (Object(a), Object(b)) => {
                a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|b| equal(v, b)))
            }
            _ => a == b,
        }
    }
    #[test]
    fn ordered_native_callbacks_match_original_control_batches() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../data/jetpack-native-input-reference.json"))
                .unwrap();
        for case in reference["cases"].as_array().unwrap() {
            let mut adapter = Adapter::default();
            let mut collected = Events::default();
            for frame in case["frames"].as_array().unwrap() {
                collected.0.clear();
                for event in frame["e"].as_array().unwrap() {
                    let a = event.as_array().unwrap();
                    let kind = a[0].as_str().unwrap();
                    if matches!(kind, "down" | "up") {
                        let key = match a[1].as_u64().unwrap() {
                            37 => KeyCode::Left,
                            38 => KeyCode::Up,
                            39 => KeyCode::Right,
                            40 => KeyCode::Down,
                            _ => KeyCode::A,
                        };
                        if kind == "down" {
                            collected.key_down_event(key, KeyMods::default(), false);
                        } else {
                            collected.key_up_event(key, KeyMods::default());
                        }
                    } else if matches!(kind, "md" | "mm" | "mu") {
                        let x = a[1].as_f64().unwrap() as f32;
                        let y = a[2].as_f64().unwrap() as f32;
                        match kind {
                            "md" => collected.mouse_button_down_event(MouseButton::Left, x, y),
                            "mm" => collected.mouse_motion_event(x, y),
                            _ => collected.mouse_button_up_event(MouseButton::Left, x, y),
                        }
                    } else {
                        let phase = match kind {
                            "ts" => TouchPhase::Started,
                            "tm" => TouchPhase::Moved,
                            "te" => TouchPhase::Ended,
                            "tc" => TouchPhase::Cancelled,
                            _ => unreachable!(),
                        };
                        collected.touch_event(
                            phase,
                            a[1].as_u64().unwrap(),
                            a[2].as_f64().unwrap() as f32,
                            a[3].as_f64().unwrap() as f32,
                        );
                    }
                }
                adapter.apply(
                    &collected.0,
                    case["dpi"].as_f64().unwrap() as f32,
                    true,
                    frame["playing"].as_bool().unwrap_or(true),
                    |x, _| x < 50.,
                );
                let c = &adapter.controls;
                let output = serde_json::json!({"keys":c.keys,"id":c.touch_id,"start":c.start,"current":c.current,"nub":c.nub,
                    "visible":c.visible,"gui":c.started_in_gui,"input":c.input,"dismissed":c.dismissed});
                assert!(
                    equal(&output, &frame["output"]),
                    "{} {frame}: {output}",
                    case["name"]
                );
                adapter.controls.dismissed.clear();
            }
        }
    }
    #[test]
    fn inactive_route_does_not_consume_events_and_mouse_release_recovers() {
        let mut a = Adapter::default();
        a.apply(
            &[
                Event::Down(37),
                Event::Touch(TouchPhase::Started, 9, 100., 100.),
            ],
            1.,
            false,
            false,
            |_, _| false,
        );
        assert_eq!(a.controls.keys, [false; 4]);
        assert_eq!(a.controls.touch_id, None);
        a.reset();
        a.apply(
            &[Event::MouseDown(100., 100.), Event::MouseMove(170., 100.)],
            1.,
            true,
            true,
            |_, _| false,
        );
        assert_eq!(a.controls.input, [1., 0.]);
        assert!(a.controls.visible);
        assert!(a.release_missing_mouse());
        a.controls.frame();
        assert_eq!(a.controls.input, [0., 0.]);
        assert!(!a.controls.visible);
        // A released emulated mouse must not release the actual tracked finger.
        a.apply(
            &[Event::Touch(TouchPhase::Started, 7, 200., 200.)],
            1.,
            true,
            true,
            |_, _| false,
        );
        assert!(!a.release_missing_mouse());
        assert_eq!(a.controls.touch_id, Some(7));
    }
}
