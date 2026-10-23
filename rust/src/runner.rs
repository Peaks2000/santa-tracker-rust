// Behavioral port of static/scenes/runner/js. Original: Google, Apache-2.0.
use crate::{
    random::Random,
    ui::{Ease, Transition},
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spec {
    pub css: String,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub score: u32,
    #[serde(default)]
    pub hit_bottom: f64,
    #[serde(default)]
    pub hit_width: Option<f64>,
    #[serde(default)]
    pub presents_height: Option<f64>,
}
fn specs(name: &str) -> Vec<Spec> {
    static DATA: OnceLock<serde_json::Value> = OnceLock::new();
    let data = DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/runner-constants.json")).unwrap()
    });
    serde_json::from_value(data[name].clone()).expect("checked original constant table")
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum State {
    Running,
    Sliding,
    Jumping,
    Collision,
    JumpStart,
}
#[derive(Clone, Serialize)]
pub struct Player {
    pub x: f64,
    pub y: f64,
    pub x_speed: f64,
    pub y_speed: f64,
    pub acceleration: f64,
    pub angle: f64,
    pub jumps: u32,
    pub jump_end: bool,
    pub state: State,
    pub state_since: f64,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            x: 450.,
            y: 0.,
            x_speed: 0.,
            y_speed: 0.,
            acceleration: 0.,
            angle: 0.,
            jumps: 0,
            jump_end: false,
            state: State::Running,
            state_since: 0.,
        }
    }
}
#[derive(Copy, Clone)]
struct Hitbox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
impl Player {
    fn hitbox(&self) -> Hitbox {
        let (x, y, w, h) = match self.state {
            State::Running => (95., 0., 175., 170.),
            State::Sliding => (63., 0., 200., 95.),
            State::Jumping | State::JumpStart => (70., 30., 195., 115.),
            State::Collision => (110., 10., 120., 165.),
        };
        Hitbox {
            x: self.x + x,
            y: self.y - y,
            width: w,
            height: h,
        }
    }
}
#[derive(Clone, Serialize, PartialEq)]
pub enum Kind {
    Obstacle,
    Platform,
    Present,
    Time,
    Magnet,
    Finish,
}
#[derive(Clone, Serialize)]
pub struct Entity {
    pub kind: Kind,
    pub style: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub collected: bool,
    pub collected_at: f64,
    pub collected_wall_at: f64,
    pub born: f64,
    pub hit: bool,
    pub score: u32,
    pub hit_bottom: f64,
    pub hit_width: Option<f64>,
    pub presents_height: Option<f64>,
}
impl Entity {
    fn from_spec(kind: Kind, spec: Spec, x: f64, y: f64) -> Self {
        Self {
            kind,
            style: spec.css,
            x,
            y,
            width: spec.width,
            height: spec.height,
            collected: false,
            collected_at: 0.,
            collected_wall_at: 0.,
            born: 0.,
            hit: false,
            score: spec.score,
            hit_bottom: spec.hit_bottom,
            hit_width: spec.hit_width,
            presents_height: spec.presents_height,
        }
    }
    fn hitbox(&self, distance: f64) -> Hitbox {
        let platform = self.kind == Kind::Platform || self.kind == Kind::Finish;
        Hitbox {
            x: self.x - distance,
            y: if platform {
                -self.height
            } else {
                self.y - self.hit_bottom
            },
            width: self.width,
            height: if platform { 0. } else { self.height },
        }
    }
}
#[derive(Clone, Copy)]
enum TimerAction {
    Recover,
    CloudOff,
    MagnetOff,
    EndSlide,
}

pub struct Runner {
    pub player: Player,
    pub entities: Vec<Entity>,
    pub time: f64,
    pub countdown: f64,
    pub distance: f64,
    pub speed: f64,
    pub level: usize,
    pub score: u32,
    pub playing: bool,
    pub paused: bool,
    pub magnet: bool,
    pub cloud: bool,
    pub sounds: Vec<&'static str>,
    pub finish_x: f64,
    random: Random,
    last_entity: f64,
    last_obstacle: f64,
    last_boost: f64,
    timers: Vec<(f64, TimerAction)>,
    last_seconds: Option<f64>,
    hurry: bool,
    pub transition: f64,
    pub immediate_levels: bool,
    pub animation_time: f64,
    pub wall_time: f64,
    pub cloud_since: f64,
    pub magnet_visual: Transition<1>,
    pub visible_width: f64,
    pub mountain_x: f64,
    pub trees_x: f64,
    next_mountain: f64,
    next_trees: f64,
}
impl Runner {
    pub fn new(seed: &str) -> Self {
        let finish = Spec {
            css: "finish".into(),
            width: 300.,
            height: 265.,
            score: 0,
            hit_bottom: 0.,
            hit_width: None,
            presents_height: None,
        };
        Self {
            player: Player::default(),
            entities: vec![Entity::from_spec(Kind::Finish, finish, 60900., 0.)],
            time: 0.,
            countdown: 120.,
            distance: 0.,
            speed: 350.,
            level: 0,
            score: 0,
            playing: true,
            paused: false,
            magnet: false,
            cloud: false,
            sounds: vec!["runner_start", "music_start_ingame"],
            finish_x: 60900.,
            random: Random::seeded(seed),
            last_entity: 1000.,
            last_obstacle: 0.,
            last_boost: 0.,
            timers: vec![],
            last_seconds: None,
            hurry: false,
            transition: 0.,
            immediate_levels: false,
            animation_time: 0.,
            wall_time: 0.,
            cloud_since: 0.,
            magnet_visual: Transition::new([0.]),
            visible_width: 2115.,
            mountain_x: 0.,
            trees_x: 0.,
            next_mountain: -0.25,
            next_trees: 0.,
        }
    }
    pub fn restart(&mut self) {
        let mut next = Self::new("unused-restart-seed");
        next.random = self.random.clone();
        next.animation_time = self.animation_time;
        next.wall_time = self.wall_time;
        next.visible_width = self.visible_width;
        next.mountain_x = self.mountain_x;
        next.trees_x = self.trees_x;
        next.next_mountain = self.next_mountain;
        next.next_trees = self.next_trees;
        next.player.state_since = if self.player.state == State::Running {
            self.player.state_since
        } else {
            self.animation_time
        };
        *self = next;
    }
    pub fn player_hitbox(&self) -> (f64, f64, f64, f64) {
        let h = self.player.hitbox();
        (h.x, h.y, h.width, h.height)
    }
    fn state(&mut self, next: State, recover: bool, falling: bool) {
        if self.player.state == State::Collision && !recover {
            return;
        }
        let prev = self.player.state;
        self.player.state = next;
        if prev != next {
            self.player.state_since = self.animation_time;
        }
        if next == State::Running && prev != State::Running {
            self.player.jumps = 0;
            self.player.jump_end = false;
        } else if next == State::JumpStart && !falling {
            self.sounds.push(if self.player.jumps == 0 {
                "runner_jump_1"
            } else {
                "runner_jump_2"
            });
            self.player.acceleration = 0.;
        } else if next == State::Collision {
            self.player.x_speed = -200.;
            self.player.y_speed = -500.;
            self.cloud = true;
            self.cloud_since = self.wall_time;
            self.timers.push((0.5, TimerAction::CloudOff));
            self.timers.push((2., TimerAction::Recover));
        }
    }
    pub fn jump(&mut self) {
        if self.player.jumps < 2 {
            self.state(State::JumpStart, false, false);
            self.player.jumps += 1;
            self.player.jump_end = false;
        }
    }
    pub fn down(&mut self) {
        if self.player.state == State::Running {
            self.state(State::Sliding, false, false);
        } else if matches!(self.player.state, State::Jumping | State::JumpStart)
            && !self.player.jump_end
        {
            self.state(State::Jumping, false, false);
            self.player.y_speed += 500.;
            self.player.jump_end = true;
        }
    }
    pub fn down_for(&mut self, duration: f64) {
        let running = self.player.state == State::Running;
        self.down();
        if running && duration > 0. {
            self.timers.push((duration, TimerAction::EndSlide));
        }
    }
    pub fn end_slide(&mut self) {
        if self.player.state == State::Sliding {
            self.state(State::Running, false, false);
        }
    }
    fn land(&mut self, y: f64) {
        self.player.y = y;
        self.player.y_speed = 0.;
        self.player.angle = 0.;
        self.state(State::Running, false, false);
    }
    pub fn advance(&mut self, duration: f64) {
        let mut remaining = duration;
        while remaining > 0. && self.playing {
            let dt = remaining.min(1. / 60.);
            self.step(dt);
            remaining -= dt;
        }
    }
    pub fn step(&mut self, dt: f64) {
        if !self.playing || self.paused {
            return;
        }
        self.time += dt;
        self.animation_time += dt;
        self.wall_time += dt;
        if self.next_mountain < 352.5 - 1490. {
            self.next_mountain = 352.5;
        }
        if self.next_trees < -2980. {
            self.next_trees = 0.;
        }
        self.mountain_x = self.next_mountain;
        self.trees_x = self.next_trees;
        self.next_mountain -= 0.25;
        self.next_trees -= 0.5;
        if self.transition > 0. {
            self.transition -= dt;
            if self.transition <= 0. {
                self.level += 1;
                self.speed = 350. * [1., 1.3, 1.6, 1.9][self.level];
            }
        } else if self.level < 3 && self.time.floor() >= [28., 58., 88.][self.level] {
            if self.immediate_levels {
                self.level += 1;
                self.speed = 350. * [1., 1.3, 1.6, 1.9][self.level];
            } else {
                self.transition = 2.;
            }
        }
        let mut i = 0;
        while i < self.timers.len() {
            self.timers[i].0 = (self.timers[i].0 - dt).max(0.);
            if self.timers[i].0 <= 0. {
                let (_, action) = self.timers.remove(i);
                match action {
                    TimerAction::Recover => self.state(State::Running, true, false),
                    TimerAction::CloudOff => self.cloud = false,
                    TimerAction::EndSlide => self.end_slide(),
                    TimerAction::MagnetOff => {
                        self.magnet = false;
                        self.magnet_visual
                            .set(self.wall_time, [0.], 0.5, Ease::Default);
                    }
                }
            } else {
                i += 1;
            }
        }
        self.distance += dt * self.speed;
        self.countdown -= dt;
        let sec = self.countdown.ceil();
        if self.last_seconds != Some(sec) {
            self.last_seconds = Some(sec);
            if self.countdown < 0. {
                self.countdown = 0.;
                self.playing = false;
                self.sounds
                    .extend(["runner_game_over", "music_ingame_gameover"]);
            }
        }
        if self.countdown < 10. && !self.hurry {
            self.hurry = true;
            self.sounds.push("runner_hurry_up");
        }
        self.move_player(dt);
        self.spawn_entities();
        let player = self.player.hitbox();
        for e in &mut self.entities {
            if e.kind == Kind::Present && self.magnet && self.player.state != State::Collision {
                let dx = player.x + player.width / 2. - (e.x - self.distance + e.width / 2.);
                let dy = player.y - player.height / 2. - (e.y + e.height / 2.);
                if dx.hypot(dy) < 500. {
                    e.x += dx / 10.;
                    e.y += dy / 10.;
                }
            }
        }
        self.entities.retain(|e| e.x >= self.distance - e.width);
    }
    fn move_player(&mut self, dt: f64) {
        let start_y = self.player.y;
        let mut end_y = self.player.y;
        self.player.angle = (self.player.y_speed / 1100.).min(1.) * 15.;
        if self.player.state == State::JumpStart {
            self.player.acceleration -= 28.;
            self.player.y_speed += self.player.acceleration;
            self.player.y += self.player.y_speed * dt;
            end_y = self.player.y;
            if self.player.y_speed <= -1100. {
                self.state(State::Jumping, false, false);
            }
        }
        if (self.player.y_speed != 0. || self.player.state == State::Jumping)
            && self.player.state != State::JumpStart
        {
            self.player.y += self.player.y_speed * dt;
            self.player.y_speed += 2400. * dt;
            if self.player.y >= 0. {
                self.land(0.);
            }
            end_y = self.player.y;
        }
        self.player.x += self.player.x_speed * dt;
        if self.player.state == State::Collision {
            self.player.x_speed = (self.player.x_speed + 250. * dt).min(0.);
        } else if self.player.x < 450. {
            self.player.x_speed += 100. * dt;
        } else {
            self.player.x = 450.;
            self.player.x_speed = 0.;
        }
        let mut needs_fall = self.player.y < 0. && self.player.y_speed == 0.;
        let mut platform_found = false;
        for idx in 0..self.entities.len() {
            let e = &self.entities[idx];
            let h = e.hitbox(self.distance);
            let p = self.player.hitbox();
            let horizontal = !(h.x + h.width < p.x || h.x >= p.x + p.width);
            if matches!(e.kind, Kind::Platform | Kind::Finish) && !platform_found {
                if start_y < end_y && horizontal && -e.height <= end_y && -e.height > start_y {
                    let height = e.height;
                    let finish = e.kind == Kind::Finish;
                    self.land(-height);
                    platform_found = true;
                    if finish {
                        self.state(State::Jumping, false, false);
                        self.player.y_speed = -1750.;
                    }
                } else if needs_fall {
                    platform_found = horizontal && -e.height == self.player.y;
                    needs_fall = !platform_found;
                }
            } else if self.player.state != State::Collision
                && horizontal
                && !(p.y - p.height > h.y || p.y < h.y - h.height)
            {
                let e = &mut self.entities[idx];
                match e.kind {
                    Kind::Present if !e.collected => {
                        e.collected = true;
                        e.collected_at = self.time;
                        e.collected_wall_at = self.wall_time;
                        self.score += e.score;
                        self.sounds.push("runner_present");
                    }
                    Kind::Time | Kind::Magnet if !e.collected => {
                        e.collected = true;
                        e.collected_at = self.time;
                        e.collected_wall_at = self.wall_time;
                        let time = e.kind == Kind::Time;
                        self.sounds.push("runner_present");
                        if time {
                            self.add_time(10.);
                        } else {
                            self.magnet = true;
                            self.magnet_visual
                                .set(self.wall_time, [1.], 0.5, Ease::Default);
                            self.timers.push((20., TimerAction::MagnetOff));
                        }
                    }
                    Kind::Obstacle => {
                        if let Some(w) = e.hit_width {
                            e.width = w;
                            e.hit = true;
                        } else {
                            self.state(State::Collision, false, false);
                            self.sounds.push("runner_hit");
                        }
                    }
                    _ => {}
                }
            }
        }
        if needs_fall {
            self.state(State::Jumping, false, true);
            self.player.y_speed = 200.;
        }
    }
    fn add_time(&mut self, time: f64) {
        if self.countdown < 10. && self.countdown + time > 10. {
            self.hurry = false;
            self.sounds.push("hurry_up_end");
        }
        self.countdown += time;
        self.finish_x += time * 350. * 1.9;
        if let Some(e) = self.entities.iter_mut().find(|e| e.kind == Kind::Finish) {
            e.x = self.finish_x;
        }
    }
    fn pick(&mut self, table: &str) -> Spec {
        let specs = specs(table);
        specs[(self.random.next() * specs.len() as f64).floor() as usize].clone()
    }
    fn present(&mut self, x: f64, height: f64) -> Entity {
        let table = if self.random.next() < 0.03 {
            "TREATS"
        } else {
            "PRESENTS"
        };
        let spec = self.pick(table);
        let y = -height - spec.height / 2.;
        let mut e = Entity::from_spec(Kind::Present, spec, x, y);
        e.born = self.animation_time;
        e
    }
    fn add_presents(
        &mut self,
        mut pos: f64,
        entity_width: Option<f64>,
        height: Option<f64>,
    ) -> f64 {
        let height = height
            .filter(|v| *v != 0.)
            .unwrap_or_else(|| (self.random.next() * 500. + 75.).floor());
        if let Some(width) = entity_width {
            let mut total = 0.;
            let mut presents = vec![];
            while total < width {
                let p = self.present(0., height);
                if total + p.width > width {
                    break;
                }
                total += p.width + 75.;
                presents.push(p);
            }
            total -= 75.;
            let mut x = pos + (width - total) / 2.;
            for mut p in presents {
                p.x = x;
                x += p.width + 75.;
                self.entities.push(p);
            }
        } else {
            let num = (self.random.next() * 3. + 2.).ceil() as usize;
            let rows = (self.random.next() * 4.).ceil() as usize;
            for _ in 0..num {
                for j in 0..rows {
                    let p = self.present(pos + (j % 2) as f64 * 75., height + 75. * j as f64);
                    self.entities.push(p);
                }
                pos += 150.;
            }
        }
        pos
    }
    fn add_obstacle(&mut self, pos: f64, level: usize) -> f64 {
        if self.last_obstacle > pos - 1000. {
            return self.last_entity;
        }
        let spec = self.pick(if level > 0 {
            "OBSTACLES_PLATFORM"
        } else {
            "OBSTACLES_GROUND"
        });
        let mut e = Entity::from_spec(Kind::Obstacle, spec, pos, level as f64 * -270.);
        if level > 0 {
            e.presents_height = Some((level as f64 + 0.5) * 270. + 75.);
        }
        let width = e.width;
        let presents_height = e.presents_height;
        self.entities.push(e);
        if self.random.next() < 0.5 {
            self.add_presents(
                pos,
                Some(width),
                Some(presents_height.unwrap_or((level + 1) as f64 * 270. + 75.)),
            );
        }
        self.last_obstacle = pos + width;
        self.last_obstacle
    }
    fn add_platform(&mut self, pos: f64, level: usize, woodsy: bool, no_extra: bool) -> f64 {
        let specs = specs(if level > 0 {
            "PLATFORMS_TALL"
        } else {
            "PLATFORMS_SHORT"
        });
        let idx = if woodsy {
            specs.len() - 1
        } else {
            (self.random.next() * (specs.len() - 1) as f64).floor() as usize
        };
        let spec = specs[idx].clone();
        let width = spec.width;
        self.entities
            .push(Entity::from_spec(Kind::Platform, spec, pos, 0.));
        let random = self.random.next();
        if random < 0.25 {
            self.add_presents(pos, Some(width), Some((level + 1) as f64 * 270. + 75.));
        } else if random < 0.5 && !woodsy {
            self.add_obstacle(pos + width * 0.66, level + 1);
        }
        if self.random.next() < 0.25 && !no_extra {
            self.add_presents(pos, Some(width), Some(75.));
        }
        pos + width + 7.
    }
    fn spawn_entities(&mut self) {
        let pos = self.last_entity + 1000.;
        if pos >= self.finish_x - 1500. || self.last_entity >= self.distance + self.visible_width {
            return;
        }
        let random = self.random.next();
        self.last_entity = if random < 0.2 {
            self.add_obstacle(pos, 0)
        } else if random < 0.5 {
            self.add_presents(pos, None, None)
        } else if random < 0.75 {
            let short = (self.random.next() * 3.).ceil() as usize;
            let tall = (self.random.next() * 3.).floor() as usize;
            let woodsy = self.random.next() < 0.25;
            let mut pos = pos;
            for _ in 0..short {
                pos = self.add_platform(pos, 0, woodsy, false);
            }
            pos -= 150.;
            for i in 0..tall {
                pos = self.add_platform(pos, 1, woodsy, i == 0);
            }
            pos
        } else if random < 0.9 {
            if self.last_boost > pos - 2000. {
                self.last_entity
            } else {
                let height = (self.random.next() * 500. + 75.).floor();
                let time = self.random.next() < 0.5;
                let spec = Spec {
                    css: if time { "boost--time" } else { "boost--magnet" }.into(),
                    width: 60.,
                    height: 60.,
                    score: 0,
                    hit_bottom: 0.,
                    hit_width: None,
                    presents_height: None,
                };
                let mut e = Entity::from_spec(
                    if time { Kind::Time } else { Kind::Magnet },
                    spec,
                    pos,
                    -height - 30.,
                );
                e.born = self.animation_time;
                self.entities.push(e);
                self.last_boost = pos + 60.;
                self.last_boost
            }
        } else {
            self.last_entity + (self.random.next() * 1000.).floor()
        };
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"time":self.time,"distance":self.distance,"speed":self.speed,"level":self.level,"score":self.score,"countdown":self.countdown,"playing":self.playing,"magnet":self.magnet,"x":self.player.x,"y":self.player.y,"y_speed":self.player.y_speed,"state":self.player.state as u8,"jumps":self.player.jumps,"entities":self.entities.iter().map(|e|serde_json::json!({"style":e.style,"x":e.x,"y":e.y,"width":e.width,"height":e.height,"collected":e.collected})).collect::<Vec<_>>()})
    }
}

/// The scene uses CSS media-query scale steps, not continuous letterboxing.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub scale: f64,
    pub top: f64,
    pub stage_left: f64,
    pub wrap_left: f64,
    pub ground: f64,
    pub wall_height: f64,
}
impl Layout {
    pub fn new(width: f64, height: f64) -> Self {
        let mut scale: f64 = 1.;
        for (w, h, s) in [
            (1550., 926., 0.9),
            (1400., 836., 0.8),
            (1200., 717., 0.7),
            (1100., 657., 0.63),
            (1000., 597., 0.55),
            (800., 478., 0.4),
            (600., 358., 0.35),
            (400., 239., 0.25),
        ] {
            if width <= w || height <= h {
                scale = s;
            }
        }
        if width >= 2180. && height >= 1295. {
            scale = 1.5;
        }
        let wall_height = (810. * scale * 64.).floor() / 64.;
        let top = if height > wall_height * 1.5 {
            ((height - wall_height) * 0.35 * 64.).floor() / 64.
        } else {
            0.
        };
        let stage_left = width / 2. - 705. * scale;
        Self {
            scale,
            top,
            stage_left,
            wrap_left: stage_left - 352.5 * scale,
            ground: top + 810. * scale,
            wall_height,
        }
    }
}
#[cfg(test)]
mod visual_tests {
    use super::*;
    #[test]
    fn layout_matches_original_browser_at_five_viewports() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/runner-layout-reference.json")).unwrap();
        for (w, v) in data.as_object().unwrap() {
            let width = w.parse::<f64>().unwrap();
            let height = if width == 2400. { 1400. } else { 800. };
            let l = Layout::new(width, height);
            for (a, b) in [
                (l.top, v[".game"]["y"].as_f64().unwrap()),
                (l.stage_left, v[".stage"]["x"].as_f64().unwrap()),
                (l.top + 140. * l.scale, v[".stage"]["y"].as_f64().unwrap()),
                (
                    l.wrap_left + 450. * l.scale,
                    v[".reindeer"]["x"].as_f64().unwrap(),
                ),
                (
                    l.ground - 170. * l.scale,
                    v[".reindeer"]["y"].as_f64().unwrap(),
                ),
            ] {
                assert!((a - b).abs() < 0.03, "{width}: {a} vs {b}");
            }
        }
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;
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
    #[test]
    fn live_level_timers_and_background_match_source_helpers() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../data/runner-live-reference.json")).unwrap();
        let mut g = Runner::new("test1234");
        for (i, step) in cases.as_array().unwrap().iter().enumerate() {
            if step["op"].as_str() == Some("restart") {
                g.restart();
            }
            g.advance(step["duration"].as_f64().unwrap());
            let actual = serde_json::json!({"game":g.snapshot(),"mountain":g.mountain_x,"trees":g.trees_x,"updating":g.transition>0.});
            compare(&actual, &step["state"], &i.to_string());
        }
    }
}
