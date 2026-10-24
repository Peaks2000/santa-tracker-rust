//! Native translation of Penguin Dash's Start/Penguin functions and used Arcade rules.
use crate::random::Random;
use serde_json::{json, Value};
use std::sync::OnceLock;
fn config() -> &'static Value {
    static C: OnceLock<Value> = OnceLock::new();
    C.get_or_init(|| serde_json::from_str(include_str!("../data/penguin-config.json")).unwrap())
}
pub const GROUPS: [&str; 9] = [
    "prize",
    "obstacle",
    "finish",
    "ice",
    "moving",
    "snow",
    "scenery",
    "shadow",
    "character",
];
#[derive(Clone, Debug)]
pub struct Element {
    pub key: String,
    pub group: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub flip_x: bool,
    pub flip_y: bool,
    pub killed: bool,
    pub origin: (f64, f64),
    pub motion: Option<(bool, f64, f64)>,
    pub body: [f64; 4],
}
impl Element {
    pub fn position(&self, t: f64) -> (f64, f64) {
        let (mut x, mut y) = self.origin;
        if let Some((axis, d, duration)) = self.motion {
            let cycle = (t / duration) % 2.;
            let q = if cycle <= 1. { cycle } else { 2. - cycle };
            let e = if q < 0.5 {
                4. * q * q * q
            } else {
                1. - (-2. * q + 2.).powi(3) / 2.
            };
            if axis {
                x += d * e
            } else {
                y += d * e
            }
        }
        (x, y)
    }
}
#[derive(Default, Clone, Copy)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}
pub struct PenguinDash {
    pub levels: Vec<Vec<Element>>,
    pub random: Random,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub angle: f64,
    pub state: u8,
    pub state_since: f64,
    pub shadow: (f64, f64),
    pub shadow_alpha: f32,
    pub dust_alpha: f32,
    pub on_path: bool,
    pub on_fast: bool,
    pub rendering: bool,
    pub dead: bool,
    pub multiple: f64,
    pub moving_offset: Option<(f64, f64)>,
    pub level: usize,
    pub visible_level: usize,
    pub score: i64,
    pub elapsed: f64,
    pub timer: u32,
    pub total_timer: u32,
    pub paused: bool,
    engine_paused: bool,
    pub transition: bool,
    pub gameover: bool,
    pub time: f64,
    pub wall: f64,
    pub transition_at: f64,
    pub sounds: Vec<&'static str>,
    pub input: Input,
    pub tutorial_off: bool,
    death_at: Option<f64>,
    counter: f64,
    body_lag: bool,
    group_lag: bool,
    pub body: [f64; 4],
    pub frame_time: f64,
    pub tween_time: f64,
    pub camera: (f64, f64),
    pub viewport: (f64, f64),
    pub wave_random: Random,
}
impl PenguinDash {
    pub fn new(seed: &str) -> Self {
        Self::from_random(Random::seeded(seed))
    }
    fn from_random(mut rng: Random) -> Self {
        let c = config();
        let wave_random = rng.clone();
        // Start.create constructs four waves (type, x, y) before initLevels_.
        for _ in 0..12 {
            rng.next();
        }
        let mut levels = vec![];
        for level in c["levels"].as_array().unwrap().iter().take(10) {
            let mut elements = vec![];
            for item in level.as_array().unwrap() {
                let group = GROUPS
                    .iter()
                    .position(|g| Some(*g) == item["g"].as_str())
                    .unwrap();
                let key = format!(
                    "{}-{}",
                    if group == 8 { "sprite" } else { "element" },
                    item["t"].as_str().unwrap()
                );
                let (w, h) = image_size(&key);
                let character = group == 8;
                let scale = if character { 1. } else { 0.5 };
                let x = item["x"].as_f64().unwrap();
                let y = item["y"].as_f64().unwrap();
                let rot = item["r"]
                    .as_f64()
                    .or_else(|| item["r"].as_str().and_then(|v| v.parse().ok()))
                    .unwrap_or(0.);
                let centered = group == 2 || group == 6;
                let flip_x = item["f"] == "x";
                let flip_y = item["f"] == "y";
                let mut body = [
                    x - if centered { w * scale / 2. } else { 0. },
                    y - if centered { h * scale / 2. } else { 0. },
                    w * scale,
                    h * scale,
                ];
                if group == 1 {
                    let o = &c["constants"]["OBSTACLES"][item["t"].as_str().unwrap()];
                    body = [
                        x + o["offsetX"].as_f64().unwrap() * scale,
                        y + o["offsetY"].as_f64().unwrap() * scale,
                        o["width"].as_f64().unwrap() * scale,
                        o["height"].as_f64().unwrap() * scale,
                    ]
                }
                if character {
                    body[2] = 0.;
                    body[3] = 0.;
                }
                let motion = if group == 4 {
                    Some((
                        item["c"]["axis"] == "x",
                        item["c"]["movement"]
                            .as_str()
                            .unwrap()
                            .parse::<f64>()
                            .unwrap(),
                        item["c"]["duration"].as_f64().unwrap_or(2000.) / 1000.,
                    ))
                } else {
                    None
                };
                elements.push(Element {
                    key: key.clone(),
                    group,
                    x,
                    y,
                    width: w * scale,
                    height: h * scale,
                    rotation: rot,
                    flip_x,
                    flip_y,
                    killed: false,
                    origin: (x, y),
                    motion,
                    body,
                });
                let shadow = format!("{key}-shadow");
                if c["dims"].get(&shadow).is_some() {
                    let (sw, sh) = image_size(&shadow);
                    elements.push(Element {
                        key: shadow,
                        group: 7,
                        x: x - 6.,
                        y: y + 4.,
                        width: sw * 0.5,
                        height: sh * 0.5,
                        rotation: 0.,
                        flip_x: false,
                        flip_y: false,
                        killed: false,
                        origin: (x - 6., y + 4.),
                        motion,
                        body: [x - 6., y + 4., sw * 0.5, sh * 0.5],
                    });
                }
            }
            place_prizes(&mut elements, || rng.next());
            // Phaser inserts groups in source order; collision callbacks iterate their own children.
            elements.sort_by_key(|e| e.group);
            levels.push(elements);
        }
        let mut g = Self {
            levels,
            random: rng,
            x: 0.,
            y: 0.,
            vx: 0.,
            vy: 0.,
            angle: 90.,
            state: 0,
            state_since: 0.,
            shadow: (0., 6.),
            shadow_alpha: 1.,
            dust_alpha: 0.,
            on_path: true,
            on_fast: false,
            rendering: true,
            dead: false,
            multiple: 1.,
            moving_offset: None,
            level: 1,
            visible_level: 1,
            score: 0,
            elapsed: 0.,
            timer: 0,
            total_timer: 0,
            paused: false,
            engine_paused: false,
            transition: false,
            gameover: false,
            time: 0.,
            wall: 0.,
            transition_at: -10.,
            sounds: vec!["music_start_ingame", "pnd_slide_start"],
            input: Input::default(),
            tutorial_off: false,
            death_at: None,
            counter: 0.,
            body_lag: false,
            group_lag: false,
            body: [0.; 4],
            frame_time: 0.,
            tween_time: 0.,
            camera: (0., 0.),
            viewport: (1440., 800.),
            wave_random,
        };
        g.show_level();
        g.update_camera();
        g
    }
    pub fn resize(&mut self, w: f64, h: f64) {
        let scale = if w < 980. { (w + 490.) / 1470. } else { 1. };
        self.viewport = (w / scale, h / scale);
    }
    fn update_camera(&mut self) {
        let (w, h) = self.viewport;
        self.camera = (
            (self.x - (w / 2.).round())
                .clamp(0., (4000. - w).max(0.))
                .floor(),
            (self.y - (h / 2.).round())
                .clamp(0., (3000. - h).max(0.))
                .floor(),
        );
    }
    pub fn restart(&mut self) {
        *self = Self::from_random(self.random.clone());
    }
    fn set_state(&mut self, s: u8) {
        if self.state != s {
            self.state = s;
            self.state_since = self.time
        }
    }
    pub fn show_level(&mut self) {
        self.group_lag |= self.visible_level != self.level;
        self.visible_level = self.level;
        let e = self.levels[self.level - 1]
            .iter()
            .find(|e| e.group == 5)
            .unwrap();
        self.x = e.x + e.width / 2.;
        self.y = e.y + e.height / 2.;
        self.on_path = true;
        self.rendering = true;
    }
    pub fn restart_level(&mut self) {
        self.total_timer += self.timer;
        self.timer = 0;
        self.vx = 0.;
        self.vy = 0.;
        self.shadow_alpha = 1.;
        self.set_state(0);
        self.show_level();
        self.body_lag = true;
        self.transition = false;
        self.engine_paused = false;
        self.sounds.extend(["pnd_slide_start", "pnd_restart"]);
    }
    pub fn pause(&mut self) {
        self.paused = true;
        self.engine_paused = true;
        self.sounds.push("pnd_slide_stop")
    }
    pub fn resume(&mut self) {
        self.paused = false;
        self.engine_paused = false;
        self.sounds.push("pnd_slide_start")
    }
    pub fn wall_step(&mut self, dt: f64) {
        self.wall += dt;
        if self.death_at.is_some_and(|t| self.wall + 1e-9 >= t) {
            self.death_at = None;
            self.dead = false;
            self.restart_level();
            self.elapsed += 20.;
        }
        if self.transition && self.wall - self.transition_at >= 0.5 {
            self.restart_level();
        }
    }
    pub fn orientation(&mut self, alpha: f64, beta: f64, gamma: f64) {
        if alpha != 0.
            && beta != 0.
            && gamma != 0.
            && !self.engine_paused
            && !self.transition
            && !self.dead
            && !self.gameover
        {
            self.vx += gamma * 0.5;
            self.vy += beta * 0.25;
            if gamma.abs() > 10. || beta.abs() > 5. {
                self.tutorial_off = true;
            }
        }
    }
    pub fn running(&self) -> bool {
        !self.engine_paused && !self.gameover
    }
    pub fn finish(&mut self) {
        self.engine_paused = true;
        self.sounds.push("pnd_slide_stop");
        if self.level >= 10 {
            self.gameover = true;
            self.sounds
                .extend(["music_ingame_gameover", "pnd_game_over"]);
        } else {
            self.level += 1;
            self.score += 50;
            self.transition = true;
            self.transition_at = self.wall;
            self.sounds
                .extend(["level_transition_close", "pnd_level_up"]);
        }
    }
    pub fn advance(&mut self, seconds: f64) {
        for _ in 0..(seconds * 60.).floor() as u64 {
            self.wall_step(1. / 60.);
            self.step();
        }
    }
    pub fn teleport(&mut self, x: f64, y: f64, vx: f64, vy: f64) {
        self.x = x;
        self.y = y;
        self.vx = vx;
        self.vy = vy;
        self.on_path = true;
        self.rendering = true;
        self.body_lag = false;
    }
    pub fn step(&mut self) {
        if !self.running() {
            return;
        }
        let dt = 1. / 60.;
        self.frame_time = self.tween_time;
        self.time += dt;
        self.counter += dt;
        if self.counter + 1e-9 >= 1. {
            self.counter -= 1.;
            self.timer += 1;
            self.elapsed += 1.;
        }
        // Arcade preUpdate integrates last tick velocity before Start.update adds input.
        self.vx = self.vx.clamp(-10000., 10000.);
        self.vy = self.vy.clamp(-10000., 10000.);
        let start = if self.body_lag {
            [self.body[0], self.body[1]]
        } else {
            [self.x - 34.2, self.y - 29.]
        };
        self.body_lag = false;
        let mut b = [start[0] + self.vx * dt, start[1] + self.vy * dt, 66., 66.];
        for axis in 0..2 {
            let limit = if axis == 0 { 3934. } else { 2934. };
            if b[axis] < 0. {
                b[axis] = 0.;
                if axis == 0 {
                    self.vx *= -0.3
                } else {
                    self.vy *= -0.3
                }
            } else if b[axis] > limit {
                b[axis] = limit;
                if axis == 0 {
                    self.vx *= -0.3
                } else {
                    self.vy *= -0.3
                }
            }
        }
        // Moving tween position is applied after the state update; bodies use the last tick pose.
        let level = self.level - 1;
        for e in &mut self.levels[level] {
            if e.motion.is_some() {
                let (x, y) = e.position(self.frame_time);
                e.x = x;
                e.y = y;
                e.body[0] = x;
                e.body[1] = y;
            }
        }
        if !self.on_path && !self.rendering {
            if !self.dead {
                self.dead = true;
                self.death_at = Some(self.wall + 2.5);
                self.set_state(2);
                self.shadow_alpha = 0.;
                self.dust_alpha = 0.;
                self.sounds.extend(["pnd_freeze", "pnd_slide_stop"]);
            }
            self.vx *= 0.93;
            self.vy *= 0.93;
        } else {
            if !self.on_fast {
                self.multiple = 1.;
                self.vx *= 0.98;
                self.vy *= 0.98;
            }
            self.on_path = self.rendering;
            self.on_fast = false;
            self.rendering = false;
            let i = self.input;
            let mut accelerating = false;
            let f = 10. * self.multiple;
            if i.left {
                self.vx -= f;
                accelerating = true
            } else if i.right {
                self.vx += f;
                accelerating = true
            }
            if i.up {
                self.vy -= f;
                accelerating = true
            } else if i.down {
                self.vy += f;
                accelerating = true
            }
            if accelerating {
                self.set_state(1);
                self.tutorial_off = true
            } else {
                self.set_state(0)
            }
            self.dust_alpha = (((self.vx.abs() + self.vy.abs()) / 1000.) * 2. - 0.2) as f32;
            if self.vx != 0. || self.vy != 0. {
                let theta = self.vy.atan2(self.vx).to_degrees();
                let diff = -((self.angle - theta + 180.).rem_euclid(360.) - 180.);
                self.angle = (self.angle + diff / 8. + 180.).rem_euclid(360.) - 180.;
                self.shadow = (
                    -self.angle.to_radians().cos() * 6.,
                    self.angle.to_radians().sin() * 6.,
                );
            }
            let mut moving = false;
            for group in 0..9 {
                let level = self.level - 1;
                for idx in 0..self.levels[level].len() {
                    let e = &self.levels[level][idx];
                    if e.group != group || e.killed {
                        continue;
                    }
                    let mut body = e.body;
                    if level + 1 != self.visible_level || self.group_lag {
                        // Hidden Phaser groups retain their world transforms. If a finish
                        // changes level during this loop, later groups use that hidden
                        // level's bodies at the previous camera offset for this tick.
                        let (x, y) = e.position(self.frame_time);
                        body[0] += x - e.origin.0 + self.camera.0;
                        body[1] += y - e.origin.1 + self.camera.1;
                    }
                    let overlap = intersects(b, body);
                    if [0, 3, 4, 5, 8].contains(&group) && overlap {
                        self.on_path = true;
                    }
                    if !overlap {
                        continue;
                    }
                    match group {
                        0 => {
                            self.levels[level][idx].killed = true;
                            self.score += 10;
                            self.sounds.push("pnd_pickup");
                        }
                        1 => {
                            let eb = e.body;
                            let mut hit = false;
                            let dy = b[1] - start[1];
                            let dx = b[0] - start[0];
                            let oy = if dy > 0. {
                                b[1] + 66. - eb[1]
                            } else if dy < 0. {
                                b[1] - eb[1] - eb[3]
                            } else {
                                0.
                            };
                            if oy != 0. && oy.abs() <= dy.abs() + 4. {
                                b[1] -= oy;
                                self.vy *= -0.3;
                                hit = true;
                            }
                            if intersects(b, eb) {
                                let ox = if dx > 0. {
                                    b[0] + 66. - eb[0]
                                } else if dx < 0. {
                                    b[0] - eb[0] - eb[2]
                                } else {
                                    0.
                                };
                                if ox != 0. && ox.abs() <= dx.abs() + 4. {
                                    b[0] -= ox;
                                    self.vx *= -0.3;
                                    hit = true;
                                }
                            }
                            if hit {
                                self.sounds.push("pnd_hit")
                            }
                        }
                        2 => self.finish(),
                        3 => {
                            self.on_fast = true;
                            if self.multiple == 1. {
                                self.multiple = 2.;
                                self.vx *= 2.;
                                self.vy *= 2.;
                            }
                        }
                        4 => {
                            moving = true;
                            let p = (e.x, e.y);
                            if let Some(old) = self.moving_offset {
                                self.x += p.0 - old.0;
                                self.y += p.1 - old.1;
                            }
                            self.moving_offset = Some(p);
                        }
                        _ => {}
                    }
                }
                if group == 4 && !moving {
                    self.moving_offset = None;
                }
            }
        }
        self.x += b[0] - start[0];
        self.y += b[1] - start[1];
        self.body = b;
        self.update_camera();
        self.group_lag = false;
        if !self.transition && !self.gameover {
            self.tween_time += dt;
        }
    }
    pub fn snapshot(&self) -> Value {
        json!({"x":self.x,"y":self.y,"vx":self.vx,"vy":self.vy,"angle":self.angle,"state":self.state,"frame":player_frame(self.state,self.time-self.state_since),"onPath":self.on_path,"onFastIce":self.on_fast,"multiple":self.multiple,"dead":self.dead,"level":self.level,"visibleLevel":self.visible_level,"score":self.score,"elapsed":self.elapsed,"body":self.body,"killed":self.levels.iter().map(|l|l.iter().filter(|e|e.group==0).map(|e|e.killed).collect::<Vec<_>>()).collect::<Vec<_>>()})
    }
}
fn place_prizes(elements: &mut Vec<Element>, mut random: impl FnMut() -> f64) {
    let ice: Vec<_> = [5, 3]
        .iter()
        .flat_map(|g| elements.iter().filter(move |e| e.group == *g).cloned())
        .collect();
    let snowlen = elements.iter().filter(|e| e.group == 5).count();
    for _ in 0..ice.len() / 2 {
        let t = (random() * 6.).floor() as u32 + 1;
        let mut i = (random() * (ice.len() - 1) as f64).floor() as usize + 1;
        while i == snowlen - 1 {
            i = (random() * (ice.len() - 1) as f64).floor() as usize + 1;
        }
        let e = &ice[i];
        let x = e.x + 50. + (e.width - 120.) * random();
        let y = e.y + 50. + (e.width - 120.) * random();
        let key = format!("element-present{t}");
        let (w, h) = image_size(&key);
        elements.push(Element {
            key,
            group: 0,
            x,
            y,
            width: w * 0.5,
            height: h * 0.5,
            rotation: 0.,
            flip_x: false,
            flip_y: false,
            killed: false,
            origin: (x, y),
            motion: None,
            body: [x, y, w * 0.5, h * 0.5],
        });
    }
}
pub fn intersects(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] + a[2] > b[0] && a[1] + a[3] > b[1] && a[0] < b[0] + b[2] && a[1] < b[1] + b[3]
}
pub fn image_size(key: &str) -> (f64, f64) {
    let d = &config()["dims"][key];
    let w = d[0].as_f64().unwrap();
    let mut h = d[1].as_f64().unwrap();
    if key.starts_with("sprite-") {
        h = match key {
            "sprite-penguin" | "sprite-dust" => 108.,
            "sprite-duck" | "sprite-green_elf" | "sprite-orange_elf" | "sprite-walrus" => 202.,
            "sprite-wave1" | "sprite-wave2" => 100.,
            _ => h,
        }
    }
    (w, h)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn compare(a: &Value, b: &Value, path: &str) {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-6,
                "{path}: {a} != {b}"
            ),
            (Value::Object(a), Value::Object(b)) => {
                for (k, v) in b {
                    compare(&a[k], v, &format!("{path}.{k}"));
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}[{i}]"));
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    #[test]
    fn original_phaser_scenarios() {
        let data: Value =
            serde_json::from_str(include_str!("../data/penguin-reference.json")).unwrap();
        let mut g = PenguinDash::new("test1234");
        for (i, s) in data.as_array().unwrap().iter().enumerate() {
            let v = &s["value"];
            match s["action"].as_str().unwrap() {
                "step" => {
                    g.input = Input {
                        left: v["keys"]["left"] == true,
                        right: v["keys"]["right"] == true,
                        up: v["keys"]["up"] == true,
                        down: v["keys"]["down"] == true,
                    };
                    for _ in 0..v["n"].as_u64().unwrap() {
                        g.wall_step(1. / 60.);
                        g.step();
                    }
                }
                "pause" => g.pause(),
                "resume" => g.resume(),
                "teleport" => g.teleport(
                    v[0].as_f64().unwrap(),
                    v[1].as_f64().unwrap(),
                    v[2].as_f64().unwrap(),
                    v[3].as_f64().unwrap(),
                ),
                "level" => {
                    g.level = v.as_u64().unwrap() as usize;
                    g.restart_level()
                }
                "transition" => g.restart_level(),
                _ => unreachable!(),
            };
            compare(&g.snapshot(), &s["state"], &format!("checkpoint {i}"));
        }
    }
    #[test]
    fn original_level_layout() {
        let c: Value =
            serde_json::from_str(include_str!("../data/penguin-source-layout.json")).unwrap();
        let g = PenguinDash::new("test1234");
        for (i, l) in g.levels.iter().enumerate() {
            for (group, name) in GROUPS.iter().enumerate() {
                let actual: Vec<_> = l.iter().filter(|e| e.group == group).collect();
                let expected = c["layout"][i][name].as_array().unwrap();
                assert_eq!(actual.len(), expected.len());
                for (e, r) in actual.iter().zip(expected) {
                    assert_eq!(e.key, r["key"].as_str().unwrap());
                    compare(
                        &json!(e.body),
                        &json!([
                            r["body"]["x"],
                            r["body"]["y"],
                            r["body"]["w"],
                            r["body"]["h"]
                        ]),
                        &format!("level{i} {name} body"),
                    );
                    compare(
                        &json!([e.x, e.y, e.width, e.height]),
                        &json!([
                            r["x"],
                            r["y"],
                            r["w"].as_f64().unwrap().abs(),
                            r["h"].as_f64().unwrap().abs()
                        ]),
                        &format!("level{i} {name}"),
                    );
                }
            }
        }
    }
    #[test]
    fn live_placement_matches_original_random_draws() {
        let live: Value = serde_json::from_str(include_str!(
            "../data/penguin-live-placement-reference.json"
        ))
        .unwrap();
        assert_eq!(
            &config()["levels"].as_array().unwrap()[..10],
            live["fixedCourse"].as_array().unwrap()
        );
        assert_eq!(
            live["before"], live["after"],
            "Live retry keeps original positions"
        );
        let g = PenguinDash::new("test1234");
        for (i, level) in live["restarted"].as_array().unwrap().iter().enumerate() {
            let mut elements = g.levels[i].clone();
            elements.retain(|e| e.group != 0);
            let mut tape = level["calls"].as_array().unwrap().iter();
            place_prizes(&mut elements, || {
                tape.next()
                    .expect("Original RNG tape exhausted")
                    .as_f64()
                    .unwrap()
            });
            assert!(
                tape.next().is_none(),
                "All original placement draws consumed"
            );
            let actual: Vec<_> = elements
                .iter()
                .filter(|e| e.group == 0)
                .map(|e| json!({"key":e.key,"x":e.x,"y":e.y,"w":e.width,"h":e.height}))
                .collect();
            compare(
                &json!(actual),
                &level["prizes"],
                &format!("live level {}", i + 1),
            );
        }
    }
    #[test]
    fn retry_keeps_baubles_full_restart_generates_new_ones() {
        let positions = |g: &PenguinDash| {
            json!(g
                .levels
                .iter()
                .map(|l| l
                    .iter()
                    .filter(|e| e.group == 0)
                    .map(|e| (&e.key, e.x, e.y, e.killed))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>())
        };
        let mut g = PenguinDash::new("test1234");
        for e in &mut g.levels[0] {
            if e.group == 0 {
                e.killed = true;
                break;
            }
        }
        let before = positions(&g);
        g.restart_level();
        assert_eq!(before, positions(&g));
        g.restart();
        assert_ne!(before, positions(&g));
        let fresh = positions(&g);
        g.restart();
        assert_ne!(fresh, positions(&g));
    }
    #[test]
    fn original_orientation_conditions() {
        let data: Value =
            serde_json::from_str(include_str!("../data/penguin-motion-reference.json")).unwrap();
        let mut g = PenguinDash::new("test1234");
        for c in data.as_array().unwrap() {
            g.vx = 7.;
            g.vy = -9.;
            g.paused = c["paused"].as_bool().unwrap();
            g.engine_paused = g.paused;
            g.dead = c["dead"].as_bool().unwrap();
            g.tutorial_off = false;
            g.orientation(
                c["alpha"].as_f64().unwrap(),
                c["beta"].as_f64().unwrap(),
                c["gamma"].as_f64().unwrap(),
            );
            assert_eq!(
                (g.vx, g.vy, g.tutorial_off),
                (
                    c["vx"].as_f64().unwrap(),
                    c["vy"].as_f64().unwrap(),
                    c["dismissed"].as_bool().unwrap()
                )
            );
        }
    }
}

pub fn player_frame(state: u8, time: f64) -> usize {
    let t = (time.max(0.) * 12.).floor() as usize;
    match state {
        1 => [1, 2, 3, 4, 5, 6, 7, 8, 9, 8, 7, 6, 5, 4, 3, 2][t % 16],
        2 => 10 + t.min(9),
        3 => 20 + t.min(11),
        _ => 0,
    }
}
