//! Native execution of selected shared music control operations.
//! Source metadata, physical voices, automation and wall deadlines are distinct.
use crate::{
    audio::Sample,
    audio_param::Param,
    music_control::{Call, Spec, State},
    music_source::Op,
};
use std::sync::Arc;
#[derive(Clone, Copy)]
pub enum ReferenceBrowser {
    Chrome,
    Firefox,
}
#[derive(Clone, Copy)]
pub enum Event {
    Village,
    Game,
    Hurry,
    Lounge,
    GameOver,
}
impl Event {
    pub fn named(name: &str) -> Option<Self> {
        Some(match name {
            "music_start_village" => Self::Village,
            "music_start_ingame" => Self::Game,
            "music_start_ingame_hurryup" => Self::Hurry,
            "music_start_scene" => Self::Lounge,
            "music_ingame_gameover" => Self::GameOver,
            _ => return None,
        })
    }
    pub fn track(self) -> usize {
        match self {
            Self::Village => 0,
            Self::Game => 1,
            Self::Hurry => 2,
            Self::Lounge => 3,
            Self::GameOver => 4,
        }
    }
}
#[derive(Clone, Copy)]
struct Start {
    at: f64,
    offset: f64,
    duration: Option<f64>,
}
struct Node {
    track: usize,
    id: usize,
    rate: f64,
    bound: bool,
    connected: bool,
    region: Option<(f64, f64)>,
    start: Option<Start>,
    stop: Option<f64>,
    position: f64,
    elapsed: f64,
    started: bool,
    entered_loop: bool,
    ended: bool,
}
struct Deadline {
    track: usize,
    id: usize,
    wall: f64,
    order: u64,
}
pub struct Runtime {
    pub control: State,
    samples: [Arc<Sample>; 5],
    gains: [Param; 5],
    parent: Param,
    music_bus: Param,
    nodes: Vec<Node>,
    deadlines: Vec<Deadline>,
    next_order: u64,
    /// Fixed original bus gains; source0 takes the village bus.
    bus_input: f32,
    village_output: f32,
    browser: ReferenceBrowser,
}
impl Runtime {
    pub fn new(samples: [Arc<Sample>; 5], specs: [Spec; 5], now: f64) -> Self {
        Self::with_browser(samples, specs, now, ReferenceBrowser::Chrome)
    }
    pub fn with_browser(
        samples: [Arc<Sample>; 5],
        specs: [Spec; 5],
        now: f64,
        browser: ReferenceBrowser,
    ) -> Self {
        let buffers = std::array::from_fn(|i| samples[i].frames() as f64 / samples[i].rate as f64);
        let mut gains = std::array::from_fn(|_| Param::new(1.));
        for i in 0..5 {
            gains[i].set(specs[i].volume, now, now);
        }
        Self {
            control: State::configured(&specs, buffers),
            samples,
            gains,
            parent: Param::new(1.),
            music_bus: Param::new(1.),
            nodes: vec![],
            deadlines: vec![],
            next_order: 0,
            bus_input: 0.56234132519035,
            village_output: 0.3981071705535,
            browser,
        }
    }
    pub fn begin_quantum(&mut self, now: f64) {
        for gain in &mut self.gains {
            gain.begin_quantum(now);
        }
        self.parent.begin_quantum(now);
        self.music_bus.begin_quantum(now);
    }
    pub fn transition(&mut self, now: f64, wall: f64, to: usize, ingame: bool) {
        let observed = std::array::from_fn(|i| self.gains[i].current());
        let calls = self.control.transition_to(now, to, observed);
        self.execute(now, wall, calls);
        if ingame {
            // Complete original curveParamLin, Chrome/non-Firefox branch:
            // notably it appends set/ramp events without cancelling old ones.
            for param in [&mut self.parent, &mut self.music_bus] {
                let from = if matches!(self.browser, ReferenceBrowser::Firefox) {
                    0.
                } else {
                    param.current()
                };
                param.set(from, now, now);
                param.linear(1., now + 0.5, now);
            }
        }
    }
    pub fn event(&mut self, now: f64, wall: f64, event: Event) {
        self.transition(now, wall, event.track(), matches!(event, Event::Game));
        if matches!(event, Event::Village) {
            let from = if matches!(self.browser, ReferenceBrowser::Firefox) {
                0.
            } else {
                self.music_bus.current()
            };
            self.music_bus.set(from, now, now);
            self.music_bus.linear(1., now + 0.5, now);
            // The village process also starts or7J into its muted FHaZ bus.
            // That separate disco graph is not part of the five music sources.
        }
    }
    pub fn paused(&mut self, now: f64, paused: bool) {
        let from = if matches!(self.browser, ReferenceBrowser::Firefox) {
            0.
        } else {
            self.parent.current()
        };
        self.parent.set(from, now, now);
        self.parent
            .linear(if paused { 0. } else { 1. }, now + 0.3, now);
    }
    pub fn timers(&mut self, now: f64, wall: f64) {
        // Control callbacks retain creation order for equal deadlines. Dispatch
        // occurs at native callback boundaries, not a simulated browser task loop.
        loop {
            let Some(index) = self
                .deadlines
                .iter()
                .enumerate()
                .filter(|(_, t)| t.wall <= wall)
                .min_by(|(_, a), (_, b)| a.wall.total_cmp(&b.wall).then(a.order.cmp(&b.order)))
                .map(|(i, _)| i)
            else {
                break;
            };
            let timer = self.deadlines.remove(index);
            let calls = self.control.fire(now, timer.track, timer.id);
            self.execute(now, wall, calls);
        }
    }
    pub fn stop_all(&mut self, now: f64, wall: f64) {
        let mut calls = vec![];
        for track in 0..5 {
            let timer_id = self.control.sources[track].stopping_id;
            calls.extend(
                self.control.sources[track]
                    .stop(now, 0.)
                    .ops
                    .into_iter()
                    .map(|op| Call {
                        track,
                        timer_id: if matches!(&op, Op::ClearTimer) {
                            timer_id
                        } else {
                            None
                        },
                        op,
                    }),
            );
        }
        self.execute(now, wall, calls);
        // Explicit native route cleanup is a hard stop of all physical music
        // voices, including old retriggered buffers no longer in source arrays.
        self.nodes.clear();
    }
    fn execute(&mut self, now: f64, wall: f64, calls: Vec<Call>) {
        for Call {
            track,
            op,
            timer_id,
        } in calls
        {
            match op {
                Op::GainCancel { at } => self.gains[track].cancel(at, now),
                Op::GainSet { value, at } => self.gains[track].set(value, at, now),
                Op::GainHold { at, .. } => {
                    let value = self.gains[track].current();
                    self.gains[track].set(value, at, now)
                }
                Op::GainRamp { value, at } => self.gains[track].linear(value, at, now),
                Op::ClearTimer => {
                    let id = timer_id;
                    self.deadlines
                        .retain(|t| t.track != track || Some(t.id) != id);
                }
                Op::SetTimer { id, delay_ms, .. } => {
                    self.deadlines.push(Deadline {
                        track,
                        id,
                        wall: wall + delay_ms.max(0.) / 1000.,
                        order: self.next_order,
                    });
                    self.next_order += 1;
                }
                Op::Create { node } => self.nodes.push(Node {
                    track,
                    id: node,
                    rate: 1.,
                    bound: false,
                    connected: false,
                    region: None,
                    start: None,
                    stop: None,
                    position: 0.,
                    elapsed: 0.,
                    started: false,
                    entered_loop: false,
                    ended: false,
                }),
                Op::Init => {} // selected runtime is created with all buffers bound
                other => {
                    let id = match other {
                        Op::Disconnect { node }
                        | Op::Rate { node, .. }
                        | Op::BindBuffer { node }
                        | Op::Loop { node, .. }
                        | Op::Connect { node }
                        | Op::Start { node, .. }
                        | Op::Stop { node, .. } => node,
                        _ => unreachable!(),
                    };
                    let Some(node) = self
                        .nodes
                        .iter_mut()
                        .find(|n| n.track == track && n.id == id)
                    else {
                        continue;
                    };
                    match other {
                        Op::Disconnect { .. } => node.connected = false,
                        Op::Rate { value, .. } => node.rate = value as f32 as f64,
                        Op::BindBuffer { .. } => node.bound = true,
                        Op::Loop { start, end, .. } => node.region = Some((start, end)),
                        Op::Connect { .. } => node.connected = true,
                        Op::Start {
                            at,
                            offset,
                            duration,
                            ..
                        } => {
                            node.start = Some(Start {
                                at: at.max(now),
                                offset,
                                duration,
                            })
                        }
                        Op::Stop { at, .. } => node.stop = Some(at.max(now)),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
    pub fn frame(&mut self, now: f64, dt: f64) -> [f32; 2] {
        let parent = self.parent.value(now);
        let music = self.music_bus.value(now) * self.bus_input;
        let gains = std::array::from_fn::<_, 5, _>(|i| {
            self.gains[i].value(now) * parent * if i == 0 { self.village_output } else { music }
        });
        let mut out = [0.; 2];
        for node in &mut self.nodes {
            if node.ended {
                continue;
            }
            if node.stop.is_some_and(|t| now >= t) {
                node.ended = true;
                continue;
            }
            let Some(start) = node.start else { continue };
            if now < start.at {
                continue;
            }
            let sample = &self.samples[node.track];
            let duration = sample.frames() as f64 / sample.rate as f64;
            if start.duration.is_some_and(|d| node.elapsed >= d) {
                node.ended = true;
                continue;
            }
            let region = node.region.map(|(a, b)| {
                if a >= 0. && b > 0. && a < b {
                    (a, b.min(duration))
                } else {
                    (0., duration)
                }
            });
            if !node.started {
                node.position = start.offset.clamp(0., duration);
                if let Some((a, b)) = region {
                    if node.rate >= 0. && node.position >= b {
                        node.position = b;
                    } else if node.rate < 0. && node.position < a {
                        node.position = a;
                    }
                }
                node.started = true;
            }
            if let Some((a, b)) = region {
                if !node.entered_loop
                    && ((start.offset < b && node.position >= a)
                        || (start.offset >= b && node.position < b))
                {
                    node.entered_loop = true;
                }
                if node.entered_loop && b > a && (node.position >= b || node.position < a) {
                    node.position = a + (node.position - a).rem_euclid(b - a);
                }
            }
            if node.position >= 0. && node.position < duration && node.bound && node.connected {
                let index = node.position * sample.rate as f64;
                let frame = index.floor() as usize;
                let fraction = index.fract() as f32;
                for (channel, out) in out.iter_mut().enumerate() {
                    let c = channel.min(sample.channels - 1);
                    let a = sample.samples[frame * sample.channels + c];
                    let b = if frame + 1 < sample.frames() {
                        sample.samples[(frame + 1) * sample.channels + c]
                    } else if let Some((loop_start, _)) = region {
                        let at =
                            (loop_start * sample.rate as f64).min((sample.frames() - 1) as f64);
                        let ix = at.floor() as usize;
                        let f = at.fract() as f32;
                        let x = sample.samples[ix * sample.channels + c];
                        let y = sample.samples
                            [((ix + 1).min(sample.frames() - 1)) * sample.channels + c];
                        x + (y - x) * f
                    } else {
                        0.
                    };
                    *out += (a + (b - a) * fraction) * gains[node.track];
                }
            }
            node.position += dt * node.rate;
            node.elapsed += dt * node.rate;
            if region.is_none() && node.rate >= 0. && node.position >= duration {
                node.ended = true;
            }
        }
        out
    }
    pub fn collect_finished(&mut self) {
        self.nodes.retain(|n| !n.ended);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn runtime() -> Runtime {
        let sample = Arc::new(Sample {
            samples: vec![0.1; 48000],
            channels: 1,
            rate: 48000,
        });
        let specs = std::array::from_fn(|i| Spec {
            volume: 1.,
            rate: 1.,
            offset: 0.,
            duration: 0.,
            looping: i != 4,
            retrigger: i == 4,
            loop_start: 0.,
            loop_end: 0.,
            max_sources: -1,
        });
        Runtime::new(std::array::from_fn(|_| sample.clone()), specs, 0.)
    }
    #[test]
    fn transition_uses_original_beat_schedule_and_early_stop_keeps_physical_buffer() {
        let mut rt = runtime();
        rt.event(0., 0., Event::Game);
        rt.begin_quantum(0.);
        assert!((rt.frame(0., 1. / 48000.)[0] - 0.1 * 0.56234132519035).abs() < 1e-7);
        rt.event(0.1, 0.1, Event::GameOver);
        let start = rt
            .nodes
            .iter()
            .find(|n| n.track == 4)
            .unwrap()
            .start
            .unwrap()
            .at;
        assert!((start - 1.188118811881188).abs() < 1e-12);
        rt.timers(1.09, 1.09);
        assert_eq!(rt.control.sources[1].audio.source.buffers, 0);
        let voice = rt.nodes.iter().find(|n| n.track == 1).unwrap();
        let stop = voice.stop.unwrap();
        assert!((stop - 1.288118811881188).abs() < 1e-12);
        assert!(!voice.ended);
        rt.frame(stop - 1. / 48000., 1. / 48000.);
        assert!(!rt.nodes.iter().find(|n| n.track == 1).unwrap().ended);
        rt.frame(stop, 1. / 48000.);
        assert!(rt.nodes.iter().find(|n| n.track == 1).unwrap().ended);
    }
    #[test]
    fn pause_fades_music_without_freezing_its_buffer_phase() {
        let mut rt = runtime();
        rt.event(0., 0., Event::Lounge);
        rt.begin_quantum(0.);
        rt.paused(0., true);
        let mut at_mid = 0.;
        let mut after = 0.;
        for frame in 0..20000 {
            let now = frame as f64 / 48000.;
            if frame % 128 == 0 {
                rt.begin_quantum(now);
            }
            let out = rt.frame(now, 1. / 48000.);
            if frame == 7200 {
                at_mid = out[0];
            }
            if frame == 16000 {
                after = out[0];
            }
        }
        assert!((at_mid - 0.05 * rt.bus_input).abs() < 1e-7);
        assert_eq!(after, 0.);
        let phase = rt.nodes[0].position;
        assert!((phase - 20000. / 48000.).abs() < 1e-10);
        rt.paused(20000. / 48000., false);
        assert_eq!(rt.nodes[0].position, phase);
        for frame in 20000..40000 {
            let now = frame as f64 / 48000.;
            if frame % 128 == 0 {
                rt.begin_quantum(now);
            }
            rt.frame(now, 1. / 48000.);
        }
        assert!((rt.frame(40000. / 48000., 1. / 48000.)[0] - 0.1 * rt.bus_input).abs() < 1e-7);
    }
    #[test]
    fn firefox_curve_helper_uses_supplied_zero_start_instead_of_current_gain() {
        let mut rt = runtime();
        rt.browser = ReferenceBrowser::Firefox;
        rt.event(0., 0., Event::Game);
        rt.begin_quantum(0.);
        assert_eq!(rt.parent.value(0.), 0.);
        assert_eq!(rt.parent.value(0.25), 0.5);
        rt.parent.begin_quantum(1.);
        rt.paused(1., true);
        assert_eq!(rt.parent.value(1.), 0.);
        assert_eq!(rt.parent.value(1.15), 0.);
    }
    #[test]
    fn pending_start_bus_ramp_survives_pause_as_in_original_curve_helper() {
        let mut rt = runtime();
        rt.event(0., 0., Event::Game);
        rt.begin_quantum(0.);
        rt.paused(0., true);
        // curveParamLin never cancels: the older .5s start-game ramp remains
        // after the .3s pause target, raising the bus again from .3 to .5.
        assert_eq!(rt.parent.value(0.3), 0.);
        assert!((rt.parent.value(0.4) - 0.5).abs() < 1e-7);
        assert_eq!(rt.parent.value(0.5), 1.);
    }
    #[test]
    fn source_metadata_reuse_does_not_rewind_or_duplicate_physical_nodes() {
        let mut rt = runtime();
        rt.event(0., 0., Event::Lounge);
        rt.begin_quantum(0.);
        for frame in 0..2400 {
            rt.frame(frame as f64 / 48000., 1. / 48000.);
        }
        let position = rt.nodes[0].position;
        rt.event(0.05, 0.05, Event::Lounge);
        assert_eq!(rt.nodes.len(), 1);
        assert_eq!(rt.nodes[0].position, position);
        rt.event(0.05, 0.05, Event::Game);
        assert!(rt
            .nodes
            .iter()
            .any(|n| n.track == 1 && n.start.unwrap().at > 0.05));
    }
    #[test]
    fn physical_deadline_ids_follow_control_queue_after_overlapping_requests() {
        let mut rt = runtime();
        for (now, event) in [
            (0., Event::Game),
            (0.1, Event::GameOver),
            (0.15, Event::Game),
            (0.16, Event::Hurry),
            (1.5, Event::GameOver),
            (1.51, Event::Game),
            (1.6, Event::Lounge),
            (2., Event::GameOver),
        ] {
            rt.timers(now, now);
            rt.begin_quantum(now);
            rt.event(now, now, event);
            let mut control = rt
                .control
                .sources
                .iter()
                .enumerate()
                .flat_map(|(track, s)| s.timers.iter().map(move |t| (track, t.id)))
                .collect::<Vec<_>>();
            let mut actual = rt
                .deadlines
                .iter()
                .map(|t| (t.track, t.id))
                .collect::<Vec<_>>();
            control.sort();
            actual.sort();
            assert_eq!(actual, control);
        }
    }
}
