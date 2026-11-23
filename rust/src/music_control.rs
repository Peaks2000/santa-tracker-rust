//! Shared music transition process executed through original source bookkeeping.
//! Produces backend operations; gain observations and callback delivery are inputs.
use crate::{
    music_lifecycle::State as Lifecycle,
    music_source::{Op, Request, State as Audio},
    music_transition::{self, Action, Source},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spec {
    pub volume: f64,
    pub rate: f64,
    pub offset: f64,
    pub duration: f64,
    pub looping: bool,
    pub retrigger: bool,
    pub loop_start: f64,
    pub loop_end: f64,
    pub max_sources: i64,
}
impl Spec {
    /// Native bundled config for the exact five shared music sources. Refuse
    /// newly active branches that this selected initializer does not implement.
    pub fn music() -> Result<[Self; 5], String> {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../data/audio.json")).map_err(|e| e.to_string())?;
        let mut specs = Vec::new();
        for id in music_transition::TRACKS {
            let source = &config["audio"][id];
            if source["type"] != "AudioSource" || !source["file_id"].is_string() {
                return Err(format!("Unsupported music source {id}"));
            }
            for key in [
                "streaming",
                "reverse",
                "xfade",
                "panner",
                "priority",
                "volume_start_range",
                "volume_end_range",
                "pitch_start_range",
                "pitch_end_range",
                "max_sources",
            ] {
                if !source[key].is_null() {
                    return Err(format!("Review music {id} field {key}"));
                }
            }
            let number = |key: &str, default: f64| source[key].as_f64().unwrap_or(default);
            specs.push(Self {
                volume: number("volume", 1.),
                rate: {
                    let value = number("playback_rate", 1.);
                    if value == 0. {
                        1.
                    } else {
                        value
                    }
                },
                offset: number("offset", 0.),
                duration: number("duration", 0.),
                looping: source["loop"].as_bool().unwrap_or(false),
                retrigger: source["retrig"].as_bool().unwrap_or(true),
                loop_start: number("loop_start", 0.),
                loop_end: number("loop_end", 0.),
                max_sources: -1,
            });
        }
        specs
            .try_into()
            .map_err(|_| "Expected five music sources".into())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub sources: [Lifecycle; 5],
}
#[derive(Debug, Serialize)]
pub struct Call {
    pub track: usize,
    pub op: Op,
}
impl State {
    /// Selected initialized sources with buffers already bound. The caller must
    /// exclude streaming/reverse/xfade/panner/priority/random config branches.
    /// Undefined startOffset/loopPlaying are encoded 0/false until written:
    /// neither is consulted by the selected methods before that first write.
    pub fn configured(specs: &[Spec; 5], buffers: [f64; 5]) -> Self {
        Self {
            sources: std::array::from_fn(|i| {
                let spec = &specs[i];
                let duration = if spec.duration == 0. || spec.duration.is_nan() {
                    buffers[i]
                } else {
                    spec.duration
                };
                Lifecycle {
                    audio: Audio {
                        source: Source {
                            start_time: 0.,
                            end_time: 0.,
                            duration,
                            loop_start_time: 0.,
                            start_offset: 0.,
                            loop_start: spec.loop_start,
                            loop_end: spec.loop_end,
                            stopping: false,
                            buffers: 0,
                            retrigger: spec.retrigger,
                        },
                        looping: spec.looping,
                        base_offset: spec.offset,
                        volume: spec.volume,
                        rate: spec.rate,
                        buffer_duration: Some(buffers[i]),
                        max_sources: spec.max_sources,
                        paused: false,
                        pause_start_time: -1.,
                        pause_time: -1.,
                        fading: false,
                        nodes: vec![],
                        next_node: 0,
                    },
                    loop_playing: false,
                    schedule_ahead: 0.2,
                    stopping_id: None,
                    next_timer: 0,
                    timers: vec![],
                }
            }),
        }
    }
    pub fn transition_to(&mut self, now: f64, to: usize, gains: [f64; 5]) -> Vec<Call> {
        let sources = std::array::from_fn(|i| self.sources[i].audio.source.clone());
        let plan = music_transition::music(&sources, to, now);
        let mut calls = vec![];
        for action in plan.actions {
            let (track, result) = match action {
                Action::Stop { track } => (track, self.sources[track].stop(now, 0.)),
                Action::Play {
                    track, at, offset, ..
                } => (
                    track,
                    self.sources[track].play(
                        now,
                        Request {
                            when: at,
                            offset,
                            duration: 0.,
                            resume: false,
                        },
                    ),
                ),
                Action::FadeOut {
                    track,
                    at,
                    duration,
                } => (
                    track,
                    self.sources[track].fade_out(now, duration, Some(at), gains[track]),
                ),
                Action::FadeIn {
                    track,
                    at,
                    offset,
                    duration,
                } => {
                    let play_duration = self.sources[track].audio.source.duration;
                    (
                        track,
                        self.sources[track].fade_in(
                            now,
                            duration,
                            at,
                            offset,
                            play_duration,
                            gains[track],
                        ),
                    )
                }
            };
            calls.extend(result.ops.into_iter().map(|op| Call { track, op }));
        }
        calls
    }
    pub fn fire(&mut self, now: f64, track: usize, id: usize) -> Vec<Call> {
        self.sources[track]
            .fire(now, id)
            .ops
            .into_iter()
            .map(|op| Call { track, op })
            .collect()
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
    fn assembled_music_process_matches_original_initialized_sources() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/music-control-reference.json")).unwrap();
        let specs: [Spec; 5] = serde_json::from_value(fixture["specs"].clone()).unwrap();
        let buffers: [f64; 5] = serde_json::from_value(fixture["buffers"].clone()).unwrap();
        assert!(equal(
            &serde_json::to_value(Spec::music().unwrap()).unwrap(),
            &fixture["specs"]
        ));
        for case in fixture["cases"].as_array().unwrap() {
            let mut state = State::configured(&specs, buffers);
            assert!(equal(
                &serde_json::to_value(&state).unwrap(),
                &fixture["initial_state"]
            ));
            for (step, action) in case["actions"].as_array().unwrap().iter().enumerate() {
                let now = action["now"].as_f64().unwrap();
                let ops = if action["type"] == "transition" {
                    state.transition_to(
                        now,
                        action["to"].as_u64().unwrap() as usize,
                        serde_json::from_value(action["gains"].clone()).unwrap(),
                    )
                } else {
                    state.fire(
                        now,
                        action["track"].as_u64().unwrap() as usize,
                        action["id"].as_u64().unwrap() as usize,
                    )
                };
                let actual = serde_json::json!({"state":state,"ops":ops});
                assert!(
                    equal(&actual, &case["outputs"][step]),
                    "{} step{step} {action}: {actual} != {}",
                    case["name"],
                    case["outputs"][step]
                );
            }
        }
    }
}
