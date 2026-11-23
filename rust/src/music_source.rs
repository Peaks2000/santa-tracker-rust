//! Source-side playback bookkeeping for the five shared music AudioSources.
//! Operations describe calls; they do not execute Web Audio or native playback.
use crate::music_transition::Source;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub buffer_duration: Option<f64>,
    pub start_time: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub source: Source,
    pub looping: bool,
    pub base_offset: f64,
    pub volume: f64,
    pub rate: f64,
    pub buffer_duration: Option<f64>,
    pub max_sources: i64,
    pub paused: bool,
    pub pause_start_time: f64,
    pub pause_time: f64,
    pub fading: bool,
    pub nodes: Vec<Node>,
    pub next_node: usize,
}
#[derive(Clone, Copy, Deserialize)]
pub struct Request {
    pub when: f64,
    pub offset: f64,
    pub duration: f64,
    pub resume: bool,
}
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Op {
    Disconnect {
        node: usize,
    },
    Init,
    GainCancel {
        at: f64,
    },
    GainSet {
        value: f64,
        at: f64,
    },
    GainHold {
        at: f64,
        read_at: f64,
    },
    GainRamp {
        value: f64,
        at: f64,
    },
    ClearTimer,
    Create {
        node: usize,
    },
    Rate {
        node: usize,
        value: f64,
        at: f64,
    },
    BindBuffer {
        node: usize,
    },
    Loop {
        node: usize,
        start: f64,
        end: f64,
    },
    Connect {
        node: usize,
    },
    Start {
        node: usize,
        at: f64,
        offset: f64,
        duration: Option<f64>,
    },
}
#[derive(Debug, Serialize)]
pub struct Result {
    pub return_self: bool,
    pub ops: Vec<Op>,
}
fn truthy(v: f64) -> bool {
    v != 0. && !v.is_nan()
}
fn or(v: f64, default: f64) -> f64 {
    if truthy(v) {
        v
    } else {
        default
    }
}
fn round(v: f64) -> f64 {
    if !v.is_finite() || v == 0. {
        return v;
    }
    let floor = v.floor();
    let rounded = if v - floor >= 0.5 { floor + 1. } else { floor };
    if rounded == 0. {
        0f64.copysign(v)
    } else {
        rounded
    }
}
impl State {
    pub fn play(&mut self, now: f64, request: Request) -> Result {
        let mut result = Result {
            return_self: false,
            ops: vec![],
        };
        let mut when = or(request.when, 0.);
        let mut offset = or(request.offset, 0.);
        self.nodes.retain(|node| {
            let remove = node.buffer_duration.is_none()
                || (!self.looping
                    && node
                        .start_time
                        .zip(node.buffer_duration)
                        .is_some_and(|(start, duration)| start + duration < now));
            if remove {
                result.ops.push(Op::Disconnect { node: node.id });
            }
            !remove
        });
        self.source.buffers = self.nodes.len();
        // Music configs have no priority/max-simultaneous policy or random
        // volume/pitch ranges. Retain the source's explicit max-sources guard.
        if self.max_sources >= 0 && self.nodes.len() as i64 > self.max_sources {
            return result;
        }
        let duration = if truthy(request.duration) {
            request.duration
        } else if self.looping {
            9999999999.
        } else {
            self.source.duration
        };
        let Some(buffer_duration) = self.buffer_duration else {
            result.ops.push(Op::Init);
            return result;
        };
        if when != 0. && when + 0.01 <= now {
            result.return_self = true;
            return result;
        }
        if when == 0. {
            when = now;
        }
        result.ops.extend([
            Op::GainCancel { at: when },
            Op::GainSet {
                value: self.volume,
                at: when,
            },
        ]);
        if !self.paused {
            self.pause_start_time = when;
        }
        if !request.resume {
            self.pause_time = 0.;
        }
        self.source.start_time = when;
        self.source.loop_start_time = when + self.source.duration;
        self.paused = false;
        if self.source.stopping && !self.source.retrigger {
            result.ops.extend([
                Op::GainCancel { at: when },
                Op::GainHold {
                    at: when,
                    read_at: now,
                },
                Op::GainRamp {
                    value: self.volume,
                    at: when + 0.25,
                },
                Op::ClearTimer,
            ]);
            self.source.stopping = false;
            return result;
        }
        self.fading = false;
        if !self.source.retrigger && !self.looping {
            if when < self.source.end_time {
                return result;
            }
        } else if self.looping && !self.source.retrigger {
            if self.source.end_time == -1. || when < self.source.end_time {
                return result;
            }
        } else if self.looping
            && self.source.retrigger
            && self.source.playing(now)
            && !self.source.stopping
        {
            return result;
        } else if self.source.stopping {
            self.source.stopping = false;
        } else if round(self.source.end_time * 1000.) / 1000.
            == round((when + buffer_duration) * 1000.) / 1000.
        {
            result.return_self = true;
            return result;
        }
        self.source.end_time = if self.looping {
            -1.
        } else {
            when + buffer_duration
        };
        let id = self.next_node;
        self.next_node += 1;
        result.ops.extend([
            Op::Create { node: id },
            Op::Rate {
                node: id,
                value: self.rate,
                at: now,
            },
            Op::BindBuffer { node: id },
        ]);
        self.nodes.push(Node {
            id,
            buffer_duration: Some(buffer_duration),
            start_time: None,
        });
        if self.looping {
            result.ops.push(Op::Loop {
                node: id,
                start: or(self.source.loop_start, 0.),
                end: or(self.source.loop_end, buffer_duration),
            });
        }
        result.ops.push(Op::Connect { node: id });
        if offset > self.source.duration {
            offset %= self.source.duration;
        }
        self.source.start_offset = self.base_offset + offset;
        self.nodes.last_mut().unwrap().start_time = Some(when);
        self.source.buffers = self.nodes.len();
        result.ops.push(Op::Start {
            node: id,
            at: when,
            offset: self.source.start_offset,
            duration: if self.looping {
                None
            } else {
                Some(or(duration, buffer_duration))
            },
        });
        result.return_self = true;
        result
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
    fn original_music_source_play_calls_and_bookkeeping_match() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/music-source-reference.json")).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let input = &case["input"];
            let mut state: State = serde_json::from_value(input["state"].clone()).unwrap();
            let result = state.play(
                input["now"].as_f64().unwrap(),
                serde_json::from_value(input["request"].clone()).unwrap(),
            );
            let actual = serde_json::json!({"return_self":result.return_self,"ops":result.ops,"state":state});
            assert!(
                equal(&actual, &case["output"]),
                "{} {input}: {actual} != {}",
                case["name"],
                case["output"]
            );
        }
    }
}
