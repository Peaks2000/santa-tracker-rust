//! Selected original modvil module ratio callbacks + Lottie AnimationItem clock.
//! Fully loaded, forward, subframe-enabled boolean loops; no queued segments.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Loop,
    Scroll,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Timing {
    pub frames: f64,
    pub frame_rate: f64,
    pub first_frame: f64,
}
impl Timing {
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        #[derive(Deserialize)]
        struct Data {
            ip: f64,
            op: f64,
            fr: f64,
            segments: Option<Vec<serde_json::Value>>,
        }
        let d: Data = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if d.segments.is_some_and(|v| !v.is_empty()) {
            return Err("Queued animation segments require source lifecycle support".into());
        }
        let frames = (d.op - d.ip).floor();
        if frames <= 0. || d.fr <= 0. {
            return Err("Invalid module timing".into());
        }
        Ok(Self {
            frames,
            frame_rate: d.fr,
            first_frame: d.ip.round(),
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub kind: Kind,
    pub timing: Timing,
    pub raw: f64,
    pub frame: f64,
    pub paused: bool,
    pub loaded: bool,
    pub idle: bool,
    pub play_count: i64,
    pub completed_loop: bool,
}
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Active,
    Idle,
    Enter { frame: f64 },
    Render { frame: f64 },
    LoopComplete,
    Complete,
}
impl State {
    pub fn new(kind: Kind, timing: Timing) -> Self {
        Self {
            kind,
            timing,
            raw: 0.,
            frame: 0.,
            paused: true,
            loaded: true,
            idle: true,
            play_count: 0,
            completed_loop: false,
        }
    }
    fn set_raw(&mut self, raw: f64, events: &mut Vec<Event>) {
        self.raw = raw;
        self.frame = raw;
        events.push(Event::Enter { frame: raw });
        if self.loaded {
            events.push(Event::Render {
                frame: raw + self.timing.first_frame,
            });
        }
    }
    pub fn play(&mut self) -> Vec<Event> {
        let mut e = vec![];
        if self.paused {
            self.paused = false;
            if self.idle {
                self.idle = false;
                e.push(Event::Active)
            }
        }
        e
    }
    pub fn pause(&mut self) -> Vec<Event> {
        let mut e = vec![];
        if !self.paused {
            self.paused = true;
            self.idle = true;
            e.push(Event::Idle)
        }
        e
    }
    pub fn ratio(&mut self, value: Option<f64>) -> Vec<Event> {
        match self.kind {
            Kind::Loop => {
                let visible = value.is_some();
                if visible == self.paused {
                    if visible {
                        self.play()
                    } else {
                        self.pause()
                    }
                } else {
                    vec![]
                }
            }
            Kind::Scroll => {
                let mut e = vec![];
                if let Some(ratio) = value {
                    let frame = if ratio.is_nan() {
                        f64::NAN
                    } else {
                        (self.timing.frames * ratio).max(0.).min(self.timing.frames)
                    };
                    self.set_raw(frame, &mut e);
                    e.extend(self.pause());
                }
                e
            }
        }
    }
    pub fn advance(&mut self, delta_ms: f64) -> Vec<Event> {
        let mut events = vec![];
        if self.paused || !self.loaded {
            return events;
        }
        let mut next = self.raw + delta_ms * (self.timing.frame_rate / 1000.);
        let mut complete = false;
        if next >= self.timing.frames - 1. {
            if self.kind != Kind::Loop {
                complete = true;
                next = self.timing.frames - 1.;
            } else if next >= self.timing.frames {
                self.play_count += 1;
                self.set_raw(next % self.timing.frames, &mut events);
                self.completed_loop = true;
                events.push(Event::LoopComplete);
            } else {
                self.set_raw(next, &mut events);
            }
        } else if next < 0. {
            if self.kind == Kind::Loop {
                self.play_count -= 1;
                self.set_raw(
                    self.timing.frames + (next % self.timing.frames),
                    &mut events,
                );
                if self.completed_loop {
                    events.push(Event::LoopComplete);
                } else {
                    self.completed_loop = true;
                }
            } else {
                complete = true;
                next = 0.;
            }
        } else {
            self.set_raw(next, &mut events);
        }
        if complete {
            self.set_raw(next, &mut events);
            events.extend(self.pause());
            events.push(Event::Complete);
        }
        events
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        use serde_json::Value::*;
        match (a, b) {
            (Number(a), Number(b)) => (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-10,
            (Array(a), Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b)),
            (Object(a), Object(b)) => {
                a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|b| equal(v, b)))
            }
            _ => a == b,
        }
    }
    #[test]
    fn original_module_ratio_and_lottie_clock_calls_match() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/module-playback-reference.json")).unwrap();
        for item in fixture["metadata"].as_array().unwrap() {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(item["file"].as_str().unwrap());
            let timing = Timing::from_file(&path).unwrap();
            assert_eq!(timing.frames, item["frames"].as_f64().unwrap());
            assert_eq!(timing.frame_rate, item["frame_rate"].as_f64().unwrap());
            assert_eq!(timing.first_frame, item["first_frame"].as_f64().unwrap());
        }
        for case in fixture["cases"].as_array().unwrap() {
            let mut state: State = serde_json::from_value(case["initial"].clone()).unwrap();
            for (index, action) in case["actions"].as_array().unwrap().iter().enumerate() {
                let events = match action["type"].as_str().unwrap() {
                    "ratio" => state.ratio(action["value"].as_f64()),
                    "ratio_nan" => state.ratio(Some(f64::NAN)),
                    "advance" => state.advance(action["ms"].as_f64().unwrap()),
                    "load" => {
                        state.loaded = action["value"].as_bool().unwrap();
                        vec![]
                    }
                    "play" => state.play(),
                    "pause" => state.pause(),
                    other => panic!("{other}"),
                };
                let actual = serde_json::json!({"state":state,"events":events});
                assert!(
                    equal(&actual, &case["outputs"][index]),
                    "{} step{index} {action}: {actual} != {}",
                    case["name"],
                    case["outputs"][index]
                );
            }
        }
    }
}
