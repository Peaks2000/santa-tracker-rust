//! Active Snowball Model/Elf animation lifecycle and the bundled THREE r110
//! action/property mixer semantics. Original Google code: Apache-2.0; THREE: MIT.
//! THREE copyright/license notice: assets/shaders/THREE-LICENSE.
//! This implements the scene's active linear TRS tracks, repeat/once actions,
//! callbacks and weight fades; scheduled starts, warps and ping-pong are unused.
use serde::Serialize;
use serde_json::Value;
use std::sync::OnceLock;
#[derive(Clone, Debug, Serialize)]
pub struct Transform {
    pub t: [f64; 3],
    pub r: [f64; 4],
    pub s: [f64; 3],
}
struct Track {
    node: usize,
    rotation: bool,
    times: Vec<f64>,
    values: Vec<f64>,
}
struct Clip {
    name: String,
    duration: f64,
    tracks: Vec<Track>,
}
struct Library {
    rest: Vec<Transform>,
    clips: Vec<Clip>,
}
fn library() -> &'static Library {
    static LIB: OnceLock<Library> = OnceLock::new();
    LIB.get_or_init(|| {
        let doc: Value = serde_json::from_str(include_str!(
            "../../static/scenes/snowball/models/elf-animated.gltf"
        ))
        .unwrap();
        let bytes = include_bytes!("../../static/scenes/snowball/models/elf-animated.bin");
        let read = |index: usize| {
            let a = &doc["accessors"][index];
            assert_eq!(a["componentType"], 5126);
            let view = &doc["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
            let n = match a["type"].as_str().unwrap() {
                "VEC3" => 3,
                "VEC4" => 4,
                "SCALAR" => 1,
                _ => panic!("non-animation accessor"),
            };
            let offset = (view["byteOffset"].as_u64().unwrap_or(0)
                + a["byteOffset"].as_u64().unwrap_or(0)) as usize;
            let stride = view["byteStride"].as_u64().unwrap_or((n * 4) as u64) as usize;
            let mut out = vec![];
            for i in 0..a["count"].as_u64().unwrap() as usize {
                for j in 0..n {
                    let at = offset + i * stride + j * 4;
                    out.push(f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as f64)
                }
            }
            out
        };
        let rest = doc["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| Transform {
                t: std::array::from_fn(|i| n["translation"][i].as_f64().unwrap_or(0.)),
                r: std::array::from_fn(|i| {
                    n["rotation"][i]
                        .as_f64()
                        .unwrap_or(if i == 3 { 1. } else { 0. })
                }),
                s: std::array::from_fn(|i| n["scale"][i].as_f64().unwrap_or(1.)),
            })
            .collect();
        let clips = doc["animations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                let tracks: Vec<_> = c["channels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|ch| {
                        let s = &c["samplers"][ch["sampler"].as_u64().unwrap() as usize];
                        assert_eq!(s["interpolation"], "LINEAR");
                        let path = ch["target"]["path"].as_str().unwrap();
                        assert!(path == "rotation" || path == "translation");
                        Track {
                            node: ch["target"]["node"].as_u64().unwrap() as usize,
                            rotation: path == "rotation",
                            times: read(s["input"].as_u64().unwrap() as usize),
                            values: read(s["output"].as_u64().unwrap() as usize),
                        }
                    })
                    .collect();
                let duration = tracks
                    .iter()
                    .filter_map(|t| t.times.last())
                    .copied()
                    .fold(0f64, f64::max);
                Clip {
                    name: c["name"].as_str().unwrap_or("").into(),
                    duration,
                    tracks,
                }
            })
            .collect();
        Library { rest, clips }
    })
}
/// THREE Quaternion.slerpFlat, including its exact equality / tiny-step branch.
pub fn slerp(a: [f64; 4], b: [f64; 4], mut t: f64) -> [f64; 4] {
    if a == b {
        return a;
    }
    let mut s = 1. - t;
    let cos = a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>();
    let dir = if cos >= 0. { 1. } else { -1. };
    let sq = 1. - cos * cos;
    if sq > f64::EPSILON {
        let sin = sq.sqrt();
        let len = sin.atan2(cos * dir);
        s = (s * len).sin() / sin;
        t = (t * len).sin() / sin
    }
    let mut out = std::array::from_fn(|i| a[i] * s + b[i] * t * dir);
    if s == 1. - t {
        let f = 1. / out.iter().map(|v| v * v).sum::<f64>().sqrt();
        for v in &mut out {
            *v *= f
        }
    }
    out
}
#[derive(Clone, Debug, Serialize)]
pub struct Fade {
    pub times: [f64; 2],
    pub values: [f64; 2],
}
#[derive(Clone, Debug, Serialize)]
pub struct Action {
    pub time: f64,
    pub weight: f64,
    pub effective: f64,
    pub enabled: bool,
    pub paused: bool,
    pub once: bool,
    pub time_scale: f64,
    pub clamp: bool,
    pub fade: Option<Fade>,
}
impl Default for Action {
    fn default() -> Self {
        Self {
            time: 0.,
            weight: 1.,
            effective: 1.,
            enabled: true,
            paused: false,
            once: false,
            time_scale: 1.,
            clamp: false,
            fade: None,
        }
    }
}
impl Action {
    fn reset(&mut self) {
        self.paused = false;
        self.enabled = true;
        self.time = 0.;
        self.fade = None;
    }
    fn weight(&mut self, w: f64) {
        self.weight = w;
        self.effective = if self.enabled { w } else { 0. };
        self.fade = None;
    }
    fn fade(&mut self, now: f64, duration: f64, inward: bool) {
        self.fade = Some(Fade {
            times: [now as f32 as f64, (now + duration) as f32 as f64],
            values: if inward { [0., 1.] } else { [1., 0.] },
        });
    }
    fn update_weight(&mut self, now: f64) -> f64 {
        let mut w = 0.;
        if self.enabled {
            w = self.weight;
            if let Some(f) = self.fade.clone() {
                let t = ((now - f.times[0]) / (f.times[1] - f.times[0])).clamp(0., 1.);
                let value = (f.values[0] * (1. - t) + f.values[1] * t) as f32 as f64;
                w *= value;
                if now > f.times[1] {
                    self.fade = None;
                    if value == 0. {
                        self.enabled = false
                    }
                }
            }
        }
        self.effective = w;
        w
    }
}
#[derive(Clone, Debug)]
enum Callback {
    Throw {
        action: usize,
        captured: Option<usize>,
    },
    Fall {
        variant: usize,
    },
}
#[derive(Clone)]
pub struct Animator {
    pub pose: Vec<Transform>,
    original: Vec<Transform>,
    actions: Vec<Option<Action>>,
    cache: Vec<usize>,
    active: usize,
    pub playing: Option<usize>,
    pub play_time: f64,
    pub mixer_time: f64,
    pub mixed_count: u32,
    callbacks: Vec<Option<Callback>>,
    pub events: Vec<String>,
    pub record_events: bool,
}
impl Default for Animator {
    fn default() -> Self {
        Self::new()
    }
}
impl Animator {
    pub fn new() -> Self {
        let lib = library();
        Self {
            pose: lib.rest.clone(),
            original: lib.rest.clone(),
            actions: vec![None; lib.clips.len()],
            cache: vec![],
            active: 0,
            playing: None,
            play_time: 0.,
            mixer_time: 0.,
            mixed_count: 0,
            callbacks: vec![],
            events: vec![],
            record_events: false,
        }
    }
    fn id(name: &str) -> usize {
        library()
            .clips
            .iter()
            .position(|c| c.name == name)
            .expect("original Snowball clip")
    }
    fn ensure(&mut self, i: usize) {
        if self.actions[i].is_none() {
            self.actions[i] = Some(Action::default());
            self.cache.push(i)
        }
    }
    fn active(&self, i: usize) -> bool {
        self.cache
            .iter()
            .position(|&n| n == i)
            .is_some_and(|n| n < self.active)
    }
    fn activate(&mut self, i: usize) {
        self.ensure(i);
        if !self.active(i) {
            if self.active == 0 {
                self.original = self.pose.clone()
            }
            let index = self.cache.iter().position(|&n| n == i).unwrap();
            self.cache.swap(index, self.active);
            self.active += 1;
        }
    }
    fn stop(&mut self, i: usize) {
        if self.active(i) {
            self.active -= 1;
            let index = self.cache.iter().position(|&n| n == i).unwrap();
            self.cache.swap(index, self.active);
            if self.active == 0 {
                self.pose = self.original.clone()
            }
        }
        self.actions[i].as_mut().unwrap().reset()
    }
    /// Model.play keeps cached loop/clamp/timeScale fields and does not reset a
    /// same-name action. stopAllAction resets active times, not bound properties.
    pub fn play(&mut self, name: &str, now_ms: f64) -> bool {
        let i = Self::id(name);
        if self.playing == Some(i) {
            return false;
        }
        if let Some(current) = self.playing {
            self.actions[current].as_mut().unwrap().weight(0.)
        }
        let old = self.active;
        self.active = 0;
        for k in 0..old {
            self.actions[self.cache[k]].as_mut().unwrap().reset()
        }
        self.ensure(i);
        let action = self.actions[i].as_mut().unwrap();
        action.weight(1.);
        action.reset();
        self.activate(i);
        self.playing = Some(i);
        self.play_time = now_ms;
        true
    }
    pub fn play_once(&mut self, name: &str, now_ms: f64) {
        if self.play(name, now_ms) {
            let i = Self::id(name);
            let a = self.actions[i].as_mut().unwrap();
            a.once = true;
            a.clamp = true;
        }
    }
    pub fn mix_once(&mut self, name: &str) {
        let i = Self::id(name);
        if let Some(a) = &self.actions[i] {
            if a.enabled && !a.paused && a.time_scale != 0. && self.active(i) {
                self.stop(i)
            }
        }
        self.mixed_count += 1;
        self.ensure(i);
        let a = self.actions[i].as_mut().unwrap();
        a.once = true;
        a.reset();
        a.clamp = true;
        a.time_scale = 1.75;
        self.activate(i);
        let captured = self.playing;
        if let Some(current) = captured {
            self.actions[current].as_mut().unwrap().weight = 0.;
        }
        self.callbacks.push(Some(Callback::Throw {
            action: i,
            captured,
        }));
    }
    /// Elf.fallDown's listener deliberately does not filter the finished action.
    pub fn fall(&mut self, variant: usize, now_ms: f64) {
        self.play_once(&format!("elf_rig_fall_down_{variant}"), now_ms);
        self.callbacks.push(Some(Callback::Fall { variant }));
    }
    fn finished(&mut self, i: usize, now_ms: f64) {
        if self.record_events {
            self.events.push(library().clips[i].name.clone());
        }
        let listeners = self.callbacks.clone();
        for (key, callback) in listeners.into_iter().enumerate() {
            match callback {
                Some(Callback::Throw { action, captured }) if action == i => {
                    self.callbacks[key] = None;
                    self.actions[action]
                        .as_mut()
                        .unwrap()
                        .fade(self.mixer_time, 0.25, false);
                    if let Some(current) = captured {
                        let a = self.actions[current].as_mut().unwrap();
                        a.weight(1.);
                        a.fade(self.mixer_time, 0.25, true);
                        if Some(current) != self.playing {
                            if let Some(next) = self.playing {
                                if self.actions[next].as_ref().unwrap().weight < 1. {
                                    let a = self.actions[next].as_mut().unwrap();
                                    a.weight(1.);
                                    a.fade(self.mixer_time, 0.25, true);
                                    self.activate(next)
                                }
                            }
                        }
                    }
                }
                Some(Callback::Fall { variant }) => {
                    let name = format!("elf_rig_fall_down_{variant}_idle");
                    if self.play(&name, now_ms) {
                        let current = self.playing.unwrap();
                        self.actions[current]
                            .as_mut()
                            .unwrap()
                            .fade(self.mixer_time, 0.5, true);
                        self.callbacks[key] = None;
                    }
                }
                _ => {}
            }
        }
        self.callbacks.retain(Option::is_some);
    }
    pub fn update(&mut self, now_ms: f64) {
        if self.playing.is_none() {
            return;
        }
        let delta = (now_ms - self.play_time) / 1000.;
        self.mixer_time += delta;
        let n = self.active;
        let mut accumulated: Vec<Option<[f64; 4]>> = vec![None; library().rest.len() * 2];
        let mut weights = vec![0.; accumulated.len()];
        // Like THREE, capture active length but read its mutable cache each
        // iteration. A finished callback can stop/play actions during this loop.
        for k in 0..n {
            let i = self.cache[k];
            let enabled = self.actions[i].as_ref().unwrap().enabled;
            if !enabled {
                self.actions[i]
                    .as_mut()
                    .unwrap()
                    .update_weight(self.mixer_time);
                continue;
            }
            let a = self.actions[i].as_ref().unwrap();
            let dt = if a.paused { 0. } else { delta * a.time_scale };
            let mut t = a.time + dt;
            let duration = library().clips[i].duration;
            let mut finished = false;
            if dt != 0. {
                if a.once {
                    if t >= duration || t < 0. {
                        t = t.clamp(0., duration);
                        finished = true;
                    }
                    self.actions[i].as_mut().unwrap().time = t;
                    if finished {
                        let a = self.actions[i].as_mut().unwrap();
                        if a.clamp {
                            a.paused = true
                        } else {
                            a.enabled = false
                        }
                        self.finished(i, now_ms)
                    }
                } else {
                    if t >= duration || t < 0. {
                        t -= duration * (t / duration).floor()
                    }
                    self.actions[i].as_mut().unwrap().time = t
                }
            }
            let weight = self.actions[i]
                .as_mut()
                .unwrap()
                .update_weight(self.mixer_time);
            if weight > 0. {
                for track in &library().clips[i].tracks {
                    let value = sample(track, t);
                    let key = track.node * 2 + usize::from(track.rotation);
                    if let Some(prior) = accumulated[key] {
                        let total = weights[key] + weight;
                        let f = weight / total;
                        accumulated[key] = Some(if track.rotation {
                            slerp(prior, value, f)
                        } else {
                            std::array::from_fn(|j| prior[j] * (1. - f) + value[j] * f)
                        });
                        weights[key] = total
                    } else {
                        accumulated[key] = Some(value);
                        weights[key] = weight
                    }
                }
            }
        }
        // Every active clip binds the same 28 translation/quaternion pairs.
        // Inactive bindings are not applied; zero-weight active bindings retain
        // the previous accumulator before mixing with their original value.
        if self.active > 0 {
            for track in &library().clips[self.cache[0]].tracks {
                let key = track.node * 2 + usize::from(track.rotation);
                let orig = if track.rotation {
                    self.original[track.node].r
                } else {
                    let t = self.original[track.node].t;
                    [t[0], t[1], t[2], 0.]
                };
                let value = if let Some(mut value) = accumulated[key] {
                    if weights[key] < 1. {
                        let f = 1. - weights[key];
                        value = if track.rotation {
                            slerp(value, orig, f)
                        } else {
                            std::array::from_fn(|j| value[j] * (1. - f) + orig[j] * f)
                        }
                    }
                    value
                } else {
                    orig
                };
                if track.rotation {
                    self.pose[track.node].r = value
                } else {
                    self.pose[track.node].t = [value[0], value[1], value[2]]
                }
            }
        }
        self.play_time = now_ms;
    }
    pub fn playing_name(&self) -> Option<&'static str> {
        self.playing.map(|i| library().clips[i].name.as_str())
    }
    pub fn snapshot(&self) -> Value {
        serde_json::json!({"playing":self.playing_name(),"playTime":self.play_time,"mixerTime":self.mixer_time,"mixedCount":self.mixed_count,"current":self.playing_name(),"active":self.cache[..self.active].iter().map(|&i|&library().clips[i].name).collect::<Vec<_>>(),"actions":self.cache.iter().map(|&i|{let a=self.actions[i].as_ref().unwrap();serde_json::json!({"name":library().clips[i].name,"time":a.time,"weight":a.weight,"effective":a.effective,"enabled":a.enabled,"paused":a.paused,"loop":if a.once{2200}else{2201},"timeScale":a.time_scale,"clamp":a.clamp,"fade":a.fade})}).collect::<Vec<_>>(),"listeners":self.callbacks.iter().filter(|c|c.is_some()).count(),"events":self.events,"pose":self.pose})
    }
}
fn sample(track: &Track, t: f64) -> [f64; 4] {
    let end = track.times.len() - 1;
    let hi = track.times.partition_point(|v| *v <= t);
    let a = hi.saturating_sub(1).min(end);
    let b = hi.min(end);
    let n = if track.rotation { 4 } else { 3 };
    let va = std::array::from_fn(|j| if j < n { track.values[a * n + j] } else { 0. });
    let vb = std::array::from_fn(|j| if j < n { track.values[b * n + j] } else { 0. });
    let f = if a == b {
        0.
    } else {
        (t - track.times[a]) / (track.times[b] - track.times[a])
    };
    if track.rotation {
        slerp(va, vb, f)
    } else {
        std::array::from_fn(|j| va[j] * (1. - f) + vb[j] * f)
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn compare(a: &Value, b: &Value, path: &str) {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let x = a.as_f64().unwrap();
                let y = b.as_f64().unwrap();
                assert!((x - y).abs() < 1e-9, "{path}: {x} != {y}")
            }
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}[{i}]"))
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                for (k, b) in b {
                    if k != "op" && k != "scaled" {
                        compare(&a[k], b, &format!("{path}.{k}"))
                    }
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    #[test]
    fn model_actions_and_poses_match_original_mixer() {
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-animation-reference.json"))
                .unwrap();
        for case in data["cases"].as_array().unwrap() {
            let mut model = Animator::new();
            model.record_events = true;
            for (index, state) in case["states"].as_array().unwrap().iter().enumerate() {
                let op = &state["op"];
                let now = op["at"].as_f64().unwrap();
                match op["op"].as_str().unwrap() {
                    "play" => {
                        model.play(op["name"].as_str().unwrap(), now);
                    }
                    "mix" => model.mix_once(op["name"].as_str().unwrap()),
                    "fall" => model.fall(op["n"].as_u64().unwrap() as usize, now),
                    "update" => model.update(now),
                    _ => unreachable!(),
                }
                compare(
                    &model.snapshot(),
                    state,
                    &format!("{} state {index}", case["name"]),
                );
            }
        }
    }
}
