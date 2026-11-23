//! Original AudioSource stop/fade control, with explicitly delivered wall timers.
//! No audio device or AudioParam evaluation: current_gain is supplied by caller.
use crate::music_source::{Op, Request, Result, State as Audio};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timer {
    pub id: usize,
    pub stop_at: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub audio: Audio,
    pub loop_playing: bool,
    pub schedule_ahead: f64,
    pub stopping_id: Option<usize>,
    pub next_timer: usize,
    pub timers: Vec<Timer>,
}
fn result(return_self: bool) -> Result {
    Result {
        return_self,
        ops: vec![],
    }
}
fn or(value: f64, fallback: f64) -> f64 {
    if value == 0. || value.is_nan() {
        fallback
    } else {
        value
    }
}
impl State {
    fn clear_timer(&mut self) {
        self.timers
            .retain(|timer| Some(timer.id) != self.stopping_id);
        // JS clearTimeout does not reset _stoppingId.
    }
    pub fn play(&mut self, now: f64, request: Request) -> Result {
        let r = self.audio.play(now, request);
        if r.ops.iter().any(|op| matches!(op, Op::ClearTimer)) {
            self.clear_timer();
        }
        r
    }
    pub fn stop(&mut self, now: f64, when: f64) -> Result {
        let mut r = result(true);
        if self.audio.source.stopping {
            self.audio.source.stopping = false;
            self.clear_timer();
            r.ops.push(Op::ClearTimer);
        }
        if !self.audio.nodes.is_empty() {
            let when = or(when, now);
            if self.audio.looping {
                self.loop_playing = false;
            }
            self.audio.source.end_time = when;
            if self.audio.source.retrigger {
                let node = self.audio.nodes.pop().unwrap();
                r.ops.push(Op::Stop {
                    node: node.id,
                    at: when,
                });
            } else {
                for node in self.audio.nodes.drain(..) {
                    r.ops.push(Op::Stop {
                        node: node.id,
                        at: when,
                    });
                    self.audio.source.end_time = now;
                }
            }
        } else {
            self.loop_playing = false;
        }
        self.audio.source.buffers = self.audio.nodes.len();
        self.audio.source.start_time = -1.;
        r
    }
    pub fn deschedule(&mut self, now: f64) -> Result {
        let mut r = result(true);
        self.audio.nodes.retain(|node| {
            let remove = node.start_time.is_some_and(|start| start > now);
            if remove {
                r.ops.extend([
                    Op::Stop {
                        node: node.id,
                        at: 0.,
                    },
                    Op::Disconnect { node: node.id },
                    Op::Disconnect { node: node.id },
                ]);
            }
            !remove
        });
        self.audio.source.buffers = self.audio.nodes.len();
        r
    }
    pub fn pause(&mut self, now: f64) -> Result {
        if self.audio.source.end_time > now {
            self.audio.paused = true;
            self.audio.pause_time += now - self.audio.source.start_time;
            return self.stop(now, 0.);
        }
        result(true)
    }
    pub fn unpause(&mut self, now: f64) -> Result {
        let mut r = result(true);
        if self.audio.paused {
            let offset = self.audio.base_offset;
            self.audio.base_offset += self.audio.pause_time;
            r.ops = self
                .play(
                    now,
                    Request {
                        when: 0.,
                        offset: 0.,
                        duration: 0.,
                        resume: true,
                    },
                )
                .ops;
            self.audio.base_offset = offset;
            self.audio.paused = false;
        }
        r
    }
    pub fn fade_out(
        &mut self,
        now: f64,
        duration: f64,
        when: Option<f64>,
        current_gain: f64,
    ) -> Result {
        if !self.audio.source.playing(now) && !self.audio.source.scheduled(now) {
            return result(false);
        }
        let when = when.unwrap_or(now);
        if when <= self.audio.source.start_time {
            return self.stop(now, 0.);
        }
        let mut r = result(true);
        if self.audio.source.stopping {
            self.clear_timer();
            r.ops.push(Op::ClearTimer);
        }
        r.ops.extend([
            Op::GainCancel { at: when },
            Op::GainSet {
                value: or(current_gain, self.audio.volume),
                at: when,
            },
            Op::GainRamp {
                value: 0.,
                at: when + duration,
            },
        ]);
        let id = self.next_timer;
        self.next_timer += 1;
        self.stopping_id = Some(id);
        self.timers.push(Timer {
            id,
            stop_at: when + duration,
        });
        r.ops.push(Op::SetTimer {
            id,
            delay_ms: (duration + (when - now) - self.schedule_ahead) / 0.001,
            stop_at: when + duration,
        });
        self.audio.source.stopping = true;
        r
    }
    pub fn fade_in(
        &mut self,
        now: f64,
        fade_duration: f64,
        when: f64,
        offset: f64,
        duration: f64,
        current_gain: f64,
    ) -> Result {
        let when = or(when, now);
        if self.audio.looping
            && !self.audio.source.stopping
            && ((!self.audio.source.retrigger
                && (self.audio.source.end_time == -1. || when < self.audio.source.end_time))
                || (self.audio.source.retrigger && self.audio.source.playing(now)))
        {
            return result(false);
        }
        let mut r = result(true);
        r.ops.push(Op::GainCancel { at: when });
        if self.audio.source.stopping {
            self.clear_timer();
            r.ops.extend([
                Op::ClearTimer,
                Op::GainSet {
                    value: current_gain,
                    at: when,
                },
            ]);
        } else {
            self.audio.fading = true;
            let play = self.play(
                now,
                Request {
                    when: if when == now { 0. } else { when },
                    offset,
                    duration,
                    resume: false,
                },
            );
            r.ops.extend(play.ops);
            r.ops.push(Op::GainSet {
                value: 0.,
                at: when,
            });
        }
        self.audio.source.stopping = false;
        r.ops.push(Op::GainRamp {
            value: self.audio.volume,
            at: when + fade_duration,
        });
        r
    }
    /// Deliver this callback explicitly; the host owns actual wall-time scheduling.
    /// Old callbacks inspect the current shared stopping flag, not a generation.
    pub fn fire(&mut self, now: f64, id: usize) -> Result {
        let Some(index) = self.timers.iter().position(|timer| timer.id == id) else {
            return result(false);
        };
        let timer = self.timers.remove(index);
        if !self.audio.source.stopping {
            return result(false);
        }
        self.audio.source.stopping = false;
        if self.audio.looping {
            self.loop_playing = false;
        }
        let mut r = self.stop(now, timer.stop_at);
        r.return_self = false; // timeout closure returns undefined
        r
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
    fn original_stop_fade_and_shared_timer_sequences_match() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/music-lifecycle-reference.json")).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let mut state: State = serde_json::from_value(case["input"]["state"].clone()).unwrap();
            for (index, a) in case["input"]["actions"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let now = a["now"].as_f64().unwrap();
                let num = |key: &str| a[key].as_f64().unwrap_or(0.);
                let r = match a["type"].as_str().unwrap() {
                    "play" => state.play(
                        now,
                        Request {
                            when: num("when"),
                            offset: num("offset"),
                            duration: num("duration"),
                            resume: a["resume"].as_bool().unwrap_or(false),
                        },
                    ),
                    "stop" => state.stop(now, num("when")),
                    "deschedule" => state.deschedule(now),
                    "pause" => state.pause(now),
                    "unpause" => state.unpause(now),
                    "fade_in" => state.fade_in(
                        now,
                        num("duration"),
                        num("when"),
                        num("offset"),
                        num("play_duration"),
                        num("gain"),
                    ),
                    "fade_out" => {
                        state.fade_out(now, num("duration"), a["when"].as_f64(), num("gain"))
                    }
                    "fire" => state.fire(now, a["id"].as_u64().unwrap() as usize),
                    other => panic!("Unknown action {other}"),
                };
                let actual =
                    serde_json::json!({"return_self":r.return_self,"ops":r.ops,"state":state});
                let expected = &case["output"][index];
                assert!(
                    equal(&actual, expected),
                    "{} step{index} {a}: {actual} != {expected}",
                    case["name"]
                );
            }
        }
    }
}
