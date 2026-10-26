//! Native translations of codeboogie lights, move_tiles, title, animation,
//! animation_player and sequencer. Original: Google LLC, Apache-2.0.
use crate::{
    coding::{DanceItem, Level},
    ui::{Ease, Transition},
};
use serde::Serialize;
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize)]
pub struct TitleNode {
    pub text: String,
    pub small: bool,
    pub sub: bool,
    pub born: f64,
    pub destroyed: Option<f64>,
}
impl TitleNode {
    pub fn appearance(&self, now: f64, bpm: f64) -> (f64, f64) {
        let period = match bpm as u32 {
            120 | 130 | 140 => 60. / bpm,
            _ => 0.5,
        };
        if let Some(at) = self.destroyed {
            let t = Ease::Default.at((now - at - (period - 0.25)) / 0.15);
            (1. - t, 1. + 0.5 * t)
        } else {
            let t = Ease::Default.at((now - self.born - (period - 0.15)) / 0.15);
            (t, 0.8 + 0.2 * t)
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Title {
    pub text: Option<String>,
    pub small: bool,
    pub count: u32,
    pub nodes: Vec<TitleNode>,
    text_node: Option<usize>,
    sub_node: Option<usize>,
}
impl Title {
    pub fn set(&mut self, now: f64, text: Option<String>, small: bool, start_count: u32) {
        if start_count > 0 {
            self.count = start_count;
        }
        self.text = text;
        self.small = small;
        self.update(now);
    }
    fn switch(
        &mut self,
        now: f64,
        old: Option<usize>,
        text: Option<String>,
        small: bool,
        sub: bool,
    ) -> Option<usize> {
        // JS compares numeric counts strictly against DOM textContent strings:
        // even an equal count replaces the existing span.
        let changed = old.map_or(text.as_ref().is_some_and(|s| !s.is_empty()), |i| {
            sub || self.nodes[i].text != text.as_deref().unwrap_or("")
        });
        if !changed {
            return old;
        }
        if let Some(i) = old {
            self.nodes[i].destroyed = Some(now);
        }
        text.filter(|s| !s.is_empty()).map(|text| {
            self.nodes.push(TitleNode {
                text,
                small,
                sub,
                born: now,
                destroyed: None,
            });
            self.nodes.len() - 1
        })
    }
    fn update(&mut self, now: f64) {
        self.text_node = self.switch(now, self.text_node, self.text.clone(), self.small, false);
        self.sub_node = self.switch(
            now,
            self.sub_node,
            (self.count > 0).then(|| self.count.to_string()),
            true,
            true,
        );
    }
    pub fn on_beat(&mut self, now: f64) {
        // Removal is on the next beat, not on animationend.
        let text = self.text_node.map(|i| self.nodes[i].clone());
        let sub = self.sub_node.map(|i| self.nodes[i].clone());
        self.nodes.clear();
        self.text_node = text.map(|n| {
            self.nodes.push(n);
            self.nodes.len() - 1
        });
        self.sub_node = sub.map(|n| {
            self.nodes.push(n);
            self.nodes.len() - 1
        });
        if self.count > 0 {
            self.count += 1;
            self.update(now);
            self.count %= 4;
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MoveTile {
    pub step: String,
    pub born: f64,
    pub removed: Option<f64>,
}
impl MoveTile {
    pub fn appearance(&self, now: f64) -> (f64, f64) {
        if let Some(at) = self.removed {
            let t = Ease::Default.at((now - at) / 0.5);
            (1. - t, 1. + 0.5 * t)
        } else {
            let t = Ease::Default.at((now - self.born) / 0.5);
            (t, 0.8 + 0.2 * t)
        }
    }
}
#[derive(Clone, Debug)]
pub struct MoveTiles {
    pub tiles: Vec<MoveTile>,
    pub length: usize,
    pub fade: bool,
    pub translate: Transition<1>,
    pub translate_to: f64,
    clear_at: Vec<f64>,
}
impl Default for MoveTiles {
    fn default() -> Self {
        Self {
            tiles: vec![],
            length: 4,
            fade: true,
            translate: Transition::new([0.]),
            translate_to: 0.,
            clear_at: vec![],
        }
    }
}
impl MoveTiles {
    pub fn set_length(&mut self, length: usize) {
        self.length = if length == 0 { 4 } else { length.min(4) };
    }
    pub fn add(&mut self, now: f64, step: String) {
        self.tiles.push(MoveTile {
            step,
            born: now,
            removed: None,
        });
        let count = self.tiles.len();
        self.translate_to = -(count as f64) * 12.4;
        self.translate.set(
            now,
            [self.translate_to],
            if count == 1 { 0. } else { 0.2 },
            Ease::In,
        );
        for tile in self.tiles.iter_mut().take(count.saturating_sub(4)) {
            tile.removed.get_or_insert(now);
        }
    }
    pub fn reset(&mut self, now: f64) {
        for tile in &mut self.tiles {
            tile.removed.get_or_insert(now);
        }
        // The source callback removes *all* children, including later additions.
        self.clear_at.push(now + 0.2);
    }
    pub fn clear(&mut self, now: f64) {
        if self.fade {
            self.reset(now);
        }
    }
    pub fn update(&mut self, now: f64) {
        if self.clear_at.iter().any(|at| *at <= now) {
            self.tiles.clear();
        }
        self.clear_at.retain(|at| *at > now);
    }
}

#[derive(Clone, Debug)]
pub struct Lights {
    pub ceiling: bool,
    pub floor: bool,
    pub order: [usize; 9],
    pub tiles: [bool; 9],
    /// DOM order: left white/red/green, right white/red/green.
    pub beams: [Transition<1>; 6],
    pub beam_targets: [bool; 6],
}
impl Default for Lights {
    fn default() -> Self {
        Self {
            ceiling: false,
            floor: false,
            order: std::array::from_fn(|i| i),
            tiles: [false; 9],
            beams: std::array::from_fn(|_| Transition::new([0.])),
            beam_targets: [false; 6],
        }
    }
}
impl Lights {
    pub fn on_beat(
        &mut self,
        now: f64,
        beat: i64,
        playing: bool,
        random: &mut impl FnMut() -> f64,
    ) {
        if self.floor {
            for i in (1..9).rev() {
                let j = (random() * ((i + 1) as f64)).floor() as usize;
                self.order.swap(i, j);
            }
            for (index, id) in self.order.iter().enumerate() {
                self.tiles[*id] = index < 4;
            }
        }
        if self.ceiling {
            let mut targets = [false; 6];
            targets[0] = !playing;
            targets[3] = !playing;
            for (index, id) in [1, 5, 2, 4].iter().enumerate() {
                targets[*id] = playing && beat % 4 == index as i64;
            }
            self.beam_targets = targets;
            for (beam, target) in self.beams.iter_mut().zip(targets) {
                beam.set(now, [if target { 1. } else { 0. }], 0.1, Ease::Default);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SpriteClock {
    pub step: String,
    pub frame: usize,
    pub elapsed: f64,
    pub duration: f64,
    pub count: usize,
}
impl SpriteClock {
    pub fn new(step: String, bpm: f64) -> Self {
        let count = match step.as_str() {
            "idle" => 12,
            "pony" => 104,
            "splits" | "floss" | "hiphop" | "mchammer" => 96,
            _ => 48,
        };
        Self {
            step,
            frame: 0,
            elapsed: 0.,
            duration: 1000. / 24. * (60. / bpm * 2.),
            count,
        }
    }
    pub fn update(&mut self, dt: f64) {
        self.elapsed += dt * 1000.;
        if self.elapsed > self.duration {
            let frames = (self.elapsed / self.duration).floor() as usize;
            self.frame = (self.frame + frames) % self.count;
            self.elapsed -= frames as f64 * self.duration;
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Track(usize),
    Sound(String),
    Step(Option<String>),
    Finish,
}
pub struct DanceStage {
    pub now: f64,
    pub bpm: f64,
    pub track: usize,
    pub beat: i64,
    pub playing: bool,
    pub queue: VecDeque<DanceItem>,
    pub current: Option<DanceItem>,
    pub title: Title,
    pub moves: MoveTiles,
    pub lights: Lights,
    pub player: Option<SpriteClock>,
    pub teacher: Option<SpriteClock>,
    pub player_watch: Transition<1>,
    pub teacher_show: Transition<1>,
    pub watching: bool,
    pub showing: bool,
    pub events: Vec<Event>,
    end_title: Option<String>,
    play_at: f64,
    scheduled: Option<i64>,
    variant: usize,
}
impl Default for DanceStage {
    fn default() -> Self {
        Self {
            now: 0.,
            bpm: 0.,
            track: 0,
            beat: -1,
            playing: false,
            queue: VecDeque::new(),
            current: None,
            title: Title::default(),
            moves: MoveTiles::default(),
            lights: Lights::default(),
            player: None,
            teacher: None,
            player_watch: Transition::new([0.]),
            teacher_show: Transition::new([0.]),
            watching: false,
            showing: false,
            events: vec![],
            end_title: None,
            play_at: 0.,
            scheduled: None,
            variant: 0,
        }
    }
}
impl DanceStage {
    pub fn set_level(&mut self, level: &Level) {
        self.moves.fade = level.fade_tiles;
        self.moves.set_length(level.steps.len());
        self.lights.ceiling = level.ceiling_lights;
        self.lights.floor = level.floor_lights;
        self.bpm = level.bpm;
        self.track = level.track;
        self.variant = 0;
        self.play_track();
    }
    fn play_track(&mut self) {
        self.scheduled = None;
        self.play_at = self.now;
        self.events
            .push(Event::Track(self.track * 2 + self.variant));
    }
    fn set_variant(&mut self, variant: usize) {
        self.variant = variant;
        self.scheduled = Some((self.beat.div_euclid(4) + 1) * 4);
    }
    pub fn start(
        &mut self,
        queue: Vec<DanceItem>,
        end_title: Option<String>,
        watching: bool,
        freestyle: bool,
    ) {
        if freestyle {
            self.moves.set_length(queue.len().saturating_sub(1));
        }
        self.end_title = end_title;
        self.queue = queue.into();
        self.moves.reset(self.now);
        self.title.set(
            self.now,
            self.queue.front().map(|a| a.title.clone()),
            true,
            0,
        );
        if watching {
            self.watching = true;
            self.showing = true;
            self.player_watch.set(self.now, [1.], 1., Ease::Linear);
            self.teacher_show.set(self.now, [1.], 1., Ease::Linear);
        }
        // Source start does not clear currentAnimation/isPlaying or old watcher
        // classes when interrupting a routine with another routine.
        if self.queue.is_empty() {
            self.finish();
        }
    }
    fn finish(&mut self) {
        // finish dispatch precedes clearing tiles/isPlaying in the source.
        self.events.push(Event::Finish);
        self.set_variant(0);
        self.moves.clear(self.now);
        self.playing = false;
        self.watching = false;
        self.showing = false;
        self.player_watch.set(self.now, [0.], 1., Ease::Linear);
        self.teacher_show.set(self.now, [0.], 1., Ease::Linear);
    }
    pub fn update(&mut self, dt: f64, random: &mut impl FnMut() -> f64) {
        self.now += dt;
        self.moves.update(self.now);
        if let Some(p) = &mut self.player {
            p.update(dt);
        }
        if let Some(p) = &mut self.teacher {
            p.update(dt);
        }
        if self.bpm <= 0. {
            return;
        }
        let mut beat = ((self.now - self.play_at) / (60. / self.bpm))
            .floor()
            .max(0.) as i64;
        if self.scheduled.is_some_and(|at| at <= beat) {
            self.play_track();
            beat = 0;
        } else if self.beat == beat {
            return;
        }
        self.beat = beat;
        self.on_beat(beat, random);
    }
    pub fn on_beat(&mut self, beat: i64, random: &mut impl FnMut() -> f64) {
        self.title.on_beat(self.now);
        let normalized = beat % 4 + 1;
        if normalized == 1 {
            self.current = self.queue.pop_front();
            if let Some(animation) = self.current.clone() {
                if animation.is_countdown {
                    self.playing = true;
                    self.set_variant(1);
                }
                if let Some(tile) = animation.tile {
                    self.moves.add(self.now, tile);
                }
                if animation.teacher_step != "idle" {
                    if animation.player_step == animation.teacher_step {
                        self.events.push(Event::Sound("cb_ingame_win".into()));
                    } else if animation.player_step == "fail" {
                        self.events.push(Event::Sound("cb_ingame_fail".into()));
                    }
                }
                if animation.is_countdown && self.title.count == 0 {
                    self.title.set(self.now, Some(animation.title), true, 2);
                }
                self.events.push(Event::Step(animation.block_id));
            } else if self.playing {
                self.finish();
            }
        }
        if normalized == 4 {
            let next = self.queue.front();
            self.teacher = Some(SpriteClock::new(
                next.map_or("idle".into(), |a| a.teacher_step.clone()),
                self.bpm,
            ));
            self.player = Some(SpriteClock::new(
                next.map_or("idle".into(), |a| a.player_step.clone()),
                self.bpm,
            ));
            if let Some(a) = next {
                self.title.set(
                    self.now,
                    Some(a.title.clone()),
                    a.is_intro,
                    if a.is_countdown { 1 } else { 0 },
                );
            } else {
                self.title.set(self.now, self.end_title.clone(), false, 0);
            }
        }
        if self.current.as_ref().is_some_and(|a| a.is_countdown) {
            self.events
                .push(Event::Sound(format!("cb_count_in_{normalized}")));
        }
        let performance = self.playing && self.current.as_ref().is_some_and(|a| !a.is_countdown);
        self.lights.on_beat(self.now, beat, performance, random);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    fn snapshot(stage: &mut DanceStage, draws: usize) -> Value {
        let clock = |c: &Option<SpriteClock>| {
            c.as_ref().map(|c|json!({"step":c.step,"frame":c.frame,"elapsed":c.elapsed,"duration":c.duration,"count":c.count}))
        };
        let events: Vec<Value> = stage
            .events
            .drain(..)
            .map(|e| match e {
                Event::Track(n) => json!(["track", n]),
                Event::Sound(s) => json!(["sound", s]),
                Event::Step(s) => json!(["step", s]),
                Event::Finish => json!(["finish"]),
            })
            .collect();
        json!({"beat":stage.beat,"playing":stage.playing,"queue":stage.queue,"current":stage.current,
   "title":{"text":stage.title.text,"small":stage.title.small,"count":stage.title.count,"nodes":stage.title.nodes.iter().map(|n|json!({"text":n.text,"small":n.small,"sub":n.sub,"destroyed":n.destroyed.is_some()})).collect::<Vec<_>>()},
   "moves":{"length":stage.moves.length,"fade":stage.moves.fade,"translate":stage.moves.translate_to,"tiles":stage.moves.tiles.iter().map(|n|json!({"step":n.step,"removed":n.removed.is_some()})).collect::<Vec<_>>()},
   "lights":{"order":stage.lights.order,"tiles":stage.lights.tiles,"beams":stage.lights.beam_targets},
   "player":clock(&stage.player),"teacher":clock(&stage.teacher),"watching":stage.watching,"showing":stage.showing,"draws":draws,"events":events})
    }
    fn compare(a: &Value, b: &Value, path: &str) {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => assert!(
                (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() < 1e-6,
                "{path}: {x} != {y}"
            ),
            (Value::Array(x), Value::Array(y)) => {
                assert_eq!(x.len(), y.len(), "{path} length");
                for (i, (x, y)) in x.iter().zip(y).enumerate() {
                    compare(x, y, &format!("{path}[{i}]"));
                }
            }
            (Value::Object(x), Value::Object(y)) => {
                assert_eq!(x.len(), y.len(), "{path} keys: {x:?} vs {y:?}");
                for (k, v) in y {
                    compare(&a[k], v, &format!("{path}.{k}"));
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    #[test]
    fn playback_matches_actual_chromium_source_scenarios() {
        let data: Value =
            serde_json::from_str(include_str!("../data/codeboogie-stage-reference.json")).unwrap();
        let levels = crate::coding::dance_data();
        for case in data["cases"].as_array().unwrap() {
            let level = if case["mode"] == "teacher" {
                &levels.levels[case["level"].as_u64().unwrap() as usize]
            } else {
                &levels.freestyle[case["level"].as_u64().unwrap() as usize]
            };
            let mut stage = DanceStage::default();
            let mut seed = 12345u32;
            let mut draws = 0;
            for (i, action) in case["actions"].as_array().unwrap().iter().enumerate() {
                match action["action"].as_str().unwrap() {
                    "level" => stage.set_level(
                        action["level"]
                            .as_u64()
                            .map_or(level, |i| &levels.levels[i as usize]),
                    ),
                    "start" => stage.start(
                        serde_json::from_value(action["queue"].clone()).unwrap(),
                        action["endTitle"].as_str().map(String::from),
                        action["watching"].as_bool().unwrap(),
                        action["freestyle"].as_bool().unwrap(),
                    ),
                    "update" => stage.update(action["dt"].as_f64().unwrap() / 1000., &mut || {
                        draws += 1;
                        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                        seed as f64 / 4294967296.
                    }),
                    _ => unreachable!(),
                }
                compare(
                    &snapshot(&mut stage, draws),
                    &action["state"],
                    &format!("{} action {i}", case["name"]),
                );
            }
        }
    }
}

/// Geometry from scene.scss. `em` scales with the scene, but CSS pixel borders,
/// rem title offsets and the characters' 480px parent do not scale with it.
pub struct StageLayout {
    pub em: f64,
    pub char_scale: f64,
    pub char_height: f64,
    pub char_origin: [f64; 2],
    pub ceiling: [[f64; 4]; 2],
    pub moves_y: f64,
    pub tile_size: [f64; 2],
    pub tile_stride: f64,
    pub screen_width: f64,
    pub scene: [f64; 4],
}
impl StageLayout {
    pub fn new(screen_width: f64, scene_x: f64, width: f64, height: f64) -> Self {
        let px = |x: f64| (x * 64.).floor() / 64.;
        let ratio = (width / 540.).min(height / 800.);
        let em = ratio * 10.;
        let cw = px(105.3 * em);
        let ch = px(59.1 * em);
        let left = px(width / 2. - 75. * em);
        // Blink's rounded Roboto descent leaves an inline-canvas baseline gap.
        let char_height = 578. + (em * 500. / 2048.).round();
        Self {
            em,
            char_scale: ratio.min(1.),
            char_height,
            char_origin: [scene_x + width / 2., height * 0.78],
            ceiling: [
                [scene_x + left, 0., cw, ch],
                [scene_x + width - left - cw, 0., cw, ch],
            ],
            moves_y: height * (if screen_width >= 1025. { 0.95 } else { 0.3 }) - px(8.5 * em),
            tile_size: [px(11. * em) + 6., px(8.5 * em) + 6.],
            tile_stride: px(11. * em) + 6. + px(0.8 * em),
            screen_width,
            scene: [scene_x, 0., width, height],
        }
    }
    pub fn character(&self, teacher: bool, freestyle: bool, state: f64) -> [f64; 4] {
        let gap = if self.screen_width >= 1024. {
            0.25
        } else {
            0.2
        };
        let scale = if teacher { 1. } else { 1. - 0.14 * state };
        let tx = if teacher {
            (-0.5 + gap * (1. - state)) * 622.
        } else {
            (if freestyle { -0.5 } else { -0.5 - gap }) * 622. + 311. * (1. - scale)
        };
        let y = 72. - self.char_height
            + if teacher {
                0.
            } else {
                -0.1 * self.char_height * state + self.char_height / 2. * (1. - scale)
            };
        [
            self.char_origin[0] + tx * self.char_scale,
            self.char_origin[1] + y * self.char_scale,
            622. * scale * self.char_scale,
            578. * scale * self.char_scale,
        ]
    }
    pub fn tile(&self, index: usize, length: usize, translate_em: f64, scale: f64) -> [f64; 4] {
        let left = self.scene[0]
            + ((self.scene[2] / 2. + (length as f64 * 6.2 + 0.5) * self.em) * 64.).floor() / 64.;
        let x = left + translate_em * self.em + index as f64 * self.tile_stride;
        [
            x + self.tile_size[0] * (1. - scale) / 2.,
            self.moves_y + self.tile_size[1] * (1. - scale) / 2.,
            self.tile_size[0] * scale,
            self.tile_size[1] * scale,
        ]
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn character_lights_and_cards_match_original_dom_at_six_viewports() {
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../data/codeboogie-stage-layout-reference.json"
        ))
        .unwrap();
        for c in data["cases"].as_array().unwrap() {
            let get = |selector: &str, index: usize| -> Vec<f64> {
                c["items"][selector][index]["box"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect()
            };
            let scene = get(".scene", 0);
            let layout =
                StageLayout::new(c["width"].as_f64().unwrap(), scene[0], scene[2], scene[3]);
            let check = |name: &str, native: [f64; 4], original: Vec<f64>| {
                for i in 0..4 {
                    assert!(
                        (native[i] - original[i]).abs() < 0.055,
                        "{}px {} {name}[{i}]: {native:?} vs {original:?}",
                        c["width"],
                        c["watching"]
                    );
                }
            };
            let state = if c["watching"].as_bool().unwrap() {
                1.
            } else {
                0.
            };
            check(
                "teacher canvas",
                layout.character(true, false, state),
                get("canvas", 0),
            );
            check(
                "player canvas",
                layout.character(false, false, state),
                get("canvas", 1),
            );
            for i in 0..2 {
                check("ceiling", layout.ceiling[i], get(".ceilingLight", i));
            }
            for i in 0..5 {
                check(
                    "move card",
                    layout.tile(i, 4, -62., if i == 0 { 1.5 } else { 1. }),
                    get(".scene__moves-move", i),
                );
            }
        }
    }
}

#[cfg(test)]
mod phase_tests {
    use super::*;
    #[test]
    fn active_effect_phases_match_the_original_css_engine() {
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../data/codeboogie-stage-phase-reference.json"
        ))
        .unwrap();
        for row in data["cases"].as_array().unwrap() {
            let t = row["t"].as_f64().unwrap();
            let kind = row["kind"].as_str().unwrap();
            let (opacity, scale, x, y) = match kind {
                "title-appear" | "title-destroy" => {
                    let n = TitleNode {
                        text: "x".into(),
                        small: true,
                        sub: false,
                        born: 0.,
                        destroyed: (kind == "title-destroy").then_some(0.),
                    };
                    let (opacity, scale) = n.appearance(t, 140.);
                    (opacity, scale, 0., 0.)
                }
                "tile-appear" | "tile-disappear" => {
                    let n = MoveTile {
                        step: "jump".into(),
                        born: 0.,
                        removed: (kind == "tile-disappear").then_some(0.),
                    };
                    let (opacity, scale) = n.appearance(t);
                    (opacity, scale, 0., 0.)
                }
                "beam" => {
                    let mut f = Transition::new([0.]);
                    f.set(0., [1.], 0.1, Ease::Default);
                    (f.value(t)[0], 1., 0., 0.)
                }
                "tile-shift" => {
                    let mut m = MoveTiles::default();
                    m.add(0., "pointLeft".into());
                    m.add(1., "jump".into());
                    (1., 1., m.translate.value(1. + t)[0] * 10., 0.)
                }
                "player-watch" => {
                    let p = t.clamp(0., 1.);
                    let layout = StageLayout::new(1440., 720., 720., 800.);
                    (1., 1. - 0.14 * p, -466.5, -0.1 * layout.char_height * p)
                }
                "teacher-show" => (1., 1., -155.5 - 155.5 * t.clamp(0., 1.), 0.),
                _ => unreachable!(),
            };
            for (key, native) in [("opacity", opacity), ("scale", scale), ("x", x), ("y", y)] {
                // CSS matrix() serialization keeps six significant digits.
                let tolerance = if key == "x" || key == "y" {
                    0.001
                } else {
                    0.00002
                };
                assert!(
                    (native - row[key].as_f64().unwrap()).abs() < tolerance,
                    "{kind} t={t} {key}: {native} != {}",
                    row[key]
                );
            }
        }
    }
}
