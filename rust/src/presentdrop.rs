//! Direct port of every active Present Drop game/entity/level function.
//! Original Google source Apache-2.0. Browser glue is handled by the native view.
use crate::{
    random::Random,
    ui::{Ease, Transition},
};
use serde::Serialize;
use std::collections::VecDeque;
#[derive(Clone, Serialize)]
pub struct Chimney {
    pub x: f64,
    pub large: bool,
    pub hits: u32,
    pub dead: bool,
    pub flag_score: i64,
    pub flag_until: f64,
    #[serde(skip)]
    pub flag: Transition<1>,
}
impl Default for Chimney {
    fn default() -> Self {
        Self {
            x: 0.,
            large: false,
            hits: 0,
            dead: true,
            flag_score: 0,
            flag_until: 0.,
            flag: Transition::new([0.]),
        }
    }
}
#[derive(Clone, Serialize)]
pub struct Present {
    pub x: f64,
    pub y: f64,
    pub velocity: f64,
    pub dead: bool,
}
#[derive(Clone, Copy, Serialize)]
pub enum Entity {
    Chimney(usize),
    Present(usize),
}
pub struct PresentDrop {
    pub x: f64,
    pub target_x: f64,
    pub moving: bool,
    pub pull_back: Option<bool>,
    pub drop_when_stopped: bool,
    pub has_present: bool,
    pub sound_frame: u32,
    pub chimneys: Vec<Chimney>,
    pub presents: Vec<Present>,
    pub entities: Vec<Entity>,
    chimney_pool: VecDeque<usize>,
    present_pool: VecDeque<usize>,
    ready_at: Vec<f64>,
    pub score: i64,
    pub countdown: f64,
    pub level: u32,
    pub chimney_speed: f64,
    pub next_chimney: f64,
    pub fg_right: f64,
    pub bg_right: f64,
    pub bg_target: f64,
    pub fg_loops: u32,
    pub fg_last: u32,
    pub fg_next: u32,
    pub time: f64,
    pub wall_time: f64,
    pub playing: bool,
    pub paused: bool,
    pub drop_at: f64,
    pub sounds: Vec<&'static str>,
    pub pan_events: Vec<f64>,
    last_seconds: Option<i64>,
    losing: bool,
    random: Random,
}
impl PresentDrop {
    pub fn new(seed: &str) -> Self {
        let mut game = Self {
            x: 450.,
            target_x: 450.,
            moving: false,
            pull_back: None,
            drop_when_stopped: false,
            has_present: true,
            sound_frame: 0,
            chimneys: vec![Chimney::default(); 7],
            presents: vec![
                Present {
                    x: 0.,
                    y: 0.,
                    velocity: 0.,
                    dead: true
                };
                2
            ],
            entities: vec![],
            chimney_pool: (0..7).collect(),
            present_pool: (0..2).collect(),
            ready_at: vec![],
            score: 0,
            countdown: 60.,
            level: 0,
            chimney_speed: 150.,
            next_chimney: 0.,
            fg_right: 0.,
            bg_right: 0.,
            bg_target: 0.,
            fg_loops: 0,
            fg_last: 0,
            fg_next: 0,
            time: 0.,
            wall_time: 0.,
            playing: true,
            paused: false,
            drop_at: f64::NEG_INFINITY,
            sounds: vec![],
            pan_events: vec![],
            last_seconds: None,
            losing: false,
            random: Random::seeded(seed),
        };
        game.restart();
        game
    }
    pub fn restart(&mut self) {
        // Pool instances and their sticky `large` class survive restart in source.
        for e in std::mem::take(&mut self.entities) {
            match e {
                Entity::Chimney(i) => {
                    self.chimneys[i].dead = true;
                    self.chimney_pool.push_back(i);
                }
                Entity::Present(i) => {
                    self.presents[i].dead = true;
                    self.present_pool.push_back(i);
                }
            }
        }
        self.x = 450.;
        self.target_x = 450.;
        self.moving = false;
        self.pull_back = None;
        self.drop_when_stopped = false;
        self.has_present = true;
        self.sound_frame = 0;
        self.score = 0;
        self.countdown = 60.;
        self.level = 0;
        self.chimney_speed = 150.;
        self.next_chimney = 0.;
        self.fg_right = 0.;
        self.bg_right = 0.;
        self.bg_target = 0.;
        self.fg_loops = 0;
        self.fg_last = 0;
        self.fg_next = 0;
        self.paused = false;
        self.playing = true;
        self.last_seconds = None;
        self.losing = false;
        self.sounds.extend(["music_start_ingame", "pd_start_game"]);
    }
    pub fn set_x(&mut self, x: f64) {
        let x = if x < 225. {
            self.sounds.push("pd_player_crash");
            225.
        } else if x > 680. {
            self.sounds.push("pd_player_crash");
            680.
        } else {
            x
        };
        self.target_x = x;
        if x != self.x && Some(x > self.x) != self.pull_back {
            self.pull_back = Some(x > self.x);
        }
    }
    pub fn keyboard(&mut self, left: bool, right: bool) {
        self.set_x(if right {
            680.
        } else if left {
            225.
        } else {
            self.x
        });
    }
    pub fn touch_ended(&mut self) {
        if self.x == self.target_x {
            self.drop_present();
        } else {
            self.drop_when_stopped = true;
        }
    }
    pub fn drop_present(&mut self) {
        if !self.has_present {
            return;
        }
        let id = self.present_pool.pop_front().unwrap_or_else(|| {
            let i = self.presents.len();
            self.presents.push(Present {
                x: 0.,
                y: 0.,
                velocity: 0.,
                dead: true,
            });
            i
        });
        self.presents[id] = Present {
            x: self.x - 10.,
            y: 102.,
            velocity: 0.,
            dead: false,
        };
        self.entities.push(Entity::Present(id));
        self.has_present = false;
        self.drop_at = self.time;
        self.ready_at.push(self.wall_time + 0.5);
        self.sounds.push("pd_item_drop");
    }
    pub fn gameover(&mut self) {
        self.playing = false;
        self.sounds
            .extend(["pd_game_over", "music_ingame_gameover"]);
    }
    fn next_level(&mut self) {
        if self.level == 9 {
            self.gameover();
            return;
        }
        self.level += 1;
        self.chimney_speed += 30.;
        self.sounds.push("pd_player_level_up");
    }
    fn player_frame(&mut self, delta: f64) {
        if self.moving != (self.target_x != self.x) {
            self.moving = !self.moving;
            self.sounds.push(if self.moving {
                "pd_pull_start"
            } else {
                "pd_pull_stop"
            });
        }
        if !self.moving {
            return;
        }
        let moved = delta * 400.;
        if moved > (self.target_x - self.x).abs() {
            self.x = self.target_x;
            if self.drop_when_stopped {
                self.drop_when_stopped = false;
                self.drop_present();
            }
        } else {
            self.x += moved * if self.target_x > self.x { 1. } else { -1. };
        }
        if self.sound_frame == 0 {
            self.sounds.push("pd_sled_position");
            self.pan_events.push((self.x - 225.) / 455. * 2. - 1.);
        }
        self.sound_frame = (self.sound_frame + 1) % 3;
    }
    fn levels_frame(&mut self, delta: f64) {
        let mut next = false;
        let fg = self.fg_right + (932. / 7.) * delta;
        if fg > 932. {
            self.fg_last = self.fg_next;
            self.fg_loops = (self.fg_loops + 1) % 3;
            next = self.fg_loops == 0;
            if next {
                self.fg_next = self.level % 4;
                self.bg_target = self.bg_right + 890.;
            }
        }
        self.fg_right = fg % 932.;
        if self.bg_right != self.bg_target {
            self.bg_right += (890. / 2.) * delta;
            if self.bg_right > self.bg_target {
                let bg = if (self.level as i64 - 1) % 4 != 0 {
                    self.bg_target
                } else {
                    0.
                };
                self.bg_right = bg;
                self.bg_target = bg;
            }
        }
        if next {
            self.next_level();
        }
    }
    fn scoreboard_frame(&mut self, delta: f64) {
        self.countdown -= delta;
        let seconds = self.countdown.ceil() as i64;
        if self.last_seconds == Some(seconds) {
            return;
        }
        self.last_seconds = Some(seconds);
        if delta > 0. {
            if self.countdown < 0. {
                self.countdown = 0.;
                self.gameover();
            }
            let losing = seconds <= 10 && seconds != 0;
            if self.losing != losing {
                self.losing = losing;
                if seconds > 0 {
                    self.sounds.push(if losing {
                        "game_hurry_up"
                    } else {
                        "game_hurry_up_end"
                    });
                }
            }
        }
    }
    fn spawn_frame(&mut self, delta: f64) {
        self.next_chimney -= delta;
        if self.next_chimney > 0. {
            return;
        }
        let id = self.chimney_pool.pop_front().unwrap_or_else(|| {
            let i = self.chimneys.len();
            self.chimneys.push(Chimney::default());
            i
        });
        let c = &mut self.chimneys[id];
        c.dead = false;
        c.hits = 0;
        c.x = 1000.;
        if self.random.next() < 0.5 {
            c.large = true;
        }
        c.flag.set(self.wall_time, [0.], 0.2, Ease::Default);
        self.entities.push(Entity::Chimney(id));
        let m = 0.8f64.powi(self.level as i32);
        self.next_chimney = 0.5 + 3. * m - 2. * m / 2. + self.random.next() * 2. * m;
    }
    fn hit(&mut self, id: usize) {
        let c = &mut self.chimneys[id];
        c.hits += 1;
        let base = if c.large { 10 } else { 20 };
        let score = base * (1 + self.level as i64) * 2i64.pow(c.hits);
        c.flag_score = score;
        c.flag_until = self.wall_time + 0.6;
        c.flag.set(self.wall_time, [-55.], 0.2, Ease::Default);
        self.score += score;
        self.countdown += 1.;
        self.sounds.push("pd_player_present_pickup");
    }
    pub fn step(&mut self, delta: f64) {
        // setTimeout-based cooldown/flag transitions keep running while paused.
        self.wall_time += delta;
        let mut ready = false;
        self.ready_at.retain(|at| {
            if *at <= self.wall_time {
                ready = true;
                false
            } else {
                true
            }
        });
        if ready {
            self.has_present = true;
        }
        for c in &mut self.chimneys {
            if c.flag_until > 0. && self.wall_time >= c.flag_until {
                c.flag_until = 0.;
                c.flag.set(self.wall_time, [0.], 0.2, Ease::Default);
            }
        }
        if !self.playing || self.paused {
            return;
        }
        self.time += delta;
        self.player_frame(delta);
        self.levels_frame(delta);
        self.scoreboard_frame(delta);
        self.spawn_frame(delta);
        // Entity order matters: newly spawned chimneys may move before/after a
        // falling present depending on when each was inserted into this array.
        for entity in self.entities.clone() {
            match entity {
                Entity::Chimney(i) => {
                    let c = &mut self.chimneys[i];
                    c.x -= self.chimney_speed * delta;
                    if c.x <= -300. {
                        c.dead = true;
                        self.chimney_pool.push_back(i);
                    }
                }
                Entity::Present(i) => {
                    let old = self.presents[i].y;
                    if old < 660. {
                        self.presents[i].velocity += 2000. * delta;
                        self.presents[i].y = (old + self.presents[i].velocity * delta).min(660.);
                    } else {
                        self.presents[i].dead = true;
                        self.present_pool.push_back(i);
                        self.sounds.push("pd_item_miss");
                    }
                    for e in self.entities.clone() {
                        if let Entity::Chimney(j) = e {
                            let c = &self.chimneys[j];
                            let (center, y, x) = if c.large {
                                (45., 497., c.x + 25.)
                            } else {
                                (35., 524., c.x + 35.)
                            };
                            if y <= old || y >= self.presents[i].y {
                                continue;
                            }
                            let diff = (self.presents[i].x - x - center).abs();
                            if diff < center + 10. {
                                self.presents[i].dead = true;
                                self.present_pool.push_back(i);
                                self.hit(j);
                            }
                        }
                    }
                }
            }
        }
        self.entities.retain(|e| match e {
            Entity::Chimney(i) => !self.chimneys[*i].dead,
            Entity::Present(i) => !self.presents[*i].dead,
        });
    }
    pub fn advance(&mut self, duration: f64) {
        let mut left = duration;
        while left > 0. && self.playing {
            let dt = left.min(1. / 60.);
            self.step(dt);
            left -= dt;
        }
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"x":self.x,"target":self.target_x,"moving":self.moving,"pull_back":self.pull_back,"has_present":self.has_present,"drop_when_stopped":self.drop_when_stopped,"score":self.score,"countdown":self.countdown,"level":self.level,"speed":self.chimney_speed,"next_chimney":self.next_chimney,"fg_right":self.fg_right,"bg_right":self.bg_right,"bg_target":self.bg_target,"fg_loops":self.fg_loops,"fg_last":self.fg_last,"fg_next":self.fg_next,"playing":self.playing,"entities":self.entities.iter().map(|e|match e{Entity::Chimney(i)=>serde_json::json!({"kind":"chimney","x":self.chimneys[*i].x,"large":self.chimneys[*i].large,"hits":self.chimneys[*i].hits}),Entity::Present(i)=>serde_json::json!({"kind":"present","x":self.presents[*i].x,"y":self.presents[*i].y,"velocity":self.presents[*i].velocity})}).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_original_javascript_entities_pools_levels_and_wall_timers() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../data/presentdrop-reference.json")).unwrap();
        fn compare(a: &serde_json::Value, b: &serde_json::Value, p: &str) {
            match (a, b) {
                (serde_json::Value::Number(a), serde_json::Value::Number(b)) => assert!(
                    (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-7,
                    "{p}: {a} != {b}"
                ),
                (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                    assert_eq!(a.len(), b.len(), "{p}");
                    for (i, (a, b)) in a.iter().zip(b).enumerate() {
                        compare(a, b, &format!("{p}/{i}"));
                    }
                }
                (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                    for (k, b) in b {
                        compare(&a[k], b, &format!("{p}/{k}"));
                    }
                }
                _ => assert_eq!(a, b, "{p}"),
            }
        }
        for case in cases.as_array().unwrap() {
            let mut g = PresentDrop::new("test1234");
            for (i, step) in case["steps"].as_array().unwrap().iter().enumerate() {
                match step["op"].as_str().unwrap() {
                    "left" => g.set_x(225.),
                    "right" => g.set_x(680.),
                    "stop" => g.set_x(g.x),
                    "set-x" => g.set_x(step["arg"].as_f64().unwrap()),
                    "drop" => g.drop_present(),
                    "touch-end" => g.touch_ended(),
                    "pause" => g.paused = true,
                    "resume" => g.paused = false,
                    "restart" => g.restart(),
                    _ => {}
                }
                let mut left = step["duration"].as_f64().unwrap();
                while left > 0. && (g.playing || g.paused) {
                    let dt = left.min(1. / 60.);
                    g.step(dt);
                    left -= dt;
                }
                let mut actual = g.snapshot();
                if g.paused {
                    actual["playing"] = serde_json::json!(false);
                }
                compare(&actual, &step["state"], &format!("{}/{i}", case["name"]));
            }
        }
    }
}
