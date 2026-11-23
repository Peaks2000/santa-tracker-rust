//! Direct behavioral port of static/scenes/jetpack/js (Google, Apache-2.0).
use crate::random::Random;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
pub struct ItemType {
    pub css: String,
    pub weight: f64,
    #[serde(default)]
    pub fast: bool,
    #[serde(default)]
    pub faster: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Kind {
    Score,
    Clock,
    Danger,
    Cloud,
}
#[derive(Clone, Serialize)]
pub struct Entity {
    pub kind: Kind,
    pub style: String,
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub fast: bool,
    pub flow: f64,
    pub hit: bool,
    pub message: String,
    pub hit_at: f64,
    pub born: f64,
    pub cloud_type: usize,
}
#[derive(Serialize)]
pub struct Player {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub shake: f64,
    pub hit: bool,
    pub hit_duration: f64,
    pub going_up: bool,
    pub fire_scale: f64,
    pub rotation: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct PlayerSound {
    pub name: &'static str,
    pub args: [f64; 1],
}
pub struct Jetpack {
    pub player: Player,
    pub entities: Vec<Entity>,
    pub level: u32,
    pub score: i64,
    pub countdown: f64,
    pub last_seconds: f64,
    pub playing: bool,
    pub paused: bool,
    pub finished: bool,
    pub time: f64,
    pub width: f64,
    pub height: f64,
    pub transition: Option<f64>,
    pub background_level: u32,
    pub goal_time: f64,
    pub goal_target_y: f64,
    wall_time: f64,
    goal_deadlines: Vec<f64>,
    pub sounds: Vec<&'static str>,
    pub player_sounds: Vec<PlayerSound>,
    sound_counter: u8,
    sound_velocity: Option<f64>,
    sound_position: Option<f64>,
    random: Random,
    next_item: f64,
    next_danger: f64,
    next_clock: f64,
    next_cloud: f64,
    outside: bool,
    next_level: f64,
}
impl Jetpack {
    pub fn new(seed: &str, width: f64, height: f64) -> Self {
        Self {
            player: Player {
                x: 490.,
                y: height * 0.75,
                vx: 0.,
                vy: 0.,
                shake: 0.,
                hit: false,
                hit_duration: 0.,
                going_up: false,
                fire_scale: 1.,
                rotation: 0.,
            },
            entities: vec![],
            level: 0,
            score: 0,
            countdown: 60.,
            last_seconds: f64::NAN,
            playing: true,
            paused: false,
            finished: false,
            time: 0.,
            width,
            height,
            transition: None,
            background_level: 0,
            goal_time: 0.,
            goal_target_y: 0.,
            wall_time: 0.,
            goal_deadlines: vec![],
            sounds: vec!["music_start_ingame", "jp_start_game"],
            player_sounds: vec![
                PlayerSound {
                    name: "jp_player_moving",
                    args: [0.],
                },
                PlayerSound {
                    name: "jp_player_position",
                    args: [0.],
                },
            ],
            sound_counter: 1,
            sound_velocity: Some(0.),
            sound_position: Some(0.),
            random: Random::seeded(seed),
            next_item: 0.,
            next_danger: 0.,
            next_clock: 0.,
            next_cloud: 0.,
            outside: false,
            next_level: 21.,
        }
    }
    pub fn restart(&mut self) {
        // Original reset hides the goal but never cancels its setTimeout.
        let mut fresh = Self::new("", self.width, self.height);
        fresh.random = self.random.clone();
        fresh.wall_time = self.wall_time;
        fresh.goal_deadlines = std::mem::take(&mut self.goal_deadlines);
        let mut sounds = std::mem::take(&mut self.sounds);
        sounds.append(&mut fresh.sounds);
        fresh.sounds = sounds;
        let mut player_sounds = std::mem::take(&mut self.player_sounds);
        player_sounds.append(&mut fresh.player_sounds);
        fresh.player_sounds = player_sounds;
        *self = fresh;
    }
    pub fn wall_step(&mut self, dt: f64) {
        self.wall_time += dt;
        if self.finished {
            self.goal_time += dt;
        }
        let mut i = 0;
        while i < self.goal_deadlines.len() {
            if self.wall_time >= self.goal_deadlines[i] {
                self.goal_deadlines.remove(i);
                self.gameover();
            } else {
                i += 1;
            }
        }
    }
    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }
    pub fn advance(&mut self, duration: f64, input: (f64, f64)) {
        let mut left = duration;
        while left > 0. {
            let dt = left.min(1. / 60.);
            self.wall_step(dt);
            self.step(dt, input);
            left -= dt;
        }
    }
    pub fn step(&mut self, dt: f64, mut input: (f64, f64)) {
        if !self.playing || self.paused {
            return;
        }
        self.time += dt;
        if let Some(progress) = &mut self.transition {
            *progress += dt / 5.;
            if *progress > 1. {
                self.transition = None;
                self.background_level += 1;
            }
        }
        let length = (input.0 * input.0 + input.1 * input.1).sqrt();
        if length > 1. {
            input.0 /= length;
            input.1 /= length;
        }
        let min_x = 490. - self.width / 2. + 58.;
        let max_x = min_x + self.width - 116.;
        let outside = self.player.y < 70.
            || self.player.y > self.height - 80.
            || self.player.x < min_x
            || self.player.x > max_x;
        if outside != self.outside {
            self.sounds.push(if outside {
                "jp_out_of_screen"
            } else {
                "jp_back_to_screen"
            });
            self.outside = outside;
        }
        if self.player.y < 70. {
            input.1 = 1.;
        } else if self.player.y > self.height - 80. {
            input.1 = -1.;
        }
        if self.player.x < min_x {
            input.0 = 1.;
        } else if self.player.x > max_x {
            input.0 = -1.;
        }
        if self.player.hit {
            self.player.hit_duration += dt;
            input = (0., 0.);
            if self.player.hit_duration >= 1. {
                self.player.hit = false;
                self.player.hit_duration = 0.;
            }
        }
        fn approach(n: f64, t: f64, a: f64) -> f64 {
            if n < t {
                (n + a).min(t)
            } else if n > t {
                (n - a).max(t)
            } else {
                n
            }
        }
        self.player.vx = approach(self.player.vx, input.0, dt * 3.);
        self.player.vy = approach(self.player.vy, input.1, dt * 3.);
        self.player.x += self.player.vx * 900. * dt;
        self.player.y += self.player.vy * 900. * dt;
        self.player.shake += dt;
        self.player.x += (self.player.shake * 3.).sin() * 15. * dt;
        self.player.y += (self.player.shake * 4.).sin() * 15. * dt;
        self.player.rotation = self.player.vx * 15.;
        self.player.fire_scale = 1.25f64.powf(-input.1);
        self.player.going_up = input.1 == -1.;
        for i in 0..self.entities.len() {
            let e = &self.entities[i];
            if e.kind != Kind::Cloud
                && !e.hit
                && (e.x - self.player.x).powi(2) + (e.y - self.player.y).powi(2) < 75. * 75.
            {
                self.collect(i);
            }
        }
        if self.sound_counter == 0 {
            let velocity =
                (self.player.vx * self.player.vx + self.player.vy * self.player.vy).sqrt();
            if self.sound_velocity != Some(velocity) {
                self.sound_velocity = Some(velocity);
                self.player_sounds.push(PlayerSound {
                    name: "jp_player_moving",
                    args: [velocity],
                });
            }
            let panning = (self.player.x - 490.) / (self.width / 2.) * 0.85;
            if self.sound_position != Some(panning) {
                self.sound_position = Some(panning);
                self.player_sounds.push(PlayerSound {
                    name: "jp_player_position",
                    args: [panning],
                });
            }
        }
        self.sound_counter = (self.sound_counter + 1) % 3;
        if !self.finished {
            self.countdown -= dt;
            let second = self.countdown.ceil();
            if second != self.last_seconds {
                self.last_seconds = second;
                if self.countdown < 0. {
                    self.countdown = 0.;
                    self.gameover();
                }
            }
            self.next_item -= dt;
            if self.next_item <= 0. {
                self.spawn(Kind::Score);
                self.next_item = (1. + self.random.next() * 2.) * 0.95f64.powi(self.level as i32);
            }
            self.next_danger -= dt;
            if self.level >= 2 && self.next_danger <= 0. {
                self.spawn(Kind::Danger);
                self.next_danger = 10. / (self.level as f64).powf(1.2) + self.random.next() * 3.;
            }
            self.next_clock -= dt;
            if self.next_clock <= 0. {
                self.spawn(Kind::Clock);
                self.next_clock = (9. + self.random.next() * 2.) * 0.95f64.powi(self.level as i32);
            }
            self.next_cloud -= dt;
            if self.next_cloud <= 0. {
                self.spawn(Kind::Cloud);
                self.next_cloud = 0.45 + self.random.next() * 2.5;
            }
        }
        self.next_level -= dt;
        if self.next_level <= 0. {
            self.level += 1;
            if self.level == 10 {
                self.finished = true;
                self.goal_time = 0.;
                self.goal_target_y = self.height - 412. - self.height * 10. / 100.;
                self.goal_deadlines.push(self.wall_time + 3.5);
            } else {
                self.transition = Some(0.);
                self.next_level = 21.;
            }
        }
        for e in &mut self.entities {
            e.y += e.speed * dt;
            e.x += e.flow * e.speed * dt;
        }
        self.entities.retain(|e| {
            if e.kind == Kind::Cloud {
                e.y < self.height + 150.
            } else {
                e.y < self.height + 150. && e.x > -200. && e.x < self.width + 200.
            }
        });
    }
    fn collect(&mut self, i: usize) {
        let e = &mut self.entities[i];
        e.hit = true;
        e.hit_at = self.time;
        match e.kind {
            Kind::Score => {
                let score = (if e.fast { 75 } else { 50 }) * (self.level + 1);
                self.score += score as i64;
                e.message = score.to_string();
                self.sounds.push(if e.style.contains("present") {
                    "jp_pickup_parachute_item"
                } else {
                    "jp_pickup_item"
                });
            }
            Kind::Clock => {
                let time = if e.fast { 10. } else { 5. };
                self.countdown += time;
                e.message = format!("+ 00:{:02}", time as u32);
                self.sounds.push("jetpack_clock");
            }
            Kind::Danger => {
                let score = -((self.score as f64 * self.random.next() * 0.2).trunc() as i64);
                let time = -((self.last_seconds * self.random.next() * 0.2).trunc());
                self.score += score;
                self.countdown += time;
                e.message = score.to_string();
                self.player.hit = true;
                self.sounds.push("jetpack_hit");
            }
            Kind::Cloud => {}
        }
    }
    fn spawn(&mut self, kind: Kind) {
        let mut e = Entity {
            kind,
            style: String::new(),
            x: 0.,
            y: -150.,
            speed: 0.,
            fast: false,
            flow: 0.,
            hit: false,
            message: String::new(),
            hit_at: 0.,
            born: self.time,
            cloud_type: 0,
        };
        if kind == Kind::Cloud {
            e.cloud_type = (self.random.next() * 3.).floor() as usize;
            e.speed = 120. - e.cloud_type as f64 * 30.;
            e.x = self.random.next() * self.width;
        } else {
            let data: serde_json::Value =
                serde_json::from_str(include_str!("../data/jetpack-constants.json"))
                    .expect("original constants");
            let key = match kind {
                Kind::Clock => "CLOCK_TYPES",
                Kind::Danger => "DANGER_TYPES",
                _ => "ITEM_TYPES",
            };
            let types: Vec<ItemType> = serde_json::from_value(data[key].clone()).unwrap();
            let total: f64 = types.iter().map(|t| t.weight).sum();
            let mut weight = self.random.next() * total;
            let ty = types
                .iter()
                .find(|ty| {
                    weight -= ty.weight;
                    weight < 0.
                })
                .unwrap();
            e.style = ty.css.clone();
            e.fast = ty.fast;
            e.speed = if ty.faster {
                225.
            } else if ty.fast {
                150.
            } else {
                75.
            };
            e.speed *= 1.15f64.powi(self.level as i32);
            e.x = self.random.next() * 980.;
            if kind == Kind::Danger {
                e.speed *= 1.5;
                if e.style.ends_with('3') {
                    e.x = 1130.;
                    e.y = self.random.next() * 0.2 * 600.;
                    e.flow = -1.;
                } else if e.style.ends_with('4') {
                    e.x = -150.;
                    e.y = self.random.next() * 0.2 * 600.;
                    e.flow = 1.;
                }
            }
        }
        self.entities.push(e);
    }
    pub fn gameover(&mut self) {
        // Source callback fires sound/host notification even if already frozen.
        self.playing = false;
        self.sounds
            .extend(["jp_game_over", "music_ingame_gameover"]);
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"x":self.player.x,"y":self.player.y,"vx":self.player.vx,"vy":self.player.vy,"shake":self.player.shake,"hit":self.player.hit,"hit_duration":self.player.hit_duration,"going_up":self.player.going_up,"level":self.level,"score":self.score,"countdown":self.countdown,"playing":self.playing,"entities":self.entities.iter().map(|e|serde_json::json!({"kind":e.kind,"style":e.style,"x":e.x,"y":e.y,"speed":e.speed,"hit":e.hit})).collect::<Vec<_>>()})
    }
}

/// Original goal CSS background: 412x452 artwork, contain/top-right inside
/// a 452x412 box capped at half the viewport. It is outside the scaled scene.
pub fn goal_image_rect(viewport_width: f64, top: f64) -> [f64; 4] {
    let box_width = 452f64.min(viewport_width / 2.);
    let fit = (box_width / 412.).min(412. / 452.);
    let width = 412. * fit;
    [viewport_width - width, top, width, 452. * fit]
}

#[cfg(test)]
mod goal_tests {
    use super::*;
    #[test]
    fn original_player_frames_match_variable_intervals_and_hit_expiry() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/jetpack-player-frames.json")).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let mut game = Jetpack::new("player-frames", 1440., 800.);
            game.next_item = f64::INFINITY;
            game.next_danger = f64::INFINITY;
            game.next_clock = f64::INFINITY;
            game.next_cloud = f64::INFINITY;
            game.next_level = f64::INFINITY;
            let mut keys = [false; 4];
            game.player_sounds.clear();
            for frame in case["frames"].as_array().unwrap() {
                let input = &frame["input"];
                if input["keys"].is_array() {
                    keys = serde_json::from_value(input["keys"].clone()).unwrap();
                }
                if let Some(place) = input["place"].as_array() {
                    game.player.x = place[0].as_f64().unwrap();
                    game.player.y = place[1].as_f64().unwrap();
                }
                if input["hit"] == true {
                    game.player.hit = true;
                }
                game.step(
                    frame["delta"].as_f64().unwrap(),
                    (
                        f64::from(keys[2]) - f64::from(keys[0]),
                        f64::from(keys[3]) - f64::from(keys[1]),
                    ),
                );
                let actual = serde_json::to_value(&game.player).unwrap();
                let expected = &frame["output"];
                for key in [
                    "x",
                    "y",
                    "vx",
                    "vy",
                    "shake",
                    "hit_duration",
                    "fire_scale",
                    "rotation",
                ] {
                    assert!(
                        (actual[key].as_f64().unwrap() - expected[key].as_f64().unwrap()).abs()
                            < 1e-9,
                        "{} {input}: {key} {} != {}",
                        case["name"],
                        actual[key],
                        expected[key]
                    );
                }
                for key in ["hit", "going_up"] {
                    assert_eq!(actual[key], expected[key], "{} {input}", case["name"]);
                }
                let source_parameters: Vec<_> = frame["sound_events"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|e| &e["payload"])
                    .filter(|e| e["name"].is_string())
                    .collect();
                assert_eq!(
                    game.player_sounds.len(),
                    source_parameters.len(),
                    "{} {input}",
                    case["name"]
                );
                for (actual, source) in game.player_sounds.drain(..).zip(source_parameters) {
                    assert_eq!(actual.name, source["name"].as_str().unwrap());
                    assert!(
                        (actual.args[0] - source["args"][0].as_f64().unwrap()).abs() < 1e-9,
                        "{} {input}",
                        case["name"]
                    );
                }
                let boundary: Vec<_> = game
                    .sounds
                    .drain(..)
                    .filter(|s| matches!(*s, "jp_out_of_screen" | "jp_back_to_screen"))
                    .collect();
                let source: Vec<_> = frame["sound_events"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|e| e["payload"].as_str())
                    .filter(|s| matches!(*s, "jp_out_of_screen" | "jp_back_to_screen"))
                    .collect();
                assert_eq!(boundary, source, "{} {input}", case["name"]);
            }
        }
    }

    #[test]
    fn source_goal_wall_callbacks_survive_pause_restart_and_prior_gameover() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../data/jetpack-goal-clock-reference.json"))
                .unwrap();
        for case in reference["cases"].as_array().unwrap() {
            let mut game = Jetpack::new("goal-clock", 1440., 800.);
            game.sounds.clear();
            let mut gameovers = 0;
            for frame in case["frames"].as_array().unwrap() {
                let action = frame["action"].as_array().unwrap();
                match action[0].as_str().unwrap() {
                    "finish" => {
                        game.level = 9;
                        game.next_level = 0.;
                        game.step(0., (0., 0.));
                    }
                    "wall" => game.wall_step(action[1].as_f64().unwrap() / 1000.),
                    "pause" => game.paused = true,
                    "resume" => game.paused = false,
                    "restart" => game.restart(),
                    "gameover" => game.gameover(),
                    _ => unreachable!(),
                }
                let sounds: Vec<_> = game.sounds.drain(..).collect();
                gameovers += sounds.iter().filter(|s| **s == "jp_game_over").count();
                let output = serde_json::json!({"running":game.playing && !game.paused,"paused":game.paused,
                    "finished":game.finished,"level":game.level,"pending":game.goal_deadlines.len(),"gameovers":gameovers});
                assert_eq!(output, frame["output"], "{} {action:?}", case["name"]);
                assert_eq!(
                    serde_json::json!(sounds),
                    frame["sounds"],
                    "{} {action:?}",
                    case["name"]
                );
            }
        }
    }

    #[test]
    fn source_goal_target_and_contain_fit_survive_resize() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/jetpack-goal-reference.json")).unwrap();
        for case in data["cases"].as_array().unwrap() {
            let n = |v: &serde_json::Value| v.as_f64().unwrap();
            let mut game = Jetpack::new(
                "goal-layout",
                n(&case["scene"]["width"]),
                n(&case["scene"]["height"]),
            );
            game.level = 9;
            game.next_level = 0.;
            game.step(1. / 60., (0., 0.));
            assert!(game.finished);
            assert_eq!(game.goal_target_y, n(&case["target"]));
            for (viewport, expected) in [
                (n(&case["width"]), &case["rect"]),
                (n(&case["resized"]["width"]), &case["resized"]["rect"]),
            ] {
                let rect = goal_image_rect(viewport, game.goal_target_y);
                for (actual, expected) in rect.into_iter().zip(expected.as_array().unwrap()) {
                    assert!(
                        (actual - n(expected)).abs() < 1e-10,
                        "{case}: {actual} != {expected}"
                    );
                }
            }
            game.resize(
                n(&case["resized"]["scene"]["width"]),
                n(&case["resized"]["scene"]["height"]),
            );
            assert_eq!(game.goal_target_y, n(&case["resized"]["target"]));
            assert_eq!(case["timers"], serde_json::json!([3500]));
            assert_eq!(case["hidden"], true);
            assert_eq!(case["shown"], true);
        }
    }
}
