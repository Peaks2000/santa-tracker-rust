//! Source music transition planning, including Klang's source-metadata quirks.
//! Calls are plans, not proof that AudioSource.play creates a new buffer.
use serde::{Deserialize, Serialize};
pub const TRACKS: [&str; 5] = ["it5v", "TcWr", "R5CC", "OckO", "z9yE"];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub start_time: f64,
    pub end_time: f64,
    pub duration: f64,
    pub loop_start_time: f64,
    pub start_offset: f64,
    pub loop_start: f64,
    pub loop_end: f64,
    pub stopping: bool,
    pub buffers: usize,
    pub retrigger: bool,
}
fn truthy(n: f64) -> bool {
    n != 0. && !n.is_nan()
}
fn or(n: f64, default: f64) -> f64 {
    if truthy(n) {
        n
    } else {
        default
    }
}
impl Source {
    pub fn playing(&self, now: f64) -> bool {
        (self.end_time == -1. || self.end_time > now) && self.start_time <= now
    }
    pub fn scheduled(&self, now: f64) -> bool {
        self.start_time > now
    }
    pub fn position(&self, now: f64) -> f64 {
        if !self.playing(now) || !truthy(self.duration) {
            return 0.;
        }
        let mut duration = self.duration;
        if truthy(self.loop_start) || truthy(self.loop_end) {
            duration = or(self.loop_end, duration) - or(self.loop_start, 0.);
        }
        let time = now - self.start_time;
        let loop_time = now + self.start_offset - self.loop_start_time;
        if self.start_offset + time > self.duration {
            or(self.loop_start, 0.) + loop_time % duration
        } else {
            self.start_offset + time
        }
    }
    fn stop(&mut self, now: f64) {
        self.stopping = false;
        if self.buffers > 0 {
            self.end_time = now;
            self.buffers = if self.retrigger { self.buffers - 1 } else { 0 };
        }
        // Source leaves endTime unchanged when its buffer array is empty.
        self.start_time = -1.;
    }
}
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Stop {
        track: usize,
    },
    Play {
        track: usize,
        at: f64,
        offset: f64,
        duration: bool,
    },
    FadeIn {
        track: usize,
        at: f64,
        offset: f64,
        duration: f64,
    },
    FadeOut {
        track: usize,
        at: f64,
        duration: f64,
    },
}
#[derive(Debug, Serialize)]
pub struct Plan {
    pub scheduled_time: f64,
    pub actions: Vec<Action>,
}
#[derive(Clone, Copy, Deserialize)]
pub struct Options {
    pub bpm: f64,
    pub sync: f64,
    pub fade_out: f64,
    pub fade_in: f64,
    pub use_offset: bool,
    pub immediate: bool,
    pub set_offset: f64,
}
/// Complete Util.transition decision/formula path over supplied source metadata.
pub fn transition(
    sources: &[Source],
    from: &[usize],
    to: Option<usize>,
    now: f64,
    options: Options,
) -> Plan {
    let mut plan = Plan {
        scheduled_time: now,
        actions: vec![],
    };
    let Some(to) = to else { return plan };
    let bpm = or(options.bpm, 120.);
    let mut fade_out = or(options.fade_out, 2.);
    let mut selected: Option<usize> = None;
    let mut stop = vec![];
    for &index in from {
        let source = &sources[index];
        if source.playing(now) || source.scheduled(now) {
            if selected.is_some_and(|old| sources[old].scheduled(now)) && source.playing(now) {
                selected = Some(index);
            } else if selected.is_none() {
                selected = Some(index);
            }
            if !source.stopping && index != to {
                stop.push(index);
            }
        }
    }
    if selected == Some(to) && !sources[to].stopping {
        return plan;
    }
    let Some(from) = selected else {
        plan.actions.push(Action::Play {
            track: to,
            at: now,
            offset: 0.,
            duration: false,
        });
        return plan;
    };
    let source = &sources[from];
    let bps = 60. / bpm;
    let spb = bpm / 60.;
    let position = or(source.position(now), 0.);
    let beat = position * spb;
    let sync = or(options.sync, 4.);
    let mut next = sync - beat % sync;
    if next < 0.5 {
        next += sync;
    }
    while options.immediate && next < fade_out * spb {
        next += sync;
    }
    let mut delay = next * bps;
    if !source.playing(now) {
        delay = 0.;
    }
    let scheduled = now + delay;
    plan.scheduled_time = scheduled;
    let mut offset = if options.use_offset {
        source.position(now) + delay
    } else {
        0.
    };
    if truthy(options.set_offset) {
        offset = options.set_offset;
    }
    if truthy(options.fade_in) {
        plan.actions.push(Action::FadeIn {
            track: to,
            at: scheduled,
            offset,
            duration: options.fade_in,
        });
    } else {
        plan.actions.push(Action::Play {
            track: to,
            at: scheduled,
            offset,
            duration: false,
        });
    }
    let start = if options.immediate {
        scheduled - fade_out
    } else {
        scheduled
    };
    if options.immediate && start < now {
        fade_out = scheduled - now;
    }
    for track in stop {
        plan.actions.push(Action::FadeOut {
            track,
            at: start,
            duration: fade_out,
        });
    }
    plan
}
/// Actual config transition process: first playing source, gameover cleanup,
/// source-specific BPM/offset, and the later lounge request after the jingle.
pub fn music(sources: &[Source; 5], to: usize, now: f64) -> Plan {
    let mut local = sources.clone();
    let selected = local.iter().position(|s| s.playing(now));
    let mut stops = vec![];
    if selected == Some(4) {
        for (i, source) in local.iter_mut().enumerate() {
            source.stop(now);
            stops.push(Action::Stop { track: i });
        }
    }
    let mut plan = transition(
        &local,
        &selected.into_iter().collect::<Vec<_>>(),
        Some(to),
        now,
        Options {
            bpm: if selected == Some(2) { 117. } else { 202. },
            sync: 0.,
            fade_out: 0.1,
            fade_in: 0.,
            use_offset: false,
            immediate: false,
            set_offset: if selected == Some(2) && to == 1 {
                1.188118811881188
            } else {
                0.
            },
        },
    );
    stops.append(&mut plan.actions);
    plan.actions = stops;
    if to == 4 {
        let at = or(plan.scheduled_time, now) + 7.128712871287128;
        plan.actions.push(Action::Play {
            track: 3,
            at,
            offset: 0.,
            duration: false,
        });
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    fn compare(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        use serde_json::Value::*;
        match (a, b) {
            (Number(a), Number(b)) => (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-10,
            (Array(a), Array(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| compare(a, b))
            }
            (Object(a), Object(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .all(|(k, v)| b.get(k).is_some_and(|b| compare(v, b)))
            }
            _ => a == b,
        }
    }
    #[test]
    fn original_music_process_and_transition_utility_calls_match() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/music-transition-reference.json")).unwrap();
        for case in data["cases"].as_array().unwrap() {
            let input = &case["input"];
            let now = input["now"].as_f64().unwrap();
            let mut sources: [Source; 5] =
                serde_json::from_value(input["sources"].clone()).unwrap();
            let plan = if input["kind"] == "music" {
                music(&sources, input["to"].as_u64().unwrap() as usize, now)
            } else {
                transition(
                    &sources,
                    &serde_json::from_value::<Vec<usize>>(input["from"].clone()).unwrap(),
                    input["to"].as_u64().map(|n| n as usize),
                    now,
                    serde_json::from_value(input["options"].clone()).unwrap(),
                )
            };
            let actual = serde_json::to_value(&plan).unwrap();
            let mut expected = case["output"].clone();
            expected.as_object_mut().unwrap().remove("playing");
            expected.as_object_mut().unwrap().remove("positions");
            assert!(
                compare(&actual, &expected),
                "{input}: {actual} != {expected}"
            );
            // Reference play/fade methods are recorders; only source stop mutates
            // metadata in this oracle, matching these pre-play query semantics.
            for action in &plan.actions {
                if let Action::Stop { track } = action {
                    sources[*track].stop(now);
                }
            }
            let playing: Vec<_> = sources.iter().map(|s| s.playing(now)).collect();
            let positions: Vec<_> = sources
                .iter()
                .map(|s| {
                    let n = s.position(now);
                    if n.is_finite() {
                        Some(n)
                    } else {
                        None
                    }
                })
                .collect();
            assert!(
                compare(&serde_json::json!(playing), &case["output"]["playing"]),
                "{input}"
            );
            assert!(
                compare(&serde_json::json!(positions), &case["output"]["positions"]),
                "{input}"
            );
        }
    }
}
