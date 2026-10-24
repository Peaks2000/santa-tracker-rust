//! Original Gift Slingshot simulation, entity pools, coordinator and controls.
use crate::random::Random;
use serde::Serialize;
use std::collections::VecDeque;
const BOATS: [(f64, f64, f64); 3] = [(195., 89., 0.8), (164., 81., 1.), (173., 81., 1.2)];
const ICE: [(f64, f64, f64); 6] = [
    (162., 98., 0.9),
    (57., 49., 1.4),
    (120., 111., 0.8),
    (73., 64., 1.3),
    (124., 95., 1.1),
    (87., 84., 1.3),
];
#[derive(Clone, Serialize)]
pub struct Boat {
    pub kind: usize,
    pub y: f64,
    pub height: f64,
    pub width: f64,
    pub speed: f64,
    pub hits: u32,
    pub next_bubble: f64,
    pub dead: bool,
}
#[derive(Clone, Serialize)]
pub struct Iceberg {
    pub kind: usize,
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub width: f64,
    pub speed: f64,
    pub presents: Vec<usize>,
    pub dead: bool,
}
#[derive(Clone, Serialize)]
pub struct Present {
    pub kind: u8,
    pub x: f64,
    pub y: f64,
    pub add_x: f64,
    pub velocity: f64,
    pub scale: f64,
    pub dead: bool,
    pub visible: bool,
    pub missed: Option<f64>,
}
#[derive(Clone, Copy, Serialize)]
pub enum Entity {
    Boat(usize),
    Ice(usize),
    Present(usize),
}
#[derive(Clone, Copy)]
enum Timer {
    Prepare(f64),
    Throw(f64),
    Sink(usize, f64),
}
#[derive(Clone, Serialize)]
pub struct Player {
    pub y: f64,
    pub target_y: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub distance: f64,
    pub present: Option<usize>,
    pub waiting: bool,
    pub preparing: bool,
    pub prepared: bool,
    pub throwing: bool,
    pub drop_when_prepared: bool,
    pub drop_when_stopped: bool,
    pub auto_prepare: bool,
    pub animation: f64,
}
pub struct Bubble {
    pub kind: u8,
    pub x: f64,
    pub y: f64,
    pub age: f64,
}
pub struct Boatload {
    pub bubbles: Vec<Bubble>,
    bubble_pool: VecDeque<u8>,
    pub player: Player,
    pub boats: Vec<Boat>,
    pub icebergs: Vec<Iceberg>,
    pub presents: Vec<Present>,
    pub entities: Vec<Entity>,
    pub level: u32,
    pub next_level: f64,
    pub score: u64,
    pub countdown: f64,
    pub playing: bool,
    pub paused: bool,
    pub height: f64,
    pub scale: f64,
    pub time: f64,
    pub boat_speed: f64,
    pub iceberg_speed: f64,
    pub next_boat: f64,
    pub next_iceberg: f64,
    pub sounds: Vec<&'static str>,
    pub effects: crate::boatload_effects::Effects,
    random: Random,
    boat_pool: VecDeque<usize>,
    ice_pool: VecDeque<usize>,
    present_pool: VecDeque<usize>,
    timers: Vec<Timer>,
    last_boat: Option<usize>,
    next_boat_type: Option<usize>,
    refills: Vec<f64>,
    last_missed: Option<usize>,
    charging: bool,
    charge_accum: f64,
    charge_count: f64,
    last_seconds: Option<f64>,
    losing: bool,
}
impl Boatload {
    pub fn new(seed: &str, width: f64, height: f64) -> Self {
        let scale = (width / 980.).min(height / 600.).min(1.);
        let height = height / scale;
        let mut g = Self {
            bubbles: vec![],
            bubble_pool: VecDeque::new(),
            player: Player {
                y: height / 2.,
                target_y: height / 2.,
                min_y: height * 0.15,
                max_y: height * 0.82,
                distance: 0.,
                present: None,
                waiting: false,
                preparing: false,
                prepared: false,
                throwing: false,
                drop_when_prepared: false,
                drop_when_stopped: false,
                auto_prepare: false,
                animation: 0.,
            },
            boats: vec![],
            icebergs: vec![],
            presents: vec![],
            entities: vec![],
            level: 0,
            next_level: 21.,
            score: 0,
            countdown: 60.,
            playing: true,
            paused: false,
            height,
            scale,
            time: 0.,
            boat_speed: 50.,
            iceberg_speed: 60.,
            next_boat: 0.,
            next_iceberg: 0.,
            sounds: vec!["bl_game_start", "music_start_ingame"],
            effects: Default::default(),
            random: Random::seeded(seed),
            boat_pool: VecDeque::new(),
            ice_pool: VecDeque::new(),
            present_pool: VecDeque::new(),
            timers: vec![],
            last_boat: None,
            next_boat_type: None,
            refills: vec![],
            last_missed: None,
            charging: false,
            charge_accum: 0.,
            charge_count: 0.,
            last_seconds: None,
            losing: false,
        };
        let held = g.pop_present(height / 2. - 10.);
        g.player.present = Some(held);
        for _ in 0..5 {
            g.pool_boat();
        }
        for _ in 0..7 {
            g.pool_ice();
            g.pool_present();
        }
        for _ in 0..30 {
            let kind = (g.random.next() * 6.).ceil() as u8;
            g.bubble_pool.push_back(kind);
        } // Original bubble constructor types.
        g
    }
    fn pool_present(&mut self) {
        let kind = (self.random.next() * 3.).ceil() as u8;
        let i = self.presents.len();
        self.presents.push(Present {
            kind,
            x: 0.,
            y: 0.,
            add_x: 0.,
            velocity: 0.,
            scale: 1.,
            dead: false,
            visible: false,
            missed: None,
        });
        self.present_pool.push_back(i);
    }
    fn pop_present(&mut self, y: f64) -> usize {
        if self.present_pool.is_empty() {
            self.pool_present();
        }
        let i = self.present_pool.pop_front().unwrap();
        let p = &mut self.presents[i];
        p.x = 166.;
        p.y = y;
        p.add_x = 0.;
        p.velocity = 400.;
        p.scale = 1.;
        p.dead = false;
        p.visible = true;
        p.missed = None;
        i
    }
    fn remove_present(&mut self, i: usize) {
        self.presents[i].dead = true;
        self.presents[i].visible = false;
        self.present_pool.push_back(i);
    }
    fn pool_boat(&mut self) {
        let i = self.boats.len();
        self.boats.push(Boat {
            kind: 0,
            y: 0.,
            height: 0.,
            width: 0.,
            speed: 0.,
            hits: 0,
            next_bubble: 0.,
            dead: true,
        });
        self.boat_pool.push_back(i);
    }
    fn pool_ice(&mut self) {
        let i = self.icebergs.len();
        self.icebergs.push(Iceberg {
            kind: 0,
            x: 0.,
            y: 0.,
            height: 0.,
            width: 0.,
            speed: 0.,
            presents: vec![],
            dead: true,
        });
        self.ice_pool.push_back(i);
    }
    pub fn resize(&mut self, width: f64, height: f64) {
        let scale = (width / 980.).min(height / 600.).min(1.);
        let h = height / scale;
        if self.height != h || self.scale != scale {
            self.scale = scale;
            self.height = h;
            self.reset_player();
        }
    }
    fn reset_player(&mut self) {
        self.player.y = self.height / 2.;
        self.player.target_y = self.player.y;
        self.player.min_y = self.height * 0.15;
        self.player.max_y = self.height * 0.82;
        if let Some(i) = self.player.present {
            let p = &mut self.presents[i];
            p.x = 166.;
            p.y = self.player.y - 10.;
            p.add_x = 0.;
            p.velocity = 400.;
            p.scale = 1.;
            p.dead = false;
            p.visible = true;
            p.missed = None;
        } else {
            let i = self.pop_present(self.player.y - 10.);
            self.player.present = Some(i);
        }
        self.player.throwing = false;
        self.player.waiting = false;
        self.player.prepared = false;
        self.player.preparing = false;
        self.timers.clear();
        self.animate(0.);
    }
    pub fn restart(&mut self) {
        for e in self.entities.clone() {
            match e {
                Entity::Boat(i) => {
                    self.boats[i].dead = true;
                    self.boat_pool.push_back(i);
                }
                Entity::Ice(i) => {
                    self.icebergs[i].dead = true;
                    let presents = std::mem::take(&mut self.icebergs[i].presents);
                    for p in presents {
                        self.remove_present(p);
                    }
                    self.ice_pool.push_back(i);
                }
                Entity::Present(i) => self.remove_present(i),
            }
        }
        self.entities.clear();
        // Latest missed present is removed on restart in the source.
        if let Some(i) = self.last_missed.take() {
            self.remove_present(i);
        }

        self.level = 0;
        self.next_level = 21.;
        self.score = 0;
        self.countdown = 60.;
        self.last_seconds = None;
        self.losing = false;
        self.boat_speed = 50.;
        self.iceberg_speed = 60.;
        self.next_boat = 0.;
        self.next_iceberg = 0.;
        self.next_boat_type = None;
        self.last_boat = None;
        self.paused = false;
        self.playing = true;
        self.reset_player();
        self.sounds.extend(["bl_game_start", "music_start_ingame"]);
    }
    pub fn set_y(&mut self, y: f64) {
        self.player.target_y = y.clamp(self.player.min_y, self.player.max_y);
    }
    pub fn keys(&mut self, up: bool, down: bool) {
        if self.playing && !self.paused {
            self.set_y(if up {
                self.player.min_y
            } else if down {
                self.player.max_y
            } else {
                self.player.y
            });
        }
    }
    pub fn begin_charge(&mut self) {
        if !self.playing || self.paused {
            return;
        }
        self.charging = true;
        self.charge_accum = 0.;
        self.charge_count = 0.;
        self.player.distance = 0.;
        self.prepare();
    }
    pub fn release(&mut self, touch: bool) {
        self.charging = false;
        if !self.playing || self.paused {
            return;
        }
        if touch && self.player.y != self.player.target_y {
            self.player.drop_when_stopped = true;
        } else {
            self.drop();
        }
    }
    fn prepare(&mut self) {
        if self.player.waiting {
            self.player.auto_prepare = true;
            return;
        }
        if self.player.preparing || self.player.throwing || self.player.present.is_none() {
            return;
        }
        self.player.preparing = true;
        self.sounds.push("bl_rope_stretch");
        self.timers.push(Timer::Prepare(0.3));
    }
    fn drop(&mut self) {
        if self.player.prepared {
            self.player.prepared = false;
            self.throw();
        } else if self.player.preparing {
            self.player.drop_when_prepared = true;
        }
        self.player.auto_prepare = false;
    }
    fn throw(&mut self) {
        self.player.throwing = true;
        self.timers.push(Timer::Throw(0.1));
    }
    /// Browser setInterval/setTimeout continue while game simulation is paused.
    pub fn wall_step(&mut self, dt: f64) {
        self.effects.advance(dt, self.playing && !self.paused);
        for b in &mut self.bubbles {
            b.age += dt;
        }
        let mut b = 0;
        while b < self.bubbles.len() {
            if self.bubbles[b].age >= 0.6 {
                let old = self.bubbles.remove(b);
                self.bubble_pool.push_back(old.kind);
            } else {
                b += 1;
            }
        }
        if self.charging {
            self.charge_accum += dt;
            while self.charge_accum + 1e-12 >= 0.01 {
                self.charge_accum = (self.charge_accum - 0.01).max(0.);
                self.charge_count += 0.15;
                self.player.distance = self.charge_count.min(3.);
            }
        }
        let mut pending = std::mem::take(&mut self.refills);
        for left in &mut pending {
            *left -= dt;
            if *left <= 0. && self.player.present.is_none() {
                let p = self.pop_present(self.player.y - 10.);
                self.player.present = Some(p);
                self.player.waiting = false;
                if self.player.auto_prepare {
                    self.prepare();
                }
            }
        }
        pending.retain(|left| *left > 0.);
        self.refills = pending;
    }

    fn animate(&mut self, status: f64) {
        self.player.animation = status;
        if let Some(i) = self.player.present {
            self.presents[i].add_x = -self.player.distance * 20. * status;
        }
    }
    fn coordinator(&mut self, dt: f64) {
        let mut i = 0;
        while i < self.timers.len() {
            let timer = self.timers[i];
            let (rem, total, reverse) = match timer {
                Timer::Prepare(r) => (r - dt, 0.3, false),
                Timer::Throw(r) => (r - dt, 0.1, true),
                Timer::Sink(_, r) => (r - dt, 0.6, false),
            };
            let rem = rem.max(0.);
            let progress = if reverse {
                rem / total
            } else {
                1. - rem / total
            };
            match timer {
                Timer::Prepare(_) | Timer::Throw(_) => self.animate(progress),
                Timer::Sink(p, _) => {
                    self.presents[p].add_x = progress * 20.;
                    self.presents[p].missed = Some(progress);
                }
            }
            if rem <= 0. {
                match timer {
                    Timer::Prepare(_) => {
                        self.player.preparing = false;
                        if self.player.drop_when_prepared {
                            self.player.drop_when_prepared = false;
                            self.throw();
                        } else {
                            self.player.prepared = true;
                        }
                    }
                    Timer::Throw(_) => {
                        if let Some(p) = self.player.present.take() {
                            self.presents[p].velocity = if self.player.distance != 0. {
                                self.player.distance * 350.
                            } else {
                                400.
                            };
                            self.entities.push(Entity::Present(p));
                            self.sounds.push("bl_shoot");
                        }
                        self.player.throwing = false;
                        self.player.waiting = true;
                        self.refills.push(0.5);
                    }
                    Timer::Sink(p, _) => {
                        self.remove_present(p);
                        self.last_missed = None;
                    }
                }
                self.timers.remove(i);
            } else {
                self.timers[i] = match timer {
                    Timer::Prepare(_) => Timer::Prepare(rem),
                    Timer::Throw(_) => Timer::Throw(rem),
                    Timer::Sink(p, _) => Timer::Sink(p, rem),
                };
                i += 1;
            }
        }
    }
    fn spawn(&mut self, dt: f64) {
        self.next_boat -= dt;
        if self.next_boat <= 0. {
            let kind = if let Some(k) = self.next_boat_type {
                k
            } else {
                let k = (self.random.next() * 3.).ceil() as usize - 1;
                self.next_boat_type = Some(k);
                k
            };
            let mut allowed = true;
            if let Some(i) = self.last_boat {
                let last = &self.boats[i];
                let lt = (last.y + last.height) / (last.speed * self.boat_speed);
                let nt = self.height / (BOATS[kind].2 * self.boat_speed);
                if lt > nt {
                    self.next_boat = lt - nt;
                    allowed = false;
                }
            }
            if allowed {
                if self.boat_pool.is_empty() {
                    self.pool_boat();
                }
                let i = self.boat_pool.pop_front().unwrap();
                let (h, w, s) = BOATS[kind];
                self.boats[i] = Boat {
                    kind,
                    y: self.height,
                    height: h,
                    width: w,
                    speed: s,
                    hits: 0,
                    next_bubble: 0.,
                    dead: false,
                };
                self.entities.push(Entity::Boat(i));
                self.last_boat = Some(i);
                self.next_boat_type = None;
                self.next_boat =
                    (6.5 + self.random.next() * 2.).min((self.height + h) / (s * self.boat_speed));
            }
        }
        self.next_iceberg -= dt;
        if self.next_iceberg > 0. {
            return;
        }
        let kind = (self.random.next() * 6.).ceil() as usize - 1;
        let x = 500. + self.random.next() * 250. - 125.;
        let (h, w, s) = ICE[kind];
        let timeleft = (self.height - h) / s;
        for e in &self.entities {
            if let Entity::Ice(i) = *e {
                let o = &self.icebergs[i];
                if timeleft < (self.height + o.height - o.y) / o.speed {
                    self.next_iceberg = 1.;
                    return;
                }
            }
        }
        if self.ice_pool.is_empty() {
            self.pool_ice();
        }
        let i = self.ice_pool.pop_front().unwrap();
        self.icebergs[i] = Iceberg {
            kind,
            x,
            y: 0.,
            height: h,
            width: w,
            speed: s,
            presents: vec![],
            dead: false,
        };
        self.entities.push(Entity::Ice(i));
        self.next_iceberg = 2.5 + self.random.next() * 2.;
    }
    pub fn step(&mut self, dt: f64) {
        if !self.playing || self.paused {
            return;
        }
        self.time += dt;
        self.coordinator(dt);
        if self.player.target_y != self.player.y {
            let movement = dt * 750.;
            if movement > (self.player.target_y - self.player.y).abs() {
                self.player.y = self.player.target_y;
                if self.player.drop_when_stopped {
                    self.player.drop_when_stopped = false;
                    self.drop();
                }
            } else {
                self.player.y += movement
                    * if self.player.target_y > self.player.y {
                        1.
                    } else {
                        -1.
                    };
            }
            if let Some(p) = self.player.present {
                self.presents[p].y = self.player.y - 10.;
            }
        }
        self.next_level -= dt;
        if self.next_level <= 0. {
            if self.level == 9 {
                self.gameover();
            } else {
                self.level += 1;
                self.boat_speed += 10.;
                self.iceberg_speed += 5.;
                self.next_level = 21.;
            }
        }
        self.countdown -= dt;
        let seconds = self.countdown.ceil();
        if self.last_seconds != Some(seconds) {
            self.last_seconds = Some(seconds);
            if dt > 0. {
                if self.countdown < 0. {
                    self.countdown = 0.;
                    self.gameover();
                }
                let losing = seconds <= 10. && seconds != 0.;
                if self.losing != losing {
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
        self.spawn(dt);
        let mut dead = vec![];
        let mut n = 0;
        while n < self.entities.len() {
            let e = self.entities[n];
            match e {
                Entity::Boat(i) => {
                    let b = &mut self.boats[i];
                    b.y -= self.boat_speed * b.speed * dt;
                    if b.y <= -b.height {
                        b.dead = true;
                        self.boat_pool.push_back(i);
                    }
                    b.next_bubble -= dt;
                    if b.next_bubble <= 0. {
                        let x = self.random.next() * 10. - 5.;
                        let kind = self
                            .bubble_pool
                            .pop_front()
                            .unwrap_or_else(|| (self.random.next() * 6.).ceil() as u8);
                        self.bubbles.push(Bubble {
                            kind,
                            x,
                            y: b.y + b.height - 6.,
                            age: 0.,
                        });
                        b.next_bubble = 0.105 + self.random.next() * 0.05;
                    }
                    if b.dead {
                        dead.push(n);
                    }
                }
                Entity::Ice(i) => {
                    let speed = self.iceberg_speed * self.icebergs[i].speed * dt;
                    self.icebergs[i].y += speed;
                    if self.icebergs[i].y >= self.height + self.icebergs[i].height {
                        self.icebergs[i].dead = true;
                        let ps = std::mem::take(&mut self.icebergs[i].presents);
                        for p in ps {
                            self.remove_present(p);
                        }
                        self.ice_pool.push_back(i);
                        dead.push(n);
                    }
                    for p in &self.icebergs[i].presents {
                        self.presents[*p].y += speed;
                    }
                }
                Entity::Present(i) => {
                    if self.presents[i].x >= 830. {
                        self.presents[i].dead = true;
                        self.presents[i].missed = Some(0.);
                        self.last_missed = Some(i);
                        let p = &self.presents[i];
                        self.effects.miss(p.x - 10., p.y - 10.);
                        self.timers.push(Timer::Sink(i, 0.6));
                        self.sounds.push("bl_hit_water");
                    } else {
                        self.presents[i].x =
                            (self.presents[i].x + self.presents[i].velocity * dt / 3.).min(830.);
                        let x = self.presents[i].x;
                        let y = self.presents[i].y;
                        let list = self.entities.clone();
                        for other in list {
                            let hit = match other {
                                Entity::Boat(b) => {
                                    let b = &self.boats[b];
                                    (850., b.y, b.height / 2. - 10.)
                                }
                                Entity::Ice(b) => {
                                    let b = &self.icebergs[b];
                                    (
                                        b.x + b.width / 2. - 5.,
                                        b.y - b.height + 10.,
                                        b.height / 2. - 20.,
                                    )
                                }
                                _ => continue,
                            };
                            if hit.0 < x
                                || hit.0 >= x + 30.
                                || (y - hit.1 - hit.2).abs() > hit.2 - 10. + 20.
                            {
                                continue;
                            }
                            match other {
                                Entity::Boat(b) => {
                                    self.remove_present(i);
                                    self.boats[b].hits += 1;
                                    let hits = self.boats[b].hits;
                                    let score =
                                        50 * (1 + self.level as u64) * 2u64.pow(hits.min(3));
                                    self.score += score;
                                    self.countdown += if hits > 1 { 1. } else { 2. };
                                    self.effects.hit(x + 10., y + 10., score);
                                    self.sounds.push(if self.boats[b].kind == 0 {
                                        "bl_score_yellow"
                                    } else {
                                        "bl_score_red"
                                    });
                                }
                                Entity::Ice(b) => {
                                    self.presents[i].dead = true;
                                    self.icebergs[b].presents.push(i);
                                    self.sounds.push("bl_hit_ice");
                                }
                                _ => {}
                            }
                        }
                        self.presents[i].scale = 1.5 - ((x - 166.) / (830. - 166.) - 0.5).abs();
                    }
                    if self.presents[i].dead {
                        dead.push(n);
                    }
                }
            }
            n += 1;
        }
        for (offset, index) in dead.into_iter().enumerate() {
            self.entities.remove(index - offset);
        }
    }
    fn gameover(&mut self) {
        self.playing = false;
        self.sounds
            .extend(["bl_game_stop", "music_ingame_gameover"]);
    }
    pub fn advance(&mut self, seconds: f64) {
        let mut remaining = seconds;
        while remaining > 1e-12 && self.playing {
            let dt = remaining.min(1. / 60.);
            self.wall_step(dt);
            self.step(dt);
            remaining -= dt;
        }
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"player":self.player,"boats":self.boats.iter().filter(|b|!b.dead).collect::<Vec<_>>(),"icebergs":self.icebergs.iter().filter(|b|!b.dead).collect::<Vec<_>>(),"presents":self.presents.iter().filter(|p|p.visible).collect::<Vec<_>>(),"level":self.level,"nextLevel":self.next_level,"score":self.score,"countdown":self.countdown,"playing":self.playing,"boatSpeed":self.boat_speed,"icebergSpeed":self.iceberg_speed,"nextBoat":self.next_boat,"nextIceberg":self.next_iceberg})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn compare(a: &serde_json::Value, b: &serde_json::Value, path: &str) {
        use serde_json::Value;
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-7,
                "{path}: {a} != {b}"
            ),
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}.{i}"));
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                for (k, b) in b {
                    compare(&a[k], b, &format!("{path}.{k}"));
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    #[test]
    fn original_simulation_scenarios() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../data/boatload-reference.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let mut g = Boatload::new("test1234", 1440., 800.);
            if case["name"] == "finish" {
                g.countdown += 300.;
            }
            for step in case["steps"].as_array().unwrap() {
                match step["action"].as_str().unwrap() {
                    "restart" => g.restart(),
                    "resize" => g.resize(900., 600.),
                    "charge100" => {
                        g.begin_charge();
                        g.wall_step(0.1);
                    }
                    "prepare" => {
                        g.player.distance = 3.;
                        g.prepare();
                    }
                    "release" => g.drop(),
                    "up" => g.set_y(g.player.min_y),
                    "down" => g.set_y(g.player.max_y),
                    "stop" => g.set_y(g.player.y),
                    _ => {}
                }
                g.advance(step["duration"].as_f64().unwrap());
                let state = serde_json::json!({"y":g.player.y,"target":g.player.target_y,"distance":g.player.distance,"prepared":g.player.prepared,"preparing":g.player.preparing,"waiting":g.player.waiting,"throwing":g.player.throwing,"has_present":g.player.present.is_some(),"score":g.score,"countdown":g.countdown,"level":g.level,"nextLevel":g.next_level,"boatSpeed":g.boat_speed,"icebergSpeed":g.iceberg_speed,"nextBoat":g.next_boat,"nextIceberg":g.next_iceberg,"playing":g.playing,"boats":g.entities.iter().filter_map(|e|if let Entity::Boat(i)=e{let b=&g.boats[*i];Some(serde_json::json!({"y":b.y,"height":b.height,"width":b.width,"speed":b.speed,"hits":b.hits,"nextBubble":b.next_bubble}))}else{None}).collect::<Vec<_>>(),"icebergs":g.entities.iter().filter_map(|e|if let Entity::Ice(i)=e{let b=&g.icebergs[*i];Some(serde_json::json!({"x":b.x,"y":b.y,"height":b.height,"width":b.width,"speed":b.speed,"presents":b.presents.len()}))}else{None}).collect::<Vec<_>>(),"presents":g.entities.iter().filter_map(|e|if let Entity::Present(i)=e{let p=&g.presents[*i];Some(serde_json::json!({"x":p.x,"y":p.y,"addX":p.add_x,"scale":p.scale,"velocity":p.velocity}))}else{None}).collect::<Vec<_>>()});
                compare(
                    &state,
                    &step["state"],
                    &format!("{} {}", case["name"], step["action"]),
                );
            }
        }
    }
}
