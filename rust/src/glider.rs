//! Native translation of the original Glider scene graph, FIFO pools and timers.
use crate::random::Random;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::VecDeque;
#[derive(Clone, Deserialize)]
pub struct BuildingSpec {
    pub width: f64,
    pub height: f64,
    pub originalwidth: f64,
    pub xgap: f64,
    pub ygap: f64,
    pub chimneys: Vec<ChimneySpec>,
}
#[derive(Clone, Deserialize)]
pub struct ChimneySpec {
    #[serde(rename = "type")]
    pub points: i32,
    pub xpos: f64,
    pub ypos: f64,
}
#[derive(Clone, Deserialize)]
pub struct ObjectSpec {
    pub css: String,
    pub fast: bool,
    pub width: f64,
    pub height: f64,
    pub xgap: f64,
    pub ygap: f64,
    #[serde(default)]
    pub score: f64,
    #[serde(default)]
    pub time: f64,
    #[serde(default, rename = "yVariance")]
    pub variance: f64,
    #[serde(default)]
    pub margintop: f64,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Item,
    Obstacle,
    Building,
    Cloud,
    Present,
    Collision,
}
impl Kind {
    fn pool(self) -> usize {
        match self {
            Self::Present => 0,
            Self::Collision => 1,
            Self::Item => 2,
            Self::Obstacle => 3,
            Self::Building => 4,
            Self::Cloud => 5,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Item => "Item",
            Self::Obstacle => "Obstacle",
            Self::Building => "Building",
            Self::Cloud => "Cloud",
            Self::Present => "Present",
            Self::Collision => "Collision",
        }
    }
}
#[derive(Clone)]
pub struct Entity {
    pub kind: Kind,
    pub style: usize,
    pub x: f64,
    pub y: f64,
    pub dead: bool,
    pub hit: bool,
    pub speed: f64,
    pub size: f64,
    pub variance: f64,
    pub back: bool,
    pub velocity: f64,
    pub chimney_hits: Vec<Option<f64>>,
    pub born: f64,
    pub anim_at: f64,
    pub hit_anim: f64,
    pub hit_at: f64,
    pub popping: bool,
}
#[derive(Clone)]
pub struct Player {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub shake: f64,
    pub has_present: bool,
    pub hit: bool,
    pub crash: bool,
    pub dropping: bool,
    pub crash_at: f64,
    pub drop_at: f64,
    pub anim_at: f64,
    pub anim_base: f64,
    pub hit_anim: f64,
    pub drop_anim: f64,
}
#[derive(Clone, Copy)]
enum Task {
    Present,
    Reload,
    CrashEnd,
    HitEnd,
    Pop(usize),
    CollisionEnd(usize),
}
pub struct Glider {
    pub width: f64,
    pub height: f64,
    pub level: i32,
    pub score: f64,
    pub countdown: f64,
    pub playing: bool,
    pub paused: bool,
    pub wall: f64,
    pub time: f64,
    pub bg_time: f64,
    pub css_time: f64,
    pub player_mode: u8,
    pub player_epoch: f64,
    pub player: Player,
    pub entities: Vec<Entity>,
    pub active: Vec<usize>,
    pub buildings: Vec<BuildingSpec>,
    pub items: Vec<ObjectSpec>,
    pub obstacles: Vec<ObjectSpec>,
    pub sounds: Vec<&'static str>,
    random: Random,
    pools: [VecDeque<usize>; 6],
    timers: Vec<(f64, Task)>,
    next_level: f64,
    building_speed: f64,
    next_item: f64,
    next_obstacle: f64,
    next_building: f64,
    next_cloud: f64,
    last_seconds: Option<f64>,
    losing: bool,
    pub input: (f64, f64, bool),
}
impl Glider {
    pub fn dimensions(w: f64, h: f64) -> (f64, f64, f64) {
        let scale = if w < 980. {
            w / 980.
        } else if w > 1200. {
            (w / 1200.).min(1.25)
        } else {
            1.
        };
        let em = 10. * scale;
        (w / em, h / em, em)
    }
    pub fn new(seed: &str, w: f64, h: f64) -> Self {
        let constants: Value =
            serde_json::from_str(include_str!("../data/glider-constants.json")).unwrap();
        let mut g = Self {
            width: w,
            height: h,
            level: 0,
            score: 0.,
            countdown: 60.,
            playing: true,
            paused: false,
            wall: 0.,
            time: 0.,
            bg_time: 0.,
            css_time: 0.,
            player_mode: 0,
            player_epoch: 0.,
            player: Player {
                x: 0.,
                y: h * 0.5,
                vx: 0.,
                vy: 0.,
                shake: 0.,
                has_present: true,
                hit: false,
                crash: false,
                dropping: false,
                crash_at: -100.,
                drop_at: -100.,
                anim_at: 0.1,
                anim_base: 0.1,
                hit_anim: 0.,
                drop_anim: 0.,
            },
            entities: vec![],
            active: vec![],
            buildings: serde_json::from_value(constants["BUILDINGS"].clone()).unwrap(),
            items: serde_json::from_value(constants["ITEM_TYPES"].clone()).unwrap(),
            obstacles: serde_json::from_value(constants["OBSTACLE_TYPES"].clone()).unwrap(),
            sounds: vec![],
            random: Random::seeded(seed),
            pools: std::array::from_fn(|_| VecDeque::new()),
            timers: vec![],
            next_level: 14.,
            building_speed: 13.,
            next_item: 0.,
            next_obstacle: 0.,
            next_building: 0.,
            next_cloud: 0.,
            last_seconds: None,
            losing: false,
            input: (0., 0., false),
        };
        for (k, n) in [
            (Kind::Present, 2),
            (Kind::Collision, 2),
            (Kind::Item, 5),
            (Kind::Obstacle, 12),
            (Kind::Building, 20),
            (Kind::Cloud, 20),
        ] {
            for _ in 0..n {
                let id = g.construct(k);
                g.pools[k.pool()].push_back(id)
            }
        }
        g.restart();
        g
    }
    fn construct(&mut self, kind: Kind) -> usize {
        let style = match kind {
            Kind::Present => (self.random.next() * 8.).ceil() as usize,
            Kind::Building => (self.random.next() * 26.).floor() as usize,
            _ => 0,
        };
        let id = self.entities.len();
        let chimney_hits = if kind == Kind::Building {
            vec![None; self.buildings[style].chimneys.len()]
        } else {
            vec![]
        };
        self.entities.push(Entity {
            kind,
            style,
            x: 0.,
            y: 0.,
            dead: true,
            hit: false,
            speed: if kind == Kind::Cloud { 60. } else { 0. },
            size: if kind == Kind::Cloud { 1. } else { 0. },
            variance: 0.,
            back: false,
            velocity: 0.,
            chimney_hits,
            born: 0.,
            anim_at: 0.,
            hit_anim: 0.,
            hit_at: 0.,
            popping: false,
        });
        id
    }
    fn pop(&mut self, kind: Kind) -> usize {
        let id = self.pools[kind.pool()]
            .pop_front()
            .unwrap_or_else(|| self.construct(kind));
        let e = &mut self.entities[id];
        e.dead = false;
        e.born = self.wall;
        e.anim_at = self.time;
        e.popping = false;
        match kind {
            Kind::Building => {
                e.chimney_hits.fill(None);
                e.x = self.width;
                e.y = self.height - self.buildings[e.style].height;
            }
            Kind::Item => {
                e.hit = false;
                e.style = if self.random.next() * 10. < 6. { 0 } else { 1 };
                let t = &self.items[e.style];
                e.speed = if t.fast { 15. } else { 7.5 } * 1.15_f64.powi(self.level);
                e.x = self.width;
                e.y = self.random.next() * self.height * 0.4;
            }
            Kind::Obstacle => {
                e.style = (self.random.next() * 5.).floor() as usize;
                let t = &self.obstacles[e.style];
                e.speed = if t.fast { 15. } else { 7.5 } * 1.25_f64.powi(self.level);
                e.x = self.width;
                e.y = self.random.next() * self.height * 0.4 + t.margintop;
                e.variance = 0.;
            }
            Kind::Cloud => {
                let t = (self.random.next() * 3.).floor();
                e.size = 1. - t * 0.25;
                e.speed = 120. - t * 30.;
                e.x = self.width;
                e.y = self.random.next() * self.height * 0.6;
            }
            Kind::Present => {
                e.velocity = 0.;
            }
            Kind::Collision => {}
        }
        self.active.push(id);
        id
    }
    fn remove(&mut self, id: usize) {
        self.entities[id].dead = true;
        let k = self.entities[id].kind;
        // display:none cancels the original CSS collision animation/end event.
        if k == Kind::Collision {
            self.timers
                .retain(|(_, t)| !matches!(t,Task::CollisionEnd(other)if *other==id));
        }
        self.pools[k.pool()].push_back(id);
    }
    pub fn restart(&mut self) {
        for id in self.active.clone() {
            self.remove(id);
        }
        self.active.clear();
        self.level = 0;
        self.score = 0.;
        self.countdown = 60.;
        self.losing = false;
        self.last_seconds = None;
        self.next_level = 14.;
        self.building_speed = 13.;
        self.next_item = 0.;
        self.next_obstacle = 0.;
        self.next_building = 0.;
        self.next_cloud = 0.;
        self.paused = false;
        self.playing = true;
        self.time = 0.;
        self.bg_time = 0.;
        self.css_time = 0.;
        self.player.x = 0.;
        self.player.y = self.height * 0.5;
        self.player.vx = 0.;
        self.player.vy = 0.;
        self.player.shake = 0.;
        self.player.anim_at = self.wall + 0.1;
        self.player.anim_base = self.time + 0.1;
        self.sounds.extend(["music_start_ingame", "glider_start"]);
        self.player_collisions();
        self.sync_player_animation();
    }
    fn sync_player_animation(&mut self) {
        let mode = if self.wall + 1e-12 < self.player.anim_at {
            0
        } else if self.player.hit {
            3
        } else if self.player.dropping {
            2
        } else {
            1
        };
        if mode != self.player_mode {
            self.player_mode = mode;
            self.player_epoch = self.css_time;
        }
    }
    pub fn drop_present(&mut self) {
        if !self.player.has_present {
            return;
        }
        self.player.has_present = false;
        self.player.dropping = true;
        self.player.drop_at = self.wall;
        self.player.drop_anim = self.time;
        self.timers.push((self.wall + 0.2, Task::Present));
        self.timers.push((self.wall + 0.5, Task::Reload));
        self.sounds.push("glider_presentdrop");
        self.sync_player_animation();
    }
    pub fn hit(&mut self) {
        if self.player.hit {
            return;
        }
        self.player.hit = true;
        self.player.crash = true;
        self.player.crash_at = self.wall;
        self.player.hit_anim = self.time;
        let id = self.pop(Kind::Collision);
        self.entities[id].x = self.player.x + 5.;
        self.entities[id].y = self.player.y + 2.;
        self.timers.push((self.wall + 0.5, Task::CollisionEnd(id)));
        self.timers.push((self.wall + 0.7, Task::CrashEnd));
        self.timers.push((self.wall + 3., Task::HitEnd));
        self.sync_player_animation();
    }
    pub fn wall_step(&mut self, dt: f64) {
        self.wall += dt;
        if self.playing {
            self.css_time += dt;
        }
        let (mut due, mut pending) = (vec![], vec![]);
        for (at, t) in std::mem::take(&mut self.timers) {
            if at <= self.wall + 1e-12 {
                due.push(t)
            } else {
                pending.push((at, t))
            }
        }
        self.timers = pending;
        for t in due {
            match t {
                Task::Present => {
                    self.player.dropping = false;
                    let id = self.pop(Kind::Present);
                    self.entities[id].x = self.player.x + 9.;
                    self.entities[id].y = self.player.y + 8.5;
                }
                Task::Reload => self.player.has_present = true,
                Task::CrashEnd => self.player.crash = false,
                Task::HitEnd => self.player.hit = false,
                Task::Pop(id) | Task::CollisionEnd(id) => {
                    self.entities[id].popping = false;
                    self.remove(id);
                }
            }
        }
        self.sync_player_animation();
    }
    pub fn pause(&mut self) {
        self.paused = true;
        self.playing = false;
    }
    pub fn resume(&mut self) {
        self.paused = false;
        self.playing = true;
    }
    pub fn step(&mut self, dt: f64) {
        if !self.playing {
            return;
        }
        self.time += dt;
        self.bg_time += dt;
        if self.input.2 {
            self.drop_present();
        }
        let (mut x, mut y) = (self.input.0, self.input.1);
        if self.player.x < 0. {
            x = 1.
        } else if self.player.x > self.width - 10. {
            x = -1.
        }
        if self.player.y < 0. {
            y = 1.
        } else if self.player.y > self.height - 4. {
            y = -1.
        }
        let mov = |v: f64, t: f64| {
            if v < t {
                t.min(v + dt * 3.)
            } else {
                t.max(v - dt * 3.)
            }
        };
        self.player.vx = mov(self.player.vx, x);
        self.player.vy = mov(self.player.vy, y);
        self.player.shake += dt;
        self.player.x += self.player.vx * 90. * dt + (self.player.shake * 3.).sin() * dt;
        self.player.y += self.player.vy * 90. * dt + (self.player.shake * 4.).sin() * dt;
        self.player_collisions();
        self.countdown -= dt;
        let seconds = self.countdown.ceil();
        if self.last_seconds != Some(seconds) {
            self.last_seconds = Some(seconds);
            if dt > 0. {
                if self.countdown < 0. {
                    self.countdown = 0.;
                    self.playing = false;
                    self.sounds
                        .extend(["glider_game_over", "music_ingame_gameover"]);
                }
                let losing = seconds <= 10. && seconds != 0.;
                if losing != self.losing {
                    self.losing = losing;
                    if seconds > 0. {
                        self.sounds.push(if losing {
                            "game_hurry_up"
                        } else {
                            "game_hurry_up_end"
                        });
                    }
                }
            }
        }
        self.next_item -= dt;
        if self.next_item <= 0. {
            self.pop(Kind::Item);
            let m = 0.95_f64.powi(self.level);
            self.next_item = 6. * m + self.random.next() * 2. * m;
        }
        self.next_obstacle -= dt;
        if self.next_obstacle <= 0. {
            self.pop(Kind::Obstacle);
            let m = 0.95_f64.powi(self.level);
            self.next_obstacle = 6. * m + self.random.next() * 2. * m;
        }
        self.next_building -= dt;
        if self.next_building <= 0. {
            self.pop(Kind::Building);
            self.next_building = 0.5 + 2.7 * 0.7_f64.powi(self.level);
        }
        self.next_cloud -= dt;
        if self.next_cloud <= 0. {
            self.pop(Kind::Cloud);
            self.next_cloud = 3.25 + self.random.next() * 2.5;
        }
        self.next_level -= dt;
        if self.next_level <= 0. {
            self.level += 1;
            self.building_speed += 5.;
            self.next_level = 14.;
        }
        for id in self.active.clone() {
            self.entity_frame(id, dt);
        }
        self.active.retain(|id| !self.entities[*id].dead);
    }
    fn entity_frame(&mut self, id: usize, dt: f64) {
        match self.entities[id].kind {
            Kind::Building => {
                let e = &mut self.entities[id];
                e.x -= self.building_speed * dt;
                e.y = self.height - self.buildings[e.style].height;
                if e.x <= -self.buildings[e.style].originalwidth {
                    self.remove(id);
                }
            }
            Kind::Item => {
                let e = &mut self.entities[id];
                e.x -= e.speed * dt;
                if e.x <= -self.items[e.style].width {
                    self.remove(id);
                }
            }
            Kind::Obstacle => {
                let e = &mut self.entities[id];
                e.x -= e.speed * dt;
                if e.back {
                    e.variance -= e.speed * dt;
                    if e.variance <= 0. {
                        e.back = false;
                    }
                } else {
                    e.variance += e.speed * dt;
                    if e.variance >= self.obstacles[e.style].variance {
                        e.back = true;
                    }
                }
                if e.x <= -self.obstacles[e.style].width {
                    self.remove(id);
                }
            }
            Kind::Cloud => {
                let e = &mut self.entities[id];
                e.x -= e.speed / 10. * dt;
                if e.x <= -18. {
                    self.remove(id);
                }
            }
            Kind::Present => {
                let e = &mut self.entities[id];
                if e.y < self.height {
                    e.velocity += 200. * dt;
                    e.y += e.velocity * dt;
                } else {
                    self.remove(id);
                }
                self.present_collisions(id);
            }
            Kind::Collision => {}
        }
    }
    fn player_collisions(&mut self) {
        if self.player.hit {
            return;
        }
        let (px, py) = (self.player.x + 5., self.player.y + 5.);
        let ids = self.active.clone();
        for id in ids {
            let e = &self.entities[id];
            let (w, h, gx, gy, variance) = match e.kind {
                Kind::Building => {
                    let b = &self.buildings[e.style];
                    (b.width, b.height, b.xgap, b.ygap, 0.)
                }
                Kind::Item if !e.hit => {
                    let t = &self.items[e.style];
                    (t.width, t.height, t.xgap, t.ygap, 0.)
                }
                Kind::Obstacle => {
                    let t = &self.obstacles[e.style];
                    (t.width, t.height, t.xgap, t.ygap, e.variance)
                }
                _ => continue,
            };
            if px < e.x + gx + w
                && px + 10. > e.x + gx
                && py < e.y + gy + variance + h
                && py + 4. > e.y + gy + variance
            {
                if e.kind == Kind::Item {
                    let e = &mut self.entities[id];
                    e.hit = true;
                    e.hit_at = self.wall;
                    let t = &self.items[e.style];
                    self.score += t.score * (self.level + 1) as f64;
                    self.countdown += t.time;
                    self.sounds.push(if t.time > 0. && t.score == 0. {
                        "glider_clock"
                    } else {
                        "generic_score"
                    });
                } else {
                    let balloon = e.kind == Kind::Obstacle && e.style == 0;
                    self.hit();
                    self.sounds.push("glider_hit");
                    if balloon {
                        self.entities[id].popping = true;
                        self.entities[id].hit_at = self.wall;
                        self.entities[id].hit_anim = self.time;
                        self.timers.push((self.wall + 0.5, Task::Pop(id)));
                    }
                }
            }
        }
    }
    fn present_collisions(&mut self, id: usize) {
        let (px, py) = (self.entities[id].x, self.entities[id].y);
        for b in self.active.clone() {
            if self.entities[b].kind != Kind::Building {
                continue;
            }
            for i in 0..self.buildings[self.entities[b].style].chimneys.len() {
                if self.entities[b].chimney_hits[i].is_some() {
                    continue;
                }
                let spec = &self.buildings[self.entities[b].style].chimneys[i];
                let (cx, cy) = (
                    self.entities[b].x + spec.xpos,
                    self.entities[b].y + spec.ypos,
                );
                let w = match spec.points {
                    100 => 5.,
                    200 => 4.,
                    _ => 3.,
                };
                let points = spec.points;
                if px < cx + w && px + 2.5 > cx && py < cy + 1. && py + 2.5 > cy {
                    self.remove(id);
                    self.entities[b].chimney_hits[i] = Some(self.wall);
                    self.score += (points * (self.level + 1)) as f64;
                    self.sounds.push("glider_chimney_hit");
                }
            }
        }
    }
    pub fn advance(&mut self, duration: f64) {
        let mut left = duration;
        while left > 1e-12 {
            let dt = left.min(1. / 60.);
            self.wall_step(dt);
            self.step(dt);
            left -= dt;
        }
    }
    pub fn snapshot(&self) -> Value {
        json!({"x":self.player.x,"y":self.player.y,"vx":self.player.vx,"vy":self.player.vy,"shake":self.player.shake,"has_present":self.player.has_present,"hit":self.player.hit,"score":self.score,"countdown":self.countdown,"level":self.level,"next_level":self.next_level,"building_speed":self.building_speed,"next_item":self.next_item,"next_obstacle":self.next_obstacle,"next_building":self.next_building,"next_cloud":self.next_cloud,"playing":self.playing,"entities":self.active.iter().map(|id|{let e=&self.entities[*id];let style=match e.kind{Kind::Building=>json!(e.style),Kind::Present=>json!(0),Kind::Item=>json!(self.items[e.style].css),Kind::Obstacle=>json!(self.obstacles[e.style].css),_=>json!("")};json!({"kind":e.kind.name(),"x":e.x,"y":e.y,"dead":e.dead,"style":style,"speed":e.speed,"size":e.size,"hit":e.hit,"variance":e.variance,"back":e.back,"velocity":e.velocity,"chimneys":if e.kind==Kind::Building{self.buildings[e.style].chimneys.iter().enumerate().map(|(i,c)|json!({"x":c.xpos,"y":c.ypos,"type":c.points,"hit":e.chimney_hits[i].is_some()})).collect::<Vec<_>>()}else{vec![]}})}).collect::<Vec<_>>()})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn compare(a: &Value, b: &Value, path: &str) {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-7,
                "{path}: {a} vs {b}"
            ),
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}[{i}]"));
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(a.len(), b.len());
                for (k, b) in b {
                    compare(&a[k], b, &format!("{path}.{k}"));
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    #[test]
    fn original_glider_state_scenarios() {
        let cases: Value =
            serde_json::from_str(include_str!("../data/glider-reference.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let mut g = Glider::new("test1234", 120., 800. / 12.);
            for (i, row) in case["steps"].as_array().unwrap().iter().enumerate() {
                let v = &row["value"];
                match row["action"].as_str().unwrap() {
                    "clock" => {
                        let id = g.pop(Kind::Item);
                        g.entities[id].style = 1;
                        g.entities[id].speed = 15. * 1.15_f64.powi(g.level);
                        g.entities[id].x = g.player.x + 5.;
                        g.entities[id].y = g.player.y + 5.;
                    }
                    "catch-first" => {
                        if let Some(id) =
                            g.active.iter().copied().find(|id| {
                                g.entities[*id].kind == Kind::Item && !g.entities[*id].hit
                            })
                        {
                            g.entities[id].x = g.player.x + 5.;
                            g.entities[id].y = g.player.y + 5.;
                        }
                    }
                    "land" => {
                        if let Some(b) = g.active.iter().copied().find(|id| {
                            g.entities[*id].kind == Kind::Building
                                && g.entities[*id].chimney_hits.iter().any(Option::is_none)
                        }) {
                            let i = g.entities[b]
                                .chimney_hits
                                .iter()
                                .position(Option::is_none)
                                .unwrap();
                            let a = &g.buildings[g.entities[b].style].chimneys[i];
                            let (x, y) = (
                                g.entities[b].x + a.xpos + 0.2,
                                g.entities[b].y + a.ypos - 2.6,
                            );
                            let id = g.pop(Kind::Present);
                            g.entities[id].x = x;
                            g.entities[id].y = y;
                        }
                    }
                    "balloon" => {
                        if let Some(id) = g
                            .active
                            .iter()
                            .copied()
                            .find(|id| g.entities[*id].kind == Kind::Obstacle)
                        {
                            g.entities[id].style = 0;
                            g.entities[id].x = g.player.x + 3.;
                            g.entities[id].y = g.player.y + 5. - g.entities[id].variance;
                        }
                    }
                    "keys" => {
                        let yes = |s: &str| v[s].as_bool().unwrap_or(false);
                        g.input = (
                            (i32::from(yes("right")) - i32::from(yes("left"))) as f64 * 0.6,
                            (i32::from(yes("down")) - i32::from(yes("up"))) as f64 * 0.6,
                            yes("space"),
                        );
                    }
                    "drop" => g.drop_present(),
                    "hit" => g.hit(),
                    "pause" => g.pause(),
                    "resume" => g.resume(),
                    "restart" => g.restart(),
                    "time" => g.countdown += v.as_f64().unwrap(),
                    "place" => {
                        g.player.x = v[0].as_f64().unwrap();
                        g.player.y = v[1].as_f64().unwrap();
                        g.player.vx = 0.;
                        g.player.vy = 0.;
                    }
                    _ => {}
                }
                g.advance(row["duration"].as_f64().unwrap());
                compare(
                    &g.snapshot(),
                    &row["state"],
                    &format!("{} step {i}", case["name"]),
                );
            }
        }
    }
}
/// Source device-motion branch. Native hardware sensor delivery is platform work;
/// callers with a sensor sample can use the same asymmetric landscape thresholds.
pub fn motion_input(orientation: i32, ax: f64, ay: f64) -> (f64, f64) {
    let mut x = 0.;
    let mut y = 0.;
    if orientation == 90 {
        if ay > 1. && ax > 1. {
            x -= 0.5;
            y -= 0.5;
        } else if ay < -1. && ax > 1. {
            x += 0.5;
            y -= 0.5;
        } else if ay < -1. && ax < -1. {
            x += 0.5;
            y += 0.5;
        } else if ay > 1. && ax < -1. {
            x -= 0.5;
            y += 0.5;
        } else if ay > 1. {
            x -= 0.5;
        } else if ay < -1. {
            x += 0.5;
        } else if ax < -1. {
            y += 0.5;
        } else if ax > 1. {
            y -= 0.5;
        }
    } else if orientation == -90 {
        if ay > 2. && ax > 2. {
            x += 0.5;
            y += 0.5;
        } else if ay < -2. && ax > 2. {
            x -= 0.5;
            y += 0.5;
        } else if ay < -2. && ax < -2. {
            x -= 0.5;
            y -= 0.5;
        } else if ay > 2. && ax < -2. {
            x += 0.5;
            y -= 0.5;
        } else if ay > 2. {
            x += 0.5;
        } else if ay < -2. {
            x -= 0.5;
        } else if ax < -2. {
            y -= 0.5;
        } else if ax > 2. {
            y += 0.5;
        }
    }
    (x, y)
}
#[cfg(test)]
mod motion_tests {
    #[test]
    fn source_landscape_motion_thresholds() {
        let rows: serde_json::Value =
            serde_json::from_str(include_str!("../data/glider-motion-reference.json")).unwrap();
        for r in rows.as_array().unwrap() {
            let p = super::motion_input(
                r["orientation"].as_i64().unwrap() as i32,
                r["x"].as_f64().unwrap(),
                r["y"].as_f64().unwrap(),
            );
            assert_eq!(p.0, r["result"][0].as_f64().unwrap(), "{r}");
            assert_eq!(p.1, r["result"][1].as_f64().unwrap(), "{r}");
        }
    }
}
#[cfg(test)]
mod css_lifecycle_tests {
    use super::*;
    #[test]
    fn source_animation_name_lifecycle() {
        let rows: Value = serde_json::from_str(include_str!(
            "../data/glider-animation-lifecycle-reference.json"
        ))
        .unwrap();
        let mut g = Glider::new("test1234", 120., 800. / 12.);
        for r in rows.as_array().unwrap() {
            match r["action"].as_str().unwrap() {
                "drop" => g.drop_present(),
                "hit" => g.hit(),
                "pause" => g.pause(),
                "resume" => g.resume(),
                "restart" => g.restart(),
                _ => {}
            }
            g.advance(r["duration"].as_f64().unwrap());
            let name = match g.player_mode {
                0 => "none",
                1 => "module-glider-playernormal",
                2 => "module-glider-playerdrop",
                _ => "module-glider-playerpulse",
            };
            assert_eq!(name, r["animation"].as_str().unwrap(), "{r}");
        }
    }
}
