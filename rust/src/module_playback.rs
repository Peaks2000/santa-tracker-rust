//! Selected original modvil module ratio callbacks + Lottie AnimationItem clock.
//! Pending/ready lifecycle and subframe-enabled boolean loops; no queued segments.
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
        Self::from_data(d.ip, d.op, d.fr)
    }
    pub fn from_data(ip: f64, op: f64, frame_rate: f64) -> Result<Self, String> {
        let frames = (op - ip).floor();
        if frames <= 0.
            || frame_rate <= 0.
            || !frames.is_finite()
            || !frame_rate.is_finite()
            || !ip.is_finite()
        {
            return Err("Invalid module timing".into());
        }
        let floor = ip.floor();
        let rounded = if ip - floor >= 0.5 { floor + 1. } else { floor };
        let first_frame = if rounded == 0. {
            0f64.copysign(ip)
        } else {
            rounded
        };
        Ok(Self {
            frames,
            frame_rate,
            first_frame,
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
    ConfigReady,
    DataReady,
    LoadedImages,
    InitItems,
    DomScheduled,
    DomLoaded,
    FontScheduled,
    ResizeScheduled,
    Resize,
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
    pub fn pending(kind: Kind) -> Self {
        let mut state = Self::new(
            kind,
            Timing {
                frames: 0.,
                frame_rate: 0.,
                first_frame: 0.,
            },
        );
        state.loaded = false;
        state
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
/// Logical JSON/image/font/DOM readiness. Callback delivery is explicit; this
/// does not run a browser task scheduler or decode assets.
#[derive(Debug, Serialize)]
pub struct Loading {
    pub state: State,
    pub configured: bool,
    pub images_ready: bool,
    pub fonts_ready: bool,
    pub dom_pending: bool,
    pub dom_loaded: bool,
    pub font_pending: u64,
    pub resize_pending: bool,
}
impl Loading {
    pub fn new(kind: Kind) -> Self {
        Self {
            state: State::pending(kind),
            configured: false,
            images_ready: false,
            fonts_ready: true,
            dom_pending: false,
            dom_loaded: false,
            font_pending: 0,
            resize_pending: false,
        }
    }
    pub fn configure(&mut self, timing: Timing, images: bool, fonts: bool) -> Vec<Event> {
        self.state.timing = timing;
        self.configured = true;
        self.images_ready = images;
        self.fonts_ready = fonts;
        let mut events = vec![Event::ConfigReady, Event::DataReady];
        events.extend(self.wait_fonts());
        events
    }
    fn wait_fonts(&mut self) -> Vec<Event> {
        if self.fonts_ready {
            self.check()
        } else {
            self.font_pending += 1;
            vec![Event::FontScheduled]
        }
    }
    pub fn font_task(&mut self) -> Vec<Event> {
        if self.font_pending == 0 {
            return vec![];
        }
        self.font_pending -= 1;
        self.wait_fonts()
    }
    pub fn images(&mut self, ready: bool) -> Vec<Event> {
        self.images_ready = ready;
        let mut events = vec![Event::LoadedImages];
        events.extend(self.check());
        events
    }
    pub fn check(&mut self) -> Vec<Event> {
        if !self.configured || self.state.loaded || !self.images_ready || !self.fonts_ready {
            return vec![];
        }
        self.state.loaded = true;
        self.dom_pending = true;
        let mut events = vec![Event::InitItems, Event::DomScheduled];
        self.state.set_raw(self.state.raw, &mut events);
        events
    }
    pub fn dom_task(&mut self) -> Vec<Event> {
        if !self.dom_pending {
            return vec![];
        }
        self.dom_pending = false;
        self.dom_loaded = true;
        let mut events = vec![Event::DomLoaded];
        events.extend(self.request_resize());
        events
    }
    pub fn request_resize(&mut self) -> Vec<Event> {
        if !self.configured || self.resize_pending {
            return vec![];
        }
        self.resize_pending = true;
        vec![Event::ResizeScheduled]
    }
    pub fn resize_task(&mut self) -> Vec<Event> {
        if !self.resize_pending {
            return vec![];
        }
        self.resize_pending = false;
        vec![Event::Resize]
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
    fn original_loading_callbacks_and_readiness_match() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/module-loading-reference.json")).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let mut loading = Loading::new(serde_json::from_value(case["kind"].clone()).unwrap());
            let metadata = &case["metadata"];
            for (index, action) in case["actions"].as_array().unwrap().iter().enumerate() {
                let events = match action["type"].as_str().unwrap() {
                    "configure" => loading.configure(
                        Timing::from_data(
                            metadata["ip"].as_f64().unwrap(),
                            metadata["op"].as_f64().unwrap(),
                            metadata["fr"].as_f64().unwrap(),
                        )
                        .unwrap(),
                        action["images"].as_bool().unwrap(),
                        action["fonts"].as_bool().unwrap(),
                    ),
                    "ratio" => loading.state.ratio(action["value"].as_f64()),
                    "advance" => loading.state.advance(action["ms"].as_f64().unwrap()),
                    "images" => loading.images(action["value"].as_bool().unwrap()),
                    "fonts" => {
                        loading.fonts_ready = action["value"].as_bool().unwrap();
                        vec![]
                    }
                    "check" => loading.check(),
                    "font_task" => loading.font_task(),
                    "dom_task" => loading.dom_task(),
                    "request_resize" => loading.request_resize(),
                    "resize_task" => loading.resize_task(),
                    "play" => loading.state.play(),
                    "pause" => loading.state.pause(),
                    other => panic!("{other}"),
                };
                let actual = serde_json::json!({"state":loading,"events":events});
                assert!(
                    equal(&actual, &case["outputs"][index]),
                    "{} step{index} {action}: {actual} != {}",
                    case["name"],
                    case["outputs"][index]
                );
            }
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
