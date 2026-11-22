//! Snowball Storm local scene, translated from the original 2020 Google source.
//! Copyright 2020 Google LLC, Apache-2.0. Browser/network scaffolding is separate.
use crate::random::Random;
use crate::snowball_collision::{Bounds, Collision, Id};
use crate::snowball_effects::{Effects, SplatSample, TrailSample};
pub use crate::snowball_effects::Particle;
use crate::snowball_parachute::{Event as ParachuteEvent, Parachutes};
use crate::snowball_removal::{ActorState, Event as RemovalEvent, Parent, Presence, Removal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, PI};
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct V {
    pub x: f64,
    pub y: f64,
}
impl V {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
    pub fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
    pub fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s)
    }
    pub fn len(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn norm(self) -> Self {
        let l = self.len();
        if l == 0. {
            Self::default()
        } else {
            self.mul(1. / l)
        }
    }
}
#[derive(Clone, Debug)]
pub struct SeedRandom(pub u32);
impl SeedRandom {
    pub fn next(&mut self) -> f64 {
        let mut s = self.0;
        s = s.wrapping_add(0x7ed55d16).wrapping_add(s << 12);
        s = (s ^ 0xc761c23c) ^ (s >> 19);
        s = s.wrapping_add(0x165667b1).wrapping_add(s << 5);
        s = s.wrapping_add(0xd3a2646c) ^ (s << 9);
        s = s.wrapping_add(0xfd7046c5).wrapping_add(s << 3);
        s = (s ^ 0xb55a4f09) ^ (s >> 16);
        self.0 = s;
        (s & 0xfffffff) as f64 / 268435456.
    }
    pub fn range(&mut self, low: i32, high: i32) -> i32 {
        low + ((high - low) as f64 * self.next()).trunc() as i32
    }
}
#[derive(Clone, Copy)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
    pub size: f64,
}
const DIRS: [[i32; 3]; 6] = [
    [1, -1, 0],
    [1, 0, -1],
    [0, 1, -1],
    [-1, 1, 0],
    [-1, 0, 1],
    [0, -1, 1],
];
impl Default for Grid {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            size: 64.,
        }
    }
}
impl Grid {
    pub fn pixel_width(self) -> f64 {
        self.width as f64 * self.size * 0.75 + self.size * 0.25
    }
    pub fn cell_height(self) -> f64 {
        3f64.sqrt() / 2. * self.size
    }
    pub fn pixel_height(self) -> f64 {
        (self.height as f64 + 0.5) * self.cell_height()
    }
    pub fn index(self, q: i32, r: i32) -> i32 {
        if q >= 0 && q < self.width && r >= 0 && r < self.height {
            q * self.height + r
        } else {
            -1
        }
    }
    pub fn cube(self, i: i32) -> [i32; 3] {
        let q = (i as f64 / self.height as f64).floor() as i32;
        let r = i % self.height;
        let z = r - (q - (q & 1)) / 2;
        [q, -q - z, z]
    }
    fn cube_index(self, c: [i32; 3]) -> i32 {
        self.index(c[0], c[2] + (c[0] - (c[0] & 1)) / 2)
    }
    pub fn pos(self, i: i32) -> V {
        let [x, _, z] = self.cube(i);
        V::new(
            (0.5 + 0.75 * x as f64) * self.size - self.pixel_width() / 2.,
            self.pixel_height() / 2.
                - (0.5 + 3f64.sqrt() / 4. * x as f64 + 3f64.sqrt() / 2. * z as f64) * self.size,
        )
    }
    pub fn position_index(self, p: V) -> i32 {
        let x = p.x + self.pixel_width() / 2. - self.pixel_width() * 0.5 / self.width as f64;
        let y = self.pixel_height() / 2. - p.y - self.pixel_height() * 0.5 / self.height as f64;
        let q = x * 2. / 3. / (self.size / 2.);
        let r = (-x / 3. + 3f64.sqrt() / 3. * y) / (self.size / 2.);
        let c = [q, -q - r, r];
        let mut t = c.map(|x| (x + 0.5).floor() as i32);
        let d = [
            (t[0] as f64 - c[0]).abs(),
            (t[1] as f64 - c[1]).abs(),
            (t[2] as f64 - c[2]).abs(),
        ];
        if d[0] > d[1] && d[0] > d[2] {
            t[0] = -t[1] - t[2]
        } else if d[1] > d[2] {
            t[1] = -t[0] - t[2]
        } else {
            t[2] = -t[0] - t[1]
        };
        self.cube_index(t)
    }
    pub fn neighbors(self, i: i32) -> Vec<i32> {
        let c = self.cube(i);
        DIRS.iter()
            .map(|d| self.cube_index([c[0] + d[0], c[1] + d[1], c[2] + d[2]]))
            .collect()
    }
    pub fn ring(self, i: i32, radius: i32) -> Vec<i32> {
        if radius == 0 {
            return vec![i];
        }
        let c = self.cube(i);
        let mut cur = [c[0] - radius, c[1], c[2] + radius];
        let mut out = vec![];
        for d in DIRS {
            for _ in 0..radius {
                out.push(self.cube_index(cur));
                for j in 0..3 {
                    cur[j] += d[j]
                }
            }
        }
        out
    }
    pub fn path(self, from: i32, to: i32, map: &Map) -> Vec<i32> {
        if !map.passable(to) {
            return vec![];
        }
        let n = (self.width * self.height) as usize;
        let mut cost = vec![f64::INFINITY; n];
        let mut dirs = vec![-1; n];
        if from < 0 || from as usize >= n {
            return vec![];
        }
        cost[from as usize] = 0.;
        let mut heap = Heap(vec![]);
        heap.push((0., from));
        while let Some((_, current)) = heap.pop() {
            if current == to {
                let mut p = vec![to];
                let mut i = to;
                while dirs[i as usize] >= 0 && dirs[i as usize] != from {
                    i = dirs[i as usize];
                    p.insert(0, i)
                }
                p.insert(0, from);
                return p;
            }
            for neighbor in self.neighbors(current) {
                if !map.passable(neighbor) {
                    continue;
                }
                let a = V::new((to / self.height) as f64, (to % self.height) as f64);
                let b = V::new(
                    (neighbor / self.height) as f64,
                    (neighbor % self.height) as f64,
                );
                let c = cost[from as usize] + a.sub(b).len();
                if c < cost[neighbor as usize] {
                    cost[neighbor as usize] = c;
                    dirs[neighbor as usize] = current;
                    heap.push((c, neighbor))
                }
            }
        }
        vec![]
    }
    pub fn waypoints(self, from: i32, to: i32, map: &Map) -> Vec<i32> {
        let p = self.path(from, to, map);
        let mut last = [0; 3];
        let mut next = [0; 3];
        let mut out = vec![];
        for i in 0..p.len() {
            if i + 1 < p.len() {
                let a = self.cube(p[i]);
                let b = self.cube(p[i + 1]);
                next = [b[0] - a[0], b[1] - a[1], b[2] - a[2]]
            }
            if i + 1 == p.len() || last != next {
                last = next;
                out.push(p[i])
            }
        }
        out
    }
}
// Match priorityqueue.js tie handling, including right-child >= on dequeue.
struct Heap(Vec<(f64, i32)>);
impl Heap {
    fn push(&mut self, e: (f64, i32)) {
        self.0.push(e);
        let mut i = self.0.len() - 1;
        while i > 0 {
            let p = (i - 1) / 2;
            if self.0[i].0 >= self.0[p].0 {
                break;
            }
            self.0.swap(i, p);
            i = p
        }
    }
    fn pop(&mut self) -> Option<(f64, i32)> {
        if self.0.is_empty() {
            return None;
        }
        let first = self.0[0];
        let last = self.0.pop().unwrap();
        if !self.0.is_empty() {
            self.0[0] = last;
            let mut i = 0;
            loop {
                let mut best = i;
                for j in [i * 2 + 1, i * 2 + 2] {
                    if j < self.0.len() && self.0[j].0 <= self.0[best].0 {
                        best = j
                    }
                }
                if best == i {
                    break;
                }
                self.0.swap(i, best);
                i = best
            }
        }
        Some(first)
    }
}
#[derive(Clone)]
pub struct Map {
    pub grid: Grid,
    pub states: Vec<u8>,
    pub times: Vec<f64>,
    pub obstacles: Vec<i8>,
    pub rings: Vec<Vec<i32>>,
    pub random: SeedRandom,
    pub erode_random: SeedRandom,
    pub passable_count: i32,
}
impl Map {
    pub fn new(grid: Grid, seed: u32) -> Self {
        let n = (grid.width * grid.height) as usize;
        let mut m = Self {
            grid,
            states: vec![0; n],
            times: vec![0.; n],
            obstacles: vec![-1; n],
            rings: vec![],
            random: SeedRandom(seed),
            erode_random: SeedRandom(seed),
            passable_count: 0,
        };
        let erosion =
            ((grid.width * grid.width + grid.height * grid.height) as f64).sqrt() / 2. * 0.65;
        for q in 0..grid.width {
            for r in 0..grid.height {
                let mag = V::new(
                    q as f64 - grid.width as f64 / 2.,
                    r as f64 - grid.height as f64 / 2.,
                )
                .len();
                let state =
                    if mag > erosion && m.random.next() < 0.5 + (erosion - mag).abs() / erosion {
                        0
                    } else {
                        1
                    };
                let ob = if m.random.next() > 0.85 {
                    m.random.range(0, 4) as i8
                } else {
                    -1
                };
                let i = grid.index(q, r) as usize;
                m.states[i] = state;
                m.obstacles[i] = ob;
                if state > 0 {
                    let ring = mag.floor() as usize;
                    if m.rings.len() <= ring {
                        m.rings.resize(ring + 1, vec![])
                    }
                    m.rings[ring].push(i as i32);
                    m.passable_count += 1
                }
            }
        }
        let mut frontier = vec![];
        let mut visited = vec![false; n];
        'outer: for _ in 0..n / 64 {
            let mut index;
            let mut attempts = 10;
            loop {
                index = m.random.range(0, n as i32);
                attempts -= 1;
                if attempts == 0 {
                    break 'outer;
                }
                if m.state(index) == 1 {
                    break;
                }
            }
            let mut size = 0;
            frontier.push(index);
            while !frontier.is_empty() && size < 5 {
                let i = frontier.pop().unwrap();
                if m.random.next() > 0.5
                    && i >= 0
                    && !visited[i as usize]
                    && m.obstacle(i) == -1
                    && m.state(i) == 1
                {
                    m.states[i as usize] = 5;
                    visited[i as usize] = true;
                    size += 1;
                    frontier.extend(grid.neighbors(i))
                }
            }
            if frontier.len() > 1 {
                frontier.drain(..frontier.len() - 1);
            }
        }
        m
    }
    pub fn state(&self, i: i32) -> u8 {
        self.states.get(i as usize).copied().unwrap_or(0)
    }
    pub fn obstacle(&self, i: i32) -> i8 {
        self.obstacles.get(i as usize).copied().unwrap_or(-2)
    }
    pub fn passable(&self, i: i32) -> bool {
        matches!(self.state(i), 1 | 2) && self.obstacle(i) < 0
    }
    pub fn erode(&mut self, now: f64) {
        let max = ((self.rings.len() as f64 / 5.).ceil() * 3.) as i32;
        let number = self.erode_random.range(0, max);
        for _ in 0..number {
            if self.rings.is_empty() {
                break;
            }
            let j = self.rings.len() - 1;
            if self.rings[j].is_empty() {
                self.rings.pop();
                continue;
            }
            let k = self.erode_random.range(0, self.rings[j].len() as i32) as usize;
            let i = self.rings[j][k] as usize;
            match self.states[i] {
                1 | 5 => {
                    if self.states[i] == 1 {
                        self.passable_count -= 1
                    }
                    self.states[i] = 3;
                    self.times[i] = now
                }
                3 => {
                    self.rings[j].remove(k);
                    self.states[i] = 4;
                    self.times[i] = now
                }
                _ => {}
            }
            if self.rings[j].is_empty() {
                self.rings.pop();
            }
        }
    }
    pub fn habitable(&mut self) -> i32 {
        for _ in 0..self.states.len() {
            if self.rings.is_empty() {
                break;
            }
            let min = (self.rings.len() / 4) as i32;
            let max = 2.max(self.rings.len() as i32 - 10);
            let ring = self.random.range(min, max).max(0) as usize;
            if ring >= self.rings.len() || self.rings[ring].is_empty() {
                continue;
            }
            let k = self.random.range(0, self.rings[ring].len() as i32) as usize;
            let i = self.rings[ring][k];
            if self.state(i) == 1 && self.obstacle(i) == -1 {
                return i;
            }
        }
        -1
    }
    pub fn nearby(&self, index: i32, max_radius: i32, rng: &mut Random) -> i32 {
        let r = (rng.next() * (max_radius - 1) as f64).floor() as i32 + 1;
        let mut ring = self.grid.ring(index, r);
        while !ring.is_empty() {
            let k = (rng.next() * ring.len() as f64).floor() as usize;
            let tile = ring[k]; /* Source tests origin, not candidate. */
            if self.state(index) == 1 && self.obstacle(index) == -1 {
                return tile;
            }
            ring.remove(k);
        }
        index
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Powerups {
    pub primary: Option<i32>,
    pub secondary: Option<i32>,
    pub swapped: bool,
}
impl Powerups {
    pub fn active(&self) -> Option<i32> {
        if self.swapped {
            self.secondary
        } else {
            self.primary
        }
    }
    pub fn inactive(&self) -> Option<i32> {
        if self.swapped {
            self.primary
        } else {
            self.secondary
        }
    }
    pub fn set_active(&mut self, n: Option<i32>) {
        if self.swapped {
            self.secondary = n
        } else {
            self.primary = n
        }
        if self.active().is_none() && self.inactive().is_some() {
            self.swapped = !self.swapped
        }
    }
    pub fn collect(&mut self) {
        if self.active().is_none() {
            self.set_active(Some(3))
        } else if self.swapped {
            self.primary = Some(3)
        } else {
            self.secondary = Some(3)
        }
    }
    pub fn decrement(&mut self) {
        if let Some(n) = self.active() {
            self.set_active((n > 1).then_some(n - 1))
        }
    }
}
#[derive(Clone)]
pub struct Player {
    pub active: bool,
    pub parent: Parent,
    pub presence: Presence,
    pub opacity: f64,
    pub material_opacity: f64,
    pub setup: bool,
    pub landed: bool,
    pub collision_handler: bool,
    pub lod: crate::snowball_lod::ElfLod,
    pub pos: V,
    pub z: f64,
    pub tile: i32,
    pub dropped: f64,
    pub arrived: bool,
    pub bot: bool,
    pub path: Vec<V>,
    pub face: f64,
    pub alive: bool,
    pub death: f64,
    pub gone: bool,
    pub frozen: bool,
    pub color: usize,
    pub skin: usize,
    pub hair: usize,
    pub pony: bool,
    pub powerups: Powerups,
    pub last_throw: f64,
    pub last_target: Option<usize>,
    pub animator: crate::snowball_animation::Animator,
    pending_throw: bool,
    pub anim: String,
    pub fall: usize,
}
impl Player {
    fn animation(&mut self, name: &str, now: f64) {
        self.animator.play(name, now);
        if self.anim != name {
            self.anim = name.into();
        }
    }
}
#[derive(Clone)]
pub struct Shot {
    pub id: usize,
    pub hit: Option<Id>,
    pub setup: bool,
    pub origin: V,
    pub target: V,
    pub pos: V,
    pub z: f64,
    pub thrower: usize,
    pub big: bool,
    pub born: f64,
    pub gone: bool,
    pub random: f64,
    pub skew: f64,
}
impl Shot {
    pub fn advance(&mut self, precise_tick: f64) {
        let delta = self.target.sub(self.origin);
        let path = delta.norm().mul(delta.len().min(256.));
        let duration = 42. * path.len() / 256.;
        let time = (precise_tick - self.born) / duration;
        let angle = path.y.atan2(path.x).rem_euclid(2. * PI);
        let scale = ((angle % PI) - FRAC_PI_2).abs() / FRAC_PI_2;
        self.pos = self.origin.add(path.mul(time));
        self.z = 32. + scale * scale * (time * PI).sin() * 20. * path.len() / 256.;
        if precise_tick - self.born >= duration {
            self.gone = true
        }
    }
}
#[derive(Clone)]
pub struct Drop {
    pub active: bool,
    pub parent: Parent,
    pub presence: Presence,
    pub entity_z: f64,
    pub opacity: f64,
    pub tilt: f64,
    pub setup: bool,
    pub landed: bool,
    pub colliding_player: Option<usize>,
    pub tile: i32,
    pub pos: V,
    pub dropped: f64,
    pub arrived: bool,
    pub color: usize,
    pub scale: [f64; 3],
    pub rotation: f64,
    pub spin: Option<f64>,
    pub gone: bool,
    pub frozen: bool,
    pub z: f64,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Splash,
    Intro,
    Playing,
}
#[derive(Clone, Copy)]
enum TargetPosition {
    Point(V),
    // BotSystem passes the target's position by reference. Resolve only after
    // PlayerSystem has parented/positioned arrivals, like the source object.
    Player(usize),
}
pub struct Snowball {
    pub removal: Removal,
    pub parachutes: Parachutes,
    pub map: Map,
    pub players: Vec<Player>,
    pub drops: Vec<Drop>,
    pub shots: Vec<Shot>,
    pub particles: Vec<Particle>,
    pub effects: Effects,
    pub client_arrival: crate::snowball_client::Arrival,
    pub client_ring: crate::snowball_lod::ClientRing,
    pub camera: V,
    pub camera_velocity: V,
    pub time: f64,
    pub wall: f64,
    pub phase: Phase,
    pub intro_at: f64,
    pub splash_until: f64,
    pub gameover: bool,
    pub knocked_out: u32,
    pub hud_knockouts: u32,
    pub hud_time: f64,
    pub all_time: u32,
    pub sounds: Vec<&'static str>,
    random: Random,
    last_erode: f64,
    last_drop: f64,
    last_bot: f64,
    pub seed: u32,
    pending_destination: Option<V>,
    client_destination: Option<V>,
    client_target: Option<V>,
    player_destinations: Vec<(usize, V)>,
    player_targets: Vec<(usize, TargetPosition)>,
    pub destination_marker: Option<V>,
    camera_target: Option<V>,
    collision_limit: Option<(f64, f64, V)>,
    collision: Collision,
    next_shot_id: usize,
    client_dead_at: f64,
}
impl Snowball {
    pub fn new(seed: &str) -> Self {
        let random = Random::seeded(seed);
        // The source creates SnowballGame only after the intro. These unused
        // buffers/map are placeholders until start() constructs that instance.
        let effects = Effects::new(|| 0.);
        let map_seed = 0;
        Self {
            removal: Removal::default(),
            parachutes: Parachutes::default(),
            map: Map::new(Grid::default(), map_seed),
            players: vec![],
            drops: vec![],
            shots: vec![],
            particles: vec![],
            effects,
            client_arrival: crate::snowball_client::Arrival::default(),
            client_ring: crate::snowball_lod::ClientRing::default(),
            camera: V::default(),
            camera_velocity: V::default(),
            time: 0.,
            wall: 0.,
            phase: Phase::Splash,
            intro_at: 0.,
            splash_until: 0.,
            gameover: false,
            knocked_out: 0,
            hud_knockouts: 0,
            hud_time: 0.,
            all_time: 0,
            sounds: vec![],
            random,
            last_erode: 0.,
            last_drop: 0.,
            last_bot: 0.,
            seed: map_seed,
            pending_destination: None,
            client_destination: None,
            client_target: None,
            player_destinations: vec![],
            player_targets: vec![],
            destination_marker: None,
            camera_target: None,
            collision_limit: None,
            collision: Collision::new(Bounds::rect(
                V::default(),
                Grid::default().pixel_width(),
                Grid::default().pixel_height(),
            )),
            next_shot_id: 0,
            client_dead_at: 0.,
        }
    }
    pub fn jump(&mut self) {
        if self.phase == Phase::Splash {
            self.phase = Phase::Intro;
            self.intro_at = self.wall;
            self.sounds.push("generic_button_click")
        }
    }
    pub fn start(&mut self) {
        self.phase = Phase::Playing;
        self.time = 0.;
        self.splash_until = self.wall + 1000.;
        self.gameover = false;
        self.client_dead_at = 0.;
        self.client_arrival.teardown();
        self.client_ring.leave_scene();
        self.removal.teardown();
        self.pending_destination = None;
        self.client_destination = None;
        self.client_target = None;
        self.player_destinations.clear();
        self.player_targets.clear();
        self.destination_marker = None;
        self.camera_target = None;
        self.collision_limit = None;
        self.knocked_out = 0;
        self.hud_knockouts = 0;
        self.hud_time = 0.;
        self.all_time = 0;
        self.players.clear();
        self.drops.clear();
        self.shots.clear();
        // index.js restart destroys the old game and constructs new effects,
        // even though EffectSystem.teardown itself leaves its buffers intact.
        self.effects = Effects::new(|| self.random.next());
        self.particles.clear();
        self.parachutes = Parachutes::default();
        self.last_erode = 0.;
        self.last_drop = 0.;
        self.last_bot = 0.;
        self.seed = (self.random.next() * 4294967296.) as u32;
        self.map = Map::new(Grid::default(), self.seed);
        self.reset_collisions();
        self.next_shot_id = 0;
        self.add_player(false);
        for _ in 0..25 {
            self.add_drop();
            self.add_player(true)
        }
        self.camera = V::default();
        self.camera_velocity = V::default();
        self.sounds.push("music_start_ingame")
    }
    /// Rebuild registration for a fresh map or an explicitly imported diagnostic
    /// state. Ordinary frames retain registration order and quad topology.
    pub fn reset_collisions(&mut self) {
        let grid = self.map.grid;
        self.collision = Collision::new(Bounds::rect(
            V::default(),
            grid.pixel_width(),
            grid.pixel_height(),
        ));
        for (i, &obstacle) in self.map.obstacles.iter().enumerate() {
            if obstacle >= 0 {
                self.collision.add(Id::Tree(i));
            }
        }
        for (i, p) in self.players.iter().enumerate() {
            if p.setup && p.alive && !p.gone {
                self.collision.add(Id::Player(i));
            }
        }
        for (i, d) in self.drops.iter().enumerate() {
            if d.landed && !d.gone {
                self.collision.add(Id::Drop(i));
            }
        }
        for s in &self.shots {
            if s.setup {
                self.collision.add(Id::Shot(s.id));
            }
        }
    }
    fn add_player(&mut self, bot: bool) {
        let tile = self.map.habitable();
        let color = (self.random.next() * 6.) as usize;
        let skin = (self.random.next() * 4.) as usize;
        let hair = (self.random.next() * 4.) as usize;
        let pony = self.random.next() >= 0.5;
        self.players.push(Player {
            active: true,
            parent: Parent::Detached,
            presence: Presence::default(),
            opacity: 1.,
            material_opacity: 1.,
            setup: false,
            landed: false,
            collision_handler: false,
            lod: crate::snowball_lod::ElfLod::default(),
            pos: V::default(),
            z: 0.,
            tile,
            dropped: f64::INFINITY,
            arrived: false,
            bot,
            path: vec![],
            face: 0.,
            alive: true,
            death: 0.,
            gone: false,
            frozen: false,
            color,
            skin,
            hair,
            pony,
            powerups: Default::default(),
            last_throw: (self.time * 0.06).floor(),
            last_target: None,
            animator: crate::snowball_animation::Animator::new(),
            pending_throw: false,
            anim: "elf_rig_idle".into(),
            fall: 1,
        });
        self.all_time += 1
    }
    fn add_drop(&mut self) {
        let tile = self.map.habitable();
        let color = (self.random.next() * 7.) as usize;
        let scale = [
            self.random.next() * 7. + 12.,
            self.random.next() * 7. + 12.,
            self.random.next() * 5. + 10.,
        ];
        self.drops.push(Drop {
            active: true,
            parent: Parent::Detached,
            presence: Presence::default(),
            entity_z: 0.,
            opacity: 1.,
            tilt: 0.,
            setup: false,
            landed: false,
            colliding_player: None,
            tile,
            pos: V::default(),
            dropped: f64::INFINITY,
            arrived: false,
            color,
            scale,
            rotation: 0.,
            spin: None,
            gone: false,
            frozen: false,
            z: 0.,
        })
    }
    /// Source PlayerSystem maps retain first insertion order and last values.
    pub fn assign_player_destination(&mut self, index: usize, pos: V) {
        if self.players.get(index).is_some() {
            if let Some((_, old)) = self
                .player_destinations
                .iter_mut()
                .find(|(i, _)| *i == index)
            {
                *old = pos;
            } else {
                self.player_destinations.push((index, pos));
            }
        }
    }
    pub fn assign_player_target(&mut self, index: usize, pos: V) {
        self.queue_player_target(index, TargetPosition::Point(pos));
    }
    fn queue_player_target(&mut self, index: usize, target: TargetPosition) {
        // The source also sets hasAssignedTarget for a dead player; Elf.update
        // retains that flag until a later living update, even when no shot fires.
        self.players[index].pending_throw = true;
        if let Some((_, old)) = self.player_targets.iter_mut().find(|(i, _)| *i == index) {
            *old = target;
        } else {
            self.player_targets.push((index, target));
        }
    }
    fn follow_destination(&mut self, index: usize, pos: V) {
        if !self.players[index].alive || !self.players[index].arrived {
            return;
        }
        let grid = self.map.grid;
        let path = grid.waypoints(
            grid.position_index(self.players[index].pos),
            grid.position_index(pos),
            &self.map,
        );
        if !path.is_empty() {
            let mut points: Vec<_> = path
                .iter()
                .skip(1)
                .take(path.len().saturating_sub(2))
                .map(|&i| grid.pos(i))
                .collect();
            points.push(pos);
            self.players[index].path = points
        } else if index == 0 {
            self.client_target = Some(pos)
        }
    }
    pub fn throw(&mut self, index: usize, target: V) {
        if !self.players[index].alive {
            return;
        }
        let big = self.players[index].powerups.active().is_some();
        self.players[index].powerups.decrement();
        let origin = self.players[index].pos;
        self.shots.push(Shot {
            id: self.next_shot_id,
            hit: None,
            setup: false,
            origin,
            target,
            pos: origin,
            z: 0.,
            thrower: index,
            big,
            born: -1.,
            gone: false,
            random: self.random.next(),
            skew: self.random.next() * PI / 6. - PI / 12.,
        });
        self.next_shot_id += 1;
        self.players[index].face = (target.y - origin.y).atan2(target.x - origin.x) + FRAC_PI_2;
        self.players[index].pending_throw = true;
        if index == 0 {
            self.sounds.push("snowball_throw")
        }
    }
    fn kill(&mut self, index: usize, frozen: bool) {
        let p = &mut self.players[index];
        if p.alive {
            p.death = self.time;
        }
        p.alive = false;
        p.frozen |= frozen;
        p.fall = (self.random.next() * 3.).floor() as usize + 1;
        p.animator.fall(p.fall, self.time);
        p.anim = format!("elf_rig_fall_down_{}", p.fall);
        if frozen {
            self.removal
                .freeze(Id::Player(index), self.time, Some(&mut p.presence));
        } else {
            self.removal
                .teleport(Id::Player(index), self.time, Some(&mut p.presence));
        }
    }
    fn hit_player(&mut self, index: usize) {
        self.kill(index, false);
    }
    fn update_removal(&mut self, tick: f64, width: f64, height: f64) {
        let events = self.removal.update(
            self.map.grid,
            Some(self.map.rings.len()),
            tick,
            self.time,
            width,
            height,
            |id| match id {
                Id::Player(i) => {
                    let p = &self.players[i];
                    ActorState {
                        pos: p.pos,
                        exit_time: p.presence.exit_time,
                        visibility: true,
                    }
                }
                Id::Drop(i) => {
                    let d = &self.drops[i];
                    ActorState {
                        pos: d.pos,
                        exit_time: d.presence.exit_time,
                        visibility: false,
                    }
                }
                _ => unreachable!(),
            },
        );
        for event in events {
            match event {
                RemovalEvent::Attached(Id::Player(i), slot) => {
                    let p = &mut self.players[i];
                    p.pos = V::default();
                    p.z = -16.;
                    p.parent = Parent::Ice(slot);
                }
                RemovalEvent::Attached(Id::Drop(i), slot) => {
                    let d = &mut self.drops[i];
                    d.pos = V::default();
                    d.entity_z = -16.;
                    d.parent = Parent::Ice(slot);
                }
                RemovalEvent::Gone(Id::Player(i)) => {
                    let p = &mut self.players[i];
                    p.parent = Parent::Detached;
                    p.presence.finish();
                    p.gone = true;
                }
                RemovalEvent::Gone(Id::Drop(i)) => {
                    let d = &mut self.drops[i];
                    d.parent = Parent::Detached;
                    d.presence.finish();
                    d.gone = true;
                }
                RemovalEvent::Teleport {
                    id: Id::Player(i),
                    z,
                    opacity,
                    finished,
                } => {
                    let p = &mut self.players[i];
                    p.z = z;
                    if let Some(o) = opacity {
                        p.opacity = o;
                    }
                    if finished {
                        p.presence.finish();
                        p.gone = true;
                    }
                }
                RemovalEvent::Teleport {
                    id: Id::Drop(i),
                    z,
                    finished,
                    ..
                } => {
                    let d = &mut self.drops[i];
                    d.entity_z = z;
                    if finished {
                        d.presence.finish();
                        d.gone = true;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    fn update_parachutes(&mut self, tick: f64) {
        let grid = self.map.grid;
        let events = self
            .parachutes
            .update(tick, 64., 300., 420., |id| match id {
                Id::Player(i) => (grid.pos(self.players[i].tile), self.players[i].dropped),
                Id::Drop(i) => (grid.pos(self.drops[i].tile), self.drops[i].dropped),
                _ => unreachable!("Arrival component"),
            });
        for event in events {
            match event {
                ParachuteEvent::Carried(Id::Player(i), tick) => {
                    self.players[i].pos = V::default();
                    self.players[i].z = 0.;
                    self.players[i].dropped = tick;
                    self.players[i].parent = Parent::Canopy;
                }
                ParachuteEvent::Carried(Id::Drop(i), tick) => {
                    self.drops[i].pos = V::default();
                    self.drops[i].dropped = tick;
                    self.drops[i].entity_z = 0.;
                    self.drops[i].parent = Parent::Canopy;
                }
                ParachuteEvent::Arrived(Id::Player(i)) => {
                    self.players[i].arrived = true;
                    if self.players[i].parent == Parent::Canopy {
                        self.players[i].parent = Parent::Detached;
                    }
                }
                ParachuteEvent::Arrived(Id::Drop(i)) => {
                    self.drops[i].arrived = true;
                    if self.drops[i].parent == Parent::Canopy {
                        self.drops[i].parent = Parent::Detached;
                    }
                }
                ParachuteEvent::Detached(Id::Player(i)) => {
                    if self.players[i].parent == Parent::Canopy {
                        self.players[i].parent = Parent::Detached;
                    }
                }
                ParachuteEvent::Detached(Id::Drop(i)) => {
                    if self.drops[i].parent == Parent::Canopy {
                        self.drops[i].parent = Parent::Detached;
                    }
                }
                _ => unreachable!("Arrival component"),
            }
        }
    }
    pub fn advance(&mut self, seconds: f64, width: f64, height: f64) {
        for _ in 0..(seconds * 60.).round() as usize {
            self.step(1. / 60., width, height)
        }
    }
    /// Source ClientSystem.assignTarget clones the position at pick time.
    pub fn target_position(&mut self, pos: V) {
        self.client_target = Some(pos);
    }
    pub fn pick_destination(&mut self, pos: V) {
        self.pending_destination = Some(pos);
    }
    pub fn step(&mut self, dt: f64, width: f64, height: f64) {
        if self.begin_frame(dt, width, height) {
            self.finish_frame(width, height)
        }
    }
    /// Original Level update / camera and erosion precede rendering.
    pub fn begin_frame(&mut self, dt: f64, width: f64, height: f64) -> bool {
        self.wall += dt * 1000.;
        if self.phase == Phase::Intro && self.wall - self.intro_at >= 5000. {
            self.start()
        }
        if self.phase != Phase::Playing {
            return false;
        }
        self.time += dt * 1000.;
        let tick = (self.time * 0.06).floor();
        // LocalLevel precedes all systems. Strict > gates intentionally skip ticks.
        if tick - self.last_drop > 625. {
            self.last_drop = tick;
            self.add_drop()
        }
        if tick - self.last_erode > 16. {
            self.last_erode = tick;
            self.map.erode(self.time)
        }
        if self.all_time < 100 && tick - self.last_bot > 128. {
            self.add_player(true);
            self.last_bot = tick
        }
        if self.knocked_out >= 99 {
            self.gameover = true
        }
        if let Some(destination) = self.pending_destination.take() {
            if let Some(p) = self.players.first() {
                if p.alive {
                    // MainLevel forwards to Client, not directly to PlayerSystem.
                    self.client_destination = Some(destination);
                } else {
                    self.camera_target = Some(destination);
                }
            }
        }
        if let Some(p) = self.players.first() {
            // MainLevel always reassigns the living client as tracker target,
            // including while unarrived/exiting; an old dead-player pick expires.
            if p.alive {
                self.camera_target = None;
            }
            let target = if p.alive && p.arrived && !p.presence.exiting() && !p.presence.gone() {
                Some(p.pos)
            } else if !p.alive {
                self.camera_target
            } else {
                None
            };
            if let Some(target) = target {
                crate::snowball_client::track(
                    &mut self.camera,
                    &mut self.camera_velocity,
                    target,
                    width,
                    height,
                );
            }
        }
        true
    }
    /// Original render/input precede all simulation systems. Keep one update
    /// per presented frame; tick is wall time, not a fixed-step catch-up loop.
    pub fn finish_frame(&mut self, width: f64, height: f64) {
        let grid = self.map.grid;
        let tick = (self.time * 0.06).floor();
        // MainLevel.measure creates the shared limit at the origin; LodSystem
        // moves it AFTER Collision2DSystem, so collision uses last frame's camera.
        let limit_center = self
            .collision_limit
            .filter(|&(w, h, _)| w == width && h == height)
            .map(|(_, _, p)| p)
            .unwrap_or_default();
        let collision_limit = Bounds::rect(limit_center, width + 256., height * 4. / 3. + 256.);
        let mut bounds = HashMap::new();
        for &id in &self.collision.order {
            let collider = match id {
                Id::Tree(i) => Bounds::circle(grid.pos(i as i32).add(V::new(0., -17.)), 14.),
                Id::Player(i) => Bounds::rect(self.players[i].pos, 15., 45.),
                Id::Drop(i) => Bounds::circle(self.drops[i].pos, 10.),
                Id::Shot(serial) => {
                    let s = self
                        .shots
                        .iter()
                        .find(|s| s.id == serial)
                        .expect("registered shot");
                    Bounds::circle(s.pos, if s.big { 20. } else { 6. })
                }
            };
            bounds.insert(id, collider);
        }
        let notifications = self.collision.update(collision_limit, &bounds);
        let shot_indices: HashMap<_, _> = self
            .shots
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id, i))
            .collect();
        for (id, other) in notifications {
            match (id, other) {
                (Id::Shot(serial), other) => {
                    let s = &mut self.shots[shot_indices[&serial]];
                    if other != Id::Player(s.thrower) {
                        s.hit = Some(other);
                        s.gone = true;
                        self.sounds.push("snowball_hit");
                    }
                }
                (Id::Player(i), Id::Shot(serial)) => {
                    let s = &self.shots[shot_indices[&serial]];
                    if self.players[i].collision_handler && s.thrower != i {
                        let direction = s.target.sub(s.origin).norm();
                        self.players[i].face = direction.y.atan2(direction.x) - FRAC_PI_2;
                        self.hit_player(i);
                    }
                }
                (Id::Drop(i), Id::Player(j)) => {
                    let d = &mut self.drops[i];
                    if d.colliding_player.is_none() && d.spin.is_none() && !d.frozen {
                        d.colliding_player = Some(j);
                    }
                }
                _ => {}
            }
        }
        self.collision_limit = Some((
            width,
            height,
            V::new(self.camera.x, -self.camera.y * 4. / 3.),
        ));
        // LodSystem follows collision and uses the current camera. Player
        // colliders remain in local actor coordinates, like the source objects.
        let lod_limit = Bounds::rect(
            V::new(self.camera.x, -self.camera.y * 4. / 3.),
            width + 256.,
            height * 4. / 3. + 256.,
        );
        for (i, p) in self.players.iter_mut().enumerate() {
            if p.setup && (p.active || i == 0) {
                p.lod
                    .assign(p.alive, lod_limit.contains(Bounds::rect(p.pos, 15., 45.)));
            }
        }
        // A canopy registers after its carried elf. Its Point collider's LOD
        // forwards to that elf even after another parent has taken the actor.
        for c in &self.parachutes.canopies {
            if let Id::Player(i) = c.entity {
                let p = &mut self.players[i];
                p.lod
                    .assign(p.alive, lod_limit.contains(Bounds::rect(c.pos, 0., 0.)));
            }
        }
        // Source newSnowballs.pop() sets up new shots after collision and
        // appends them in reverse throw order, before the active-array loop.
        if let Some(first_pending) = self.shots.iter().position(|s| !s.setup) {
            self.shots[first_pending..].reverse();
            for s in &mut self.shots[first_pending..] {
                s.setup = true;
                self.collision.add(Id::Shot(s.id));
                self.effects.add(s.id);
            }
        }
        let mut removed = vec![];
        let mut i = 0;
        while i < self.shots.len() {
            if self.shots[i].gone {
                let s = self.shots.remove(i);
                self.collision.remove(Id::Shot(s.id));
                self.effects.remove(s.id);
                self.effects.show_for(s.id);
                removed.push(s);
                // The source for-loop increments even after splice + continue.
                i += 1;
                continue;
            }
            let s = &mut self.shots[i];
            if s.born < 0. {
                s.born = tick
            } else {
                s.advance(self.time * 0.06)
            }
            if s.gone {
                let s = self.shots.remove(i);
                self.collision.remove(Id::Shot(s.id));
                self.effects.remove(s.id);
                self.effects.show_for(s.id);
                removed.push(s); /* source splice skips next */
                i += 1
            } else {
                i += 1
            }
        }
        self.update_removal(tick, width, height);
        let mut collected_by_client = false;
        for (di, d) in self.drops.iter_mut().enumerate() {
            if !d.active {
                continue;
            }
            if !d.setup {
                d.setup = true;
                d.z = 32.;
                d.tilt = PI / 2.5;
                d.rotation = 0.;
                d.opacity = 1.;
                self.parachutes.drop_entity(Id::Drop(di));
            }
            if d.arrived && !d.landed {
                d.landed = true;
                d.pos = grid.pos(d.tile);
                d.parent = Parent::Map;
                self.collision.add(Id::Drop(di));
            }
            if d.arrived {
                if !d.presence.exiting() {
                    d.rotation += 0.01;
                } else if let Some(start) = d.spin {
                    let t = ((self.time - start) / 300.).min(1.);
                    let delta = -((t - 1.).powi(4) + (t - 1.).powi(3));
                    d.rotation += delta * 2. * PI;
                    d.z += delta * 64.;
                    for scale in &mut d.scale {
                        *scale *= 1f64.max(1. + delta);
                    }
                    d.opacity = (1. - t).max(0.);
                    if t == 1. {
                        d.presence.finish();
                    }
                } else {
                    d.rotation = 0.;
                    d.tilt = PI / 2.;
                }
                d.gone = d.presence.gone();
                if d.gone {
                    d.active = false;
                    if d.parent == Parent::Map {
                        d.parent = Parent::Detached;
                    }
                    self.collision.remove(Id::Drop(di));
                } else if !d.presence.exiting() {
                    if self.map.state(grid.position_index(d.pos)) == 4 {
                        d.frozen = true;
                        self.collision.remove(Id::Drop(di));
                        self.removal
                            .freeze(Id::Drop(di), self.time, Some(&mut d.presence));
                    } else if let Some(i) = d.colliding_player.take() {
                        self.players[i].powerups.collect();
                        d.spin = Some(self.time);
                        d.presence.exit(self.time);
                        d.presence.present = true;
                        collected_by_client |= i == 0;
                    }
                } else {
                    self.collision.remove(Id::Drop(di));
                }
            }
        }
        self.update_parachutes(tick);
        // Source EffectSystem runs after parachutes, before Map/Client/Bot/Player.
        // Removed shots are not reallocated before this flush, so their Splat
        // component values remain stable here. Full pooled object identity is
        // still handled separately from these persistent attribute buffers.
        let splats: HashMap<_, _> = removed.iter().map(|s| (s.id, SplatSample {
            position: [s.pos.x, s.pos.y, s.z],
            direction: { let d = s.target.sub(s.origin).norm(); [d.x, d.y] },
            size: if s.big { 75. } else { 40. },
            quantity: if s.big { 7 } else { 5 },
        })).collect();
        let shots: HashMap<_, _> = self.shots.iter().map(|s| (s.id, s)).collect();
        self.effects.update(self.time, |id| shots.get(&id).map(|s| TrailSample {
            position: [s.pos.x, s.pos.y, s.z],
            size: if s.big { 10. } else { 6. },
            color: 0xaaccff,
            show: true,
        }), |id| splats.get(&id).cloned());
        self.effects.write_particles(self.time, &mut self.particles);

        // MapSystem precedes Client/Player commands: retain this rendered marker
        // until the following map update, even if movement reaches its endpoint.
        if let Some(p) = self.players.first() {
            self.destination_marker = p.path.last().map(|p| V::new(p.x, p.y - 20.));
        }
        self.client_arrival.update(
            self.players
                .first()
                .map(|p| (p.alive, p.arrived, p.tile, -10., PI / 2.25)),
            self.players
                .first()
                .map(|p| grid.pos(p.tile))
                .unwrap_or_default(),
            &mut self.camera,
        );
        if let Some(p) = self.players.first().filter(|p| !p.alive) {
            if self.client_dead_at == 0. {
                self.client_dead_at = self.time;
            }
            if p.presence.gone() || self.time - self.client_dead_at > 2500. {
                self.gameover = true;
            }
        }
        if self.players.first().is_some_and(|p| p.alive) {
            self.hud_knockouts = self.knocked_out;
            self.hud_time = self.time / 1000.;
            if let Some(destination) = self.client_destination.take() {
                self.assign_player_destination(0, destination);
            }
            if let Some(target) = self.client_target.take() {
                self.assign_player_target(0, target);
            }
        }
        // Bots consume randomness per rendered update, not a catch-up fixed loop.
        let mut moves = vec![];
        let mut throws = vec![];
        for i in 1..self.players.len() {
            let p = &self.players[i];
            if !p.bot || !p.arrived || p.gone {
                continue;
            }
            if p.path.is_empty() && self.random.next() > 0.985 {
                let tile = self
                    .map
                    .nearby(grid.position_index(p.pos), 6, &mut self.random);
                moves.push((i, grid.pos(tile)))
            }
            let nearby = self.collision.quad.near(Bounds::rect(p.pos, 200., 200.));
            let mut target = p.last_target.filter(|&j| nearby.contains(&Id::Player(j)));
            if target.is_none() {
                for id in nearby {
                    let Id::Player(j) = id else { continue };
                    let other = &self.players[j];
                    if j == i || other.pos.len() == 0. {
                        continue;
                    }
                    if self.random.next() > 0.25 && p.pos.sub(other.pos).len() < 320. {
                        target = Some(j);
                        break;
                    }
                }
            }
            if let Some(j) = target {
                if tick - p.last_throw > 40. {
                    if self.random.next() > 0.5 {
                        throws.push((i, j))
                    }
                    self.players[i].last_throw = tick
                }
            }
            self.players[i].last_target = target;
        }
        if collected_by_client {
            self.sounds.push("snowball_pickup");
        }
        for (i, p) in self.players.iter_mut().enumerate() {
            if !p.setup {
                p.setup = true;
                self.collision.add(Id::Player(i));
                if i == 0 {
                    self.client_ring.setup(i);
                }
                if !p.arrived {
                    self.parachutes.drop_entity(Id::Player(i));
                }
            }
        }
        for p in &mut self.players {
            if p.arrived && !p.landed {
                p.landed = true;
                p.parent = Parent::Map;
                p.pos = grid.pos(p.tile);
            }
        }
        for (i, p) in moves {
            self.assign_player_destination(i, p)
        }
        for (i, j) in throws {
            self.queue_player_target(i, TargetPosition::Player(j))
        }
        // All destinations precede all targeted positions. A failed local path
        // feeds Client for its NEXT update; it cannot fire in this Player pass.
        for (i, destination) in std::mem::take(&mut self.player_destinations) {
            self.follow_destination(i, destination);
        }
        for (i, target) in std::mem::take(&mut self.player_targets) {
            if !self.players[i].alive {
                continue;
            }
            let position = match target {
                TargetPosition::Point(p) => p,
                TargetPosition::Player(j) => self.players[j].pos,
            };
            self.throw(i, position);
        }
        let mut freezing = vec![];
        for (i, p) in self.players.iter_mut().enumerate() {
            if !p.active {
                continue;
            }
            self.client_ring
                .update(i, i == 0, p.arrived, p.alive, p.presence.exiting());
            if p.arrived {
                p.collision_handler = true;
            }
            p.lod.update(p.alive);
            if p.alive {
                if let Some(&next) = p.path.first() {
                    let d = next.sub(p.pos);
                    let step = 2.25 * if p.bot { 0.75 } else { 1. };
                    let v = d.norm().mul(step);
                    if d.len() <= step {
                        p.pos = next;
                        p.path.remove(0);
                    } else {
                        p.pos = p.pos.add(v)
                    }
                    p.face = v.y.atan2(v.x) + FRAC_PI_2;
                    p.animation("elf_rig_running", self.time)
                } else {
                    p.animation("elf_rig_idle", self.time)
                }
            } else {
                self.collision.remove(Id::Player(i));
                if p.presence.exiting() {
                    p.material_opacity = p.opacity;
                }
            }
            if p.alive && p.pending_throw {
                p.animator.mix_once("elf_rig_throw");
                p.pending_throw = false;
            }
            p.animator.update(self.time);
            if let Some(name) = p.animator.playing_name() {
                p.anim = name.into();
            }
            if self.map.state(grid.position_index(p.pos)) == 4
                && p.presence.present
                && !p.presence.exiting()
            {
                freezing.push(i);
            } else if p.presence.gone() {
                p.gone = true;
                p.active = false;
                self.knocked_out += 1;
                self.sounds.push("snowball_frozen");
                if i != 0 && p.parent == Parent::Map {
                    p.parent = Parent::Detached;
                }
            }
        }
        for i in freezing {
            self.kill(i, true)
        }
    }
}
pub fn circle_rect(c: V, r: f64, p: V, hw: f64, hh: f64) -> bool {
    let d = c.sub(p);
    let x = d.x.abs();
    let y = d.y.abs();
    x <= hw + r
        && y <= hh + r
        && (x <= hw || y <= hh || (x - hw).powi(2) + (y - hh).powi(2) <= r * r)
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    fn production_effects_follow_shot_stages_and_recreate_on_restart() {
        let mut game = Snowball::new("effect-system-stages");
        game.start();
        game.players.clear();
        game.drops.clear();
        game.collision = Collision::new(Bounds::rect(V::default(), 10000., 10000.));
        game.time = 1000.;
        game.wall = 6000.;
        game.shots.push(Shot {
            id: 100, hit: None, setup: false, origin: V::default(),
            target: V::new(100., 0.), pos: V::default(), z: 0.,
            thrower: usize::MAX, big: false, born: -1., gone: false,
            random: 0.25, skew: 0.,
        });
        let randoms: Vec<_> = game.effects.splats.iter().map(|p|p.random).collect();
        game.finish_frame(1440.,800.);
        assert_eq!(game.effects.registered, vec![100]);
        assert_eq!(game.effects.next_trail,1);
        assert_eq!(game.effects.trails[0].born,1000.);
        assert_eq!(game.effects.trails[0].z,1.);
        game.shots[0].gone = true;
        game.time += 16.;
        assert_eq!(game.effects.time,1000.); // render still uses preceding effect update
        game.finish_frame(1440.,800.);
        assert!(game.effects.registered.is_empty());
        assert_eq!(game.effects.next_trail,1);
        assert_eq!(game.effects.next_splat,5);
        assert_eq!(game.effects.splats[0].born,1016.);
        assert_eq!(game.effects.splats.iter().map(|p|p.random).collect::<Vec<_>>(),randoms);
        game.start();
        assert_eq!(game.effects.next_trail,0);
        assert_eq!(game.effects.next_splat,0);
        assert_eq!(game.effects.time,0.);
        assert!(game.effects.registered.is_empty());
        assert!(game.particles.is_empty());
        assert!(game.effects.trails.iter().chain(&game.effects.splats).all(|p|p.size==0.));
        assert_ne!(game.effects.splats[0].random,randoms[0]);
        assert_eq!(game.time,0.);
    }

    #[test]
    fn concrete_removal_stages_match_original_systems_and_parents() {
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../data/snowball-removal-stages-reference.json"
        ))
        .unwrap();
        let label = |id: Id| match id {
            Id::Player(i) => format!("p{i}"),
            Id::Drop(i) => format!("d{i}"),
            _ => unreachable!(),
        };
        let parent = |p: Parent| match p {
            Parent::Detached => None,
            Parent::Map => Some("map".into()),
            Parent::Canopy => Some("canopy".into()),
            Parent::Ice(i) => Some(format!("ice{i}")),
        };
        for case in data["cases"].as_array().unwrap() {
            let mut game = Snowball::new("removal-stages");
            game.map.obstacles.fill(-1);
            game.map.states.fill(1);
            game.map.rings.resize(4, vec![]);
            let air = case["air"].as_bool().unwrap();
            for tile in case["playerTiles"].as_array().unwrap() {
                game.add_player(false);
                let grid = game.map.grid;
                let p = game.players.last_mut().unwrap();
                p.tile = tile.as_i64().unwrap() as i32;
                if !air {
                    p.pos = grid.pos(p.tile);
                    p.arrived = true;
                    p.landed = true;
                    p.setup = true;
                    p.collision_handler = true;
                    p.parent = Parent::Map;
                    p.animation("elf_rig_idle", 0.);
                    p.animator.update(0.);
                }
            }
            game.add_drop();
            let d = &mut game.drops[0];
            d.tile = case["dropTile"].as_i64().unwrap() as i32;
            d.scale = [18., 16., 13.];
            if !air {
                d.pos = game.map.grid.pos(d.tile);
                d.setup = true;
                d.arrived = true;
                d.landed = true;
                d.parent = Parent::Map;
                d.z = 32.;
                d.tilt = PI / 2.5;
            }
            game.reset_collisions();
            if air {
                game.finish_frame(1440., 800.);
            }
            let snapshot = |g: &Snowball| {
                serde_json::json!({
                    "players":g.players.iter().enumerate().map(|(i,p)|serde_json::json!({"id":format!("p{i}"),"active":p.active,"pos":[p.pos.x,p.pos.y,p.z],"arrived":p.arrived,"alive":p.alive,"death":if p.alive{-1.}else{p.death},"present":p.presence.present,"exit":p.presence.exit_time,"gone":p.presence.gone(),"opacity":p.opacity,"material":p.material_opacity,"parent":parent(p.parent),"playing":p.animator.playing_name()})).collect::<Vec<_>>(),
                    "drops":g.drops.iter().enumerate().map(|(i,d)|serde_json::json!({"id":format!("d{i}"),"active":d.active,"pos":[d.pos.x,d.pos.y,d.entity_z],"arrived":d.arrived,"present":d.presence.present,"exit":d.presence.exit_time,"gone":d.presence.gone(),"parent":parent(d.parent),"rotation":d.rotation,"tilt":d.tilt,"z":d.z,"opacity":d.opacity})).collect::<Vec<_>>(),
                    "order":g.collision.order.iter().copied().map(label).collect::<Vec<_>>(),"freezing":g.removal.freezing.iter().copied().map(label).collect::<Vec<_>>(),"frozen":g.removal.frozen.iter().copied().map(label).collect::<Vec<_>>(),"teleporting":g.removal.teleporting.iter().copied().map(label).collect::<Vec<_>>(),"layer":g.removal.layer,
                    "bodies":g.removal.bodies.iter().map(|b|serde_json::json!({"pos":[b.pos.x,b.pos.y,b.z],"rotation":b.rotation,"opacity":b.opacity,"start":b.start_tick})).collect::<Vec<_>>(),
                    "canopies":g.parachutes.canopies.iter().map(|c|serde_json::json!({"id":label(c.entity),"pos":[c.pos.x,c.pos.y,c.z],"rotation":c.rotation})).collect::<Vec<_>>(),"undropped":g.parachutes.undropped.iter().copied().map(label).collect::<Vec<_>>(),"dropping":g.parachutes.dropping.iter().copied().map(label).collect::<Vec<_>>(),"knockedOut":g.knocked_out,"gameover":g.gameover,"sounds":g.sounds
                })
            };
            assert_json_close(
                &snapshot(&game),
                &case["initial"],
                case["name"].as_str().unwrap(),
            );
            let (mut w, mut h) = (1440., 800.);
            for frame in case["frames"].as_array().unwrap() {
                let input = &frame["input"];
                game.time = input["at"].as_f64().unwrap();
                game.sounds.clear();
                if input["size"].is_array() {
                    w = input["size"][0].as_f64().unwrap();
                    h = input["size"][1].as_f64().unwrap();
                }
                if let Some(n) = input["rings"].as_u64() {
                    game.map.rings.resize(n as usize, vec![]);
                }
                if let Some(tiles) = input["erode"].as_array() {
                    for t in tiles {
                        game.map.states[t.as_u64().unwrap() as usize] = 4;
                    }
                }
                if input["erodeOrigin"].as_bool().unwrap_or(false) {
                    let i = game.map.grid.position_index(V::default());
                    game.map.states[i as usize] = 4;
                }
                game.random = (0..10000)
                    .find_map(|seed| {
                        let r = Random::seeded(&format!("removal-fall-{seed}"));
                        let mut q = r.clone();
                        (0..4).all(|_| (q.next() * 3.).floor() == 1.).then_some(r)
                    })
                    .unwrap();
                if let Some(die) = input["die"].as_array() {
                    for e in die {
                        let i = e.as_str().unwrap()[1..].parse().unwrap();
                        game.kill(i, false);
                    }
                }
                game.finish_frame(w, h);
                assert_json_close(
                    &snapshot(&game),
                    &frame["after"],
                    &format!("{} at {}", case["name"], input["at"]),
                );
            }
        }
    }
    #[test]
    fn arrival_stages_match_actual_player_drop_and_parachute_systems() {
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../data/snowball-arrival-stages-reference.json"
        ))
        .unwrap();
        let label = |id: Id| match id {
            Id::Player(i) => format!("p{i}"),
            Id::Drop(i) => format!("d{i}"),
            _ => unreachable!(),
        };
        for case in data["cases"].as_array().unwrap() {
            let mut game = Snowball::new("arrival-stages");
            game.map.obstacles.fill(-1);
            game.map.states.fill(1);
            game.reset_collisions();
            let snapshot = |g: &Snowball| {
                serde_json::json!({
                    "players":g.players.iter().enumerate().map(|(i,p)|serde_json::json!({
                        "id":format!("p{i}"),"pos":[p.pos.x,p.pos.y,p.z],"tile":p.tile,"arrived":p.arrived,
                        "dropped":p.dropped.is_finite().then_some(p.dropped),"setup":p.setup,"handler":p.collision_handler,
                        "parent":if p.landed{Some("map")}else{g.parachutes.get(Id::Player(i)).map(|_|"parachute")}
                    })).collect::<Vec<_>>(),
                    "drops":g.drops.iter().enumerate().map(|(i,d)|serde_json::json!({
                        "id":format!("d{i}"),"pos":[d.pos.x,d.pos.y,0.],"tile":d.tile,"arrived":d.arrived,
                        "dropped":d.dropped.is_finite().then_some(d.dropped),"setup":d.setup,
                        "parent":if d.landed{Some("map")}else{g.parachutes.get(Id::Drop(i)).map(|_|"parachute")},
                        "rotation":d.rotation,"z":d.z,"scale":d.scale
                    })).collect::<Vec<_>>(),
                    "undropped":g.parachutes.undropped.iter().copied().map(label).collect::<Vec<_>>(),
                    "dropping":g.parachutes.dropping.iter().copied().map(label).collect::<Vec<_>>(),
                    "order":g.collision.order.iter().copied().map(label).collect::<Vec<_>>(),
                    "shots":g.shots.iter().map(|s|serde_json::json!({"origin":[s.origin.x,s.origin.y],"target":[s.target.x,s.target.y],"setup":s.setup})).collect::<Vec<_>>(),
                    "canopies":g.parachutes.canopies.iter().map(|c|serde_json::json!({"id":label(c.entity),"pos":[c.pos.x,c.pos.y,c.z],"rotation":c.rotation})).collect::<Vec<_>>()
                })
            };
            for frame in case["frames"].as_array().unwrap() {
                game.time = frame["at"].as_f64().unwrap();
                for added in frame["added"].as_array().unwrap() {
                    let id = added["id"].as_str().unwrap();
                    let tile = added["tile"].as_i64().unwrap() as i32;
                    if id.starts_with('p') {
                        game.add_player(false);
                        game.players.last_mut().unwrap().tile = tile;
                    } else {
                        game.add_drop();
                        let d = game.drops.last_mut().unwrap();
                        d.tile = tile;
                        d.scale = std::array::from_fn(|i| added["scale"][i].as_f64().unwrap());
                        d.rotation = added["rotation"].as_f64().unwrap();
                        d.z = added["z"].as_f64().unwrap();
                    }
                }
                let path = format!("{} at {}", case["name"], frame["at"]);
                assert_json_close(
                    &snapshot(&game),
                    &frame["before"],
                    &format!("{path} before"),
                );
                if frame["botLanding"].as_bool().unwrap_or(false) {
                    game.players[1].bot = true;
                    game.players[1].path = vec![V::new(800., 0.)];
                    game.players[1].last_target = Some(0);
                    game.players[1].last_throw = (game.time * 0.06).floor() - 41.;
                    game.random = (0..100)
                        .find_map(|i| {
                            let rng = Random::seeded(&format!("landing-throw-{i}"));
                            (rng.clone().next() > 0.5).then_some(rng)
                        })
                        .unwrap();
                }
                game.finish_frame(1440., 800.);
                assert_json_close(&snapshot(&game), &frame["after"], &format!("{path} after"));
            }
        }
    }
    #[test]
    fn concrete_contact_actions_match_original_actor_and_system_handlers() {
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../data/snowball-contact-actions-reference.json"
        ))
        .unwrap();
        let label = |id: Id| match id {
            Id::Player(i) => format!("p{i}"),
            Id::Drop(i) => format!("d{i}"),
            Id::Shot(i) => format!("s{i}"),
            Id::Tree(i) => format!("t{i}"),
        };
        let parse = |s: &str| {
            let i = s[1..].parse().unwrap();
            match &s[..1] {
                "p" => Id::Player(i),
                "d" => Id::Drop(i),
                "s" => Id::Shot(i),
                _ => unreachable!(),
            }
        };
        let point = |p: &serde_json::Value| V::new(p[0].as_f64().unwrap(), p[1].as_f64().unwrap());
        for case in data["cases"].as_array().unwrap() {
            let mut game = Snowball::new("concrete-contacts");
            game.map.states.fill(1);
            game.map.obstacles.fill(-1);
            let initial = &case["initial"];
            for row in initial["players"].as_array().unwrap() {
                game.add_player(false);
                let p = game.players.last_mut().unwrap();
                p.setup = true;
                p.collision_handler = true;
                p.arrived = true;
                p.landed = true;
                p.parent = Parent::Map;
                p.pos = point(&row["pos"]);
                p.face = row["face"].as_f64().unwrap();
                p.animation("elf_rig_idle", 0.);
                p.animator.update(0.);
            }
            for row in initial["drops"].as_array().unwrap() {
                game.drops.push(Drop {
                    active: true,
                    parent: Parent::Map,
                    presence: Presence::default(),
                    entity_z: 0.,
                    opacity: 1.,
                    tilt: PI / 2.5,
                    setup: true,
                    landed: true,
                    colliding_player: None,
                    tile: 2080,
                    pos: point(&row["pos"]),
                    dropped: 0.,
                    arrived: true,
                    color: 0,
                    scale: std::array::from_fn(|i| row["scale"][i].as_f64().unwrap()),
                    rotation: row["rotation"].as_f64().unwrap(),
                    spin: None,
                    gone: false,
                    frozen: false,
                    z: row["z"].as_f64().unwrap(),
                });
            }
            for row in initial["shots"].as_array().unwrap() {
                let id = row["id"].as_u64().unwrap() as usize;
                let thrower = row["thrower"].as_u64().unwrap() as usize;
                game.shots.push(Shot {
                    id,
                    hit: None,
                    setup: true,
                    origin: point(&row["origin"]),
                    target: point(&row["target"]),
                    pos: point(&row["pos"]),
                    z: row["pos"][2].as_f64().unwrap(),
                    thrower,
                    big: false,
                    born: row["born"].as_f64().unwrap(),
                    gone: false,
                    random: 0.,
                    skew: 0.,
                });
            }
            game.reset_collisions();
            game.collision.order = initial["order"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| parse(s.as_str().unwrap()))
                .collect();
            // Original fixture holds only collision fall selection at .5. Match
            // that selected sequence without claiming full-game random parity.
            let falls: usize = case["frames"][0]["players"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p["listeners"].as_u64().unwrap() as usize)
                .sum();
            game.random = (0..10000)
                .find_map(|seed| {
                    let rng = Random::seeded(&format!("contact-fall-{seed}"));
                    let mut probe = rng.clone();
                    (0..falls)
                        .all(|_| (probe.next() * 3.).floor() == 1.)
                        .then_some(rng)
                })
                .unwrap();
            for frame in case["frames"].as_array().unwrap() {
                game.sounds.clear();
                game.time = frame["at"].as_f64().unwrap();
                game.finish_frame(1440., 800.);
                let actual = serde_json::json!({
                    "order":game.collision.order.iter().copied().map(label).collect::<Vec<_>>(),
                    "sounds":game.sounds,
                    "players":game.players.iter().map(|p|serde_json::json!({
                        "alive":p.alive,"face":p.face,"exit":if p.alive{-1.}else{p.death},
                        "playing":p.animator.playing_name(),"listeners":p.animator.snapshot()["listeners"],
                        "primary":p.powerups.primary,"secondary":p.powerups.secondary,"swapped":p.powerups.swapped
                    })).collect::<Vec<_>>(),
                    "drops":game.drops.iter().enumerate().filter(|(_,d)|!d.gone).map(|(i,d)|serde_json::json!({
                        "label":format!("d{i}"),"spin":d.spin.unwrap_or(-1.),"colliding":d.colliding_player.map(|i|format!("p{i}")),
                        "rotation":d.rotation,"z":d.z,"scale":d.scale
                    })).collect::<Vec<_>>(),
                    "shots":game.shots.iter().map(|s|serde_json::json!({
                        "id":s.id,"hit":s.hit.map(label),"pos":[s.pos.x,s.pos.y,s.z],"born":s.born
                    })).collect::<Vec<_>>()
                });
                let mut expected = frame.clone();
                for key in ["at", "contactLog", "afterCollision"] {
                    expected.as_object_mut().unwrap().remove(key);
                }
                assert_json_close(
                    &actual,
                    &expected,
                    &format!("{} at {}", case["name"], frame["at"]),
                );
            }
        }
    }
    pub(crate) fn assert_json_close(a: &serde_json::Value, b: &serde_json::Value, path: &str) {
        match (a, b) {
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-9,
                "{path}: {a} vs {b}"
            ),
            (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    assert_json_close(a, b, &format!("{path}[{i}]"));
                }
            }
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (k, v) in a {
                    assert_json_close(v, &b[k], &format!("{path}.{k}"));
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    #[test]
    fn player_calls_match_actual_elf_update() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-animation-reference.json"))
                .unwrap();
        for case in data["elfCases"].as_array().unwrap() {
            let mut game = Snowball::new("animation-integration");
            game.start();
            game.players.truncate(1);
            game.drops.clear();
            game.shots.clear();
            game.all_time = 100;
            game.map.states.fill(1);
            game.map.obstacles.fill(-1);
            game.reset_collisions();
            // The source Elf pool retains these incoming transforms; this
            // comparison isolates update ordering from pool implementation.
            game.players[0].pos = V::new(
                case["initial"]["position"][0].as_f64().unwrap(),
                case["initial"]["position"][1].as_f64().unwrap(),
            );
            game.players[0].face = case["initial"]["face"].as_f64().unwrap();
            game.players[0].arrived = true;
            game.players[0].landed = true;
            game.players[0].parent = Parent::Map;
            for (index, row) in case["states"].as_array().unwrap().iter().enumerate() {
                let op = &row["op"];
                game.time = op["at"].as_f64().unwrap();
                let target = || {
                    V::new(
                        op["target"][0].as_f64().unwrap(),
                        op["target"][1].as_f64().unwrap(),
                    )
                };
                match op["op"].as_str().unwrap() {
                    "idle" => game.players[0].path.clear(),
                    "move" => game.players[0].path = vec![target()],
                    "throw" => game.throw(0, target()),
                    "die" => {
                        let variant = op["variant"].as_u64().unwrap() as usize;
                        let seed = (0..100)
                            .map(|i| Random::seeded(&format!("fall-{i}")))
                            .find(|r| {
                                let mut r = r.clone();
                                (r.next() * 3.).floor() as usize + 1 == variant
                            })
                            .unwrap();
                        game.random = seed;
                        game.kill(0, false);
                        assert_eq!(game.players[0].fall, variant)
                    }
                    "update" => {}
                    _ => unreachable!(),
                }
                game.finish_frame(1440., 800.);
                let p = &game.players[0];
                let mut snapshot = p.animator.snapshot();
                snapshot["position"] = serde_json::json!([p.pos.x, p.pos.y, 0.]);
                snapshot["face"] = serde_json::json!(p.face);
                snapshot["alive"] = serde_json::json!(p.alive);
                crate::snowball_animation::tests::compare(
                    &snapshot,
                    row,
                    &format!("Elf.update fall {} step {index}", case["variant"]),
                );
            }
        }
    }
    #[test]
    fn map_grid_paths_match_original_chromium() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-reference.json")).unwrap();
        let grid = Grid::default();
        for row in data["coords"].as_array().unwrap() {
            let i = row["index"].as_i64().unwrap() as i32;
            let p = grid.pos(i);
            assert!((p.x - row["pos"][0].as_f64().unwrap()).abs() < 1e-9);
            assert!((p.y - row["pos"][1].as_f64().unwrap()).abs() < 1e-9);
            assert_eq!(grid.position_index(p), row["back"].as_i64().unwrap() as i32);
            assert_eq!(serde_json::json!(grid.neighbors(i)), row["neighbors"]);
            assert_eq!(serde_json::json!(grid.ring(i, 3)), row["ring"])
        }
        for row in data["maps"].as_array().unwrap() {
            let mut map = Map::new(grid, row["seed"].as_i64().unwrap() as u32);
            let mut step = 0;
            for snapshot in row["states"].as_array().unwrap() {
                let next = snapshot["step"].as_i64().unwrap();
                while step < next {
                    step += 1;
                    map.erode(step as f64 * 300.)
                }
                let expected = snapshot["states"].as_array().unwrap();
                for i in 0..4096 {
                    assert_eq!(
                        map.states[i] as u64,
                        expected[i * 2].as_u64().unwrap(),
                        "seed {} step {step} tile {i}",
                        row["seed"]
                    );
                    assert!((map.times[i] - expected[i * 2 + 1].as_f64().unwrap()).abs() < 1e-4);
                    assert_eq!(
                        map.obstacles[i] as i64,
                        snapshot["obstacles"][i].as_i64().unwrap()
                    )
                }
                let rings: Vec<_> = snapshot["rings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| {
                        if v.is_null() {
                            vec![]
                        } else {
                            v.as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_i64().unwrap() as i32)
                                .collect()
                        }
                    })
                    .collect();
                assert_eq!(map.rings, rings);
                assert_eq!(
                    map.passable_count as i64,
                    snapshot["passable"].as_i64().unwrap()
                );
                assert_eq!(map.random.0, snapshot["seed"].as_i64().unwrap() as u32);
                assert_eq!(
                    map.erode_random.0,
                    snapshot["erode_seed"].as_i64().unwrap() as u32
                );
            }
        }
        let map = Map::new(grid, 1);
        for p in data["paths"].as_array().unwrap() {
            let a = p["a"].as_i64().unwrap() as i32;
            let b = p["b"].as_i64().unwrap() as i32;
            assert_eq!(serde_json::json!(grid.path(a, b, &map)), p["path"]);
            assert_eq!(
                serde_json::json!(grid.waypoints(a, b, &map)),
                p["waypoints"]
            );
        }
    }
    #[test]
    fn shot_trajectory_matches_original() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-reference.json")).unwrap();
        for s in data["shots"].as_array().unwrap() {
            let mut shot = Shot {
                id: 0,
                hit: None,
                setup: true,
                origin: V::new(12., 34.),
                target: V::new(
                    s["target"][0].as_f64().unwrap(),
                    s["target"][1].as_f64().unwrap(),
                ),
                pos: V::default(),
                z: 0.,
                thrower: 0,
                big: false,
                born: 12.,
                gone: false,
                random: 0.,
                skew: 0.,
            };
            shot.advance(s["tick"].as_f64().unwrap());
            for (a, b) in [shot.pos.x, shot.pos.y, shot.z]
                .into_iter()
                .zip(s["pos"].as_array().unwrap())
            {
                assert!((a - b.as_f64().unwrap()).abs() < 1e-9)
            }
            assert_eq!(shot.gone, s["gone"].as_bool().unwrap())
        }
    }
    #[test]
    fn snowball_queue_matches_original_system_splice_and_lifo() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-collision-reference.json"))
                .unwrap();
        for case in data["removals"].as_array().unwrap() {
            let mut game = Snowball::new("queue-reference");
            game.map.obstacles.fill(-1);
            game.time = 1000.;
            let active = case["active"].as_array().unwrap();
            let pending = case["pending"].as_array().unwrap();
            for (i, kind) in active.iter().chain(pending).enumerate() {
                let kind = kind.as_str().unwrap();
                let origin = V::new(10000. + i as f64 * 1000., 10000.);
                game.shots.push(Shot {
                    id: i,
                    hit: None,
                    setup: i < active.len(),
                    origin,
                    target: origin.add(V::new(256., 0.)),
                    pos: origin,
                    z: 0.,
                    thrower: usize::MAX,
                    big: false,
                    born: if i >= active.len() {
                        -1.
                    } else if kind == "expire" {
                        0.
                    } else {
                        60.
                    },
                    gone: kind == "hit",
                    random: i as f64,
                    skew: 0.,
                });
            }
            game.reset_collisions();
            game.finish_frame(1440., 800.);
            let remaining: Vec<_> = game.shots.iter().map(|s| s.random as usize).collect();
            assert_eq!(
                serde_json::json!(remaining),
                case["remaining"],
                "{}",
                case["name"]
            );
            for s in &game.shots {
                assert!(s.setup);
                if s.random as usize >= active.len() {
                    let updated = case["updated"]
                        .as_array()
                        .unwrap()
                        .contains(&serde_json::json!(s.random as usize));
                    assert_eq!(s.born, if updated { 60. } else { -1. }, "{}", case["name"]);
                }
            }
            assert_eq!(
                game.particles.iter().filter(|p| !p.trail).count(),
                case["removed"].as_array().unwrap().len() * 5
            );
        }
    }
    #[test]
    fn map_tree_hits_both_shots_before_splice_skips_the_second() {
        let mut game = Snowball::new("tree-integration");
        game.map.obstacles.fill(-1);
        let tile = 2080;
        game.map.obstacles[tile] = 0;
        let center = game.map.grid.pos(tile as i32).add(V::new(0., -17.));
        game.camera = V::new(center.x, -center.y * 0.75);
        game.collision_limit = Some((1440., 800., center));
        for i in 0..2 {
            game.shots.push(Shot {
                id: i,
                hit: None,
                setup: true,
                origin: center,
                target: center.add(V::new(256., 0.)),
                pos: center,
                z: 32.,
                thrower: usize::MAX,
                big: false,
                born: 0.,
                gone: false,
                random: i as f64,
                skew: 0.,
            });
        }
        game.reset_collisions();
        game.finish_frame(1440., 800.);
        assert_eq!(game.shots.len(), 1);
        assert_eq!(game.shots[0].random, 1.);
        assert!(game.shots[0].gone);
        assert_eq!(game.sounds, ["snowball_hit"; 4]);
        // Source MapSystem does not remove colliders as their tile erodes.
        game.map.states[tile] = 4;
        game.finish_frame(1440., 800.);
        assert!(game.shots.is_empty());
    }
    #[test]
    fn collision_limit_uses_previous_lod_camera_and_resets_on_resize() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-collision-reference.json"))
                .unwrap();
        let reference = &data["limit"];
        let mut game = Snowball::new("collision-limit");
        game.map.obstacles.fill(-1);
        let tile = reference["tile"].as_u64().unwrap() as usize;
        game.map.obstacles[tile] = 0;
        let center = game.map.grid.pos(tile as i32).add(V::new(0., -17.));
        assert_eq!(center.x, reference["center"][0].as_f64().unwrap());
        assert_eq!(center.y, reference["center"][1].as_f64().unwrap());
        for frame in reference["frames"].as_array().unwrap() {
            game.shots.clear();
            game.sounds.clear();
            game.camera = V::new(
                frame["camera"][0].as_f64().unwrap(),
                frame["camera"][1].as_f64().unwrap(),
            );
            game.shots.push(Shot {
                id: 0,
                hit: None,
                setup: true,
                origin: center,
                target: center.add(V::new(256., 0.)),
                pos: center,
                z: 32.,
                thrower: usize::MAX,
                big: false,
                born: 0.,
                gone: false,
                random: 0.,
                skew: 0.,
            });
            game.reset_collisions();
            game.finish_frame(
                frame["width"].as_f64().unwrap(),
                frame["height"].as_f64().unwrap(),
            );
            assert_eq!(game.sounds.len(), frame["hits"].as_u64().unwrap() as usize);
        }
    }
    #[test]
    fn powerups_match_original() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-reference.json")).unwrap();
        let mut p = Powerups::default();
        for row in data["powerups"].as_array().unwrap() {
            match row["action"].as_str().unwrap() {
                "collect" => p.collect(),
                "decrement" => p.decrement(),
                "swap" => p.swapped = !p.swapped,
                "pop" => p.set_active(None),
                _ => unreachable!(),
            }
            for (a, b) in [
                (p.primary, &row["primary"]),
                (p.secondary, &row["secondary"]),
                (p.active(), &row["active"]),
            ] {
                assert_eq!(a, b["quantity"].as_i64().map(|v| v as i32))
            }
            assert_eq!(p.swapped, row["swapped"].as_bool().unwrap())
        }
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    fn vector(v: V) -> serde_json::Value {
        serde_json::json!([v.x, v.y])
    }
    fn snapshot(g: &Snowball) -> serde_json::Value {
        serde_json::json!({
            "camera":vector(g.camera),
            "cameraVelocity":vector(g.camera_velocity),
            "sounds":g.sounds,
            "pick":g.pending_destination.map(vector),
            "clientDestination":g.client_destination.map(vector),
            "clientTarget":g.client_target.map(vector),
            "players":g.players.iter().map(|p|serde_json::json!({"position":vector(p.pos),"arrived":p.arrived,"alive":p.alive,"face":p.face,"path":p.path.iter().copied().map(vector).collect::<Vec<_>>(),"pending":p.pending_throw,"powerups":[p.powerups.primary,p.powerups.secondary,p.powerups.swapped]})).collect::<Vec<_>>(),
            "destinations":g.player_destinations.iter().map(|(i,p)|serde_json::json!([i,vector(*p)])).collect::<Vec<_>>(),
            "targets":g.player_targets.iter().map(|(i,p)|serde_json::json!([i,vector(match p {TargetPosition::Point(p)=>*p,TargetPosition::Player(j)=>g.players[*j].pos})])).collect::<Vec<_>>(),
            "shots":g.shots.iter().map(|s|serde_json::json!({"id":s.id,"thrower":s.thrower,"origin":vector(s.origin),"target":vector(s.target),"born":s.born,"setup":s.setup,"big":s.big})).collect::<Vec<_>>(),
            "marker":g.destination_marker.map(vector)
        })
    }
    fn compare(a: &serde_json::Value, b: &serde_json::Value, path: &str) {
        match (a, b) {
            (serde_json::Value::Number(x), serde_json::Value::Number(y)) => assert!(
                (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() < 1e-8,
                "{path}: {a} != {b}"
            ),
            (serde_json::Value::Array(x), serde_json::Value::Array(y)) => {
                assert_eq!(x.len(), y.len(), "{path}: {a} != {b}");
                for (i, (x, y)) in x.iter().zip(y).enumerate() {
                    compare(x, y, &format!("{path}[{i}]"));
                }
            }
            (serde_json::Value::Object(x), serde_json::Value::Object(y)) => {
                assert_eq!(x.len(), y.len(), "{path}");
                for (k, x) in x {
                    compare(x, &y[k], &format!("{path}.{k}"));
                }
            }
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    fn command(g: &mut Snowball, c: &serde_json::Value) {
        let i = c["player"].as_u64().unwrap_or(0) as usize;
        let position = || {
            V::new(
                c["position"][0].as_f64().unwrap(),
                c["position"][1].as_f64().unwrap(),
            )
        };
        match c["op"].as_str().unwrap() {
            "map" => {
                let p = if c["position"].is_array() {
                    position()
                } else {
                    g.map.grid.pos(c["tile"].as_i64().unwrap() as i32)
                };
                g.pick_destination(p);
            }
            "target" => g.target_position(g.players[i].pos),
            "relocate" => g.players[i].pos = position(),
            "arrive" => g.players[0].arrived = true,
            "dead" => g.players[i].alive = !c["value"].as_bool().unwrap(),
            "collect" => g.players[i].powerups.collect(),
            "swap" => g.players[i].powerups.swapped = !g.players[i].powerups.swapped,
            "queue-target" => g.assign_player_target(i, position()),
            "queue-player-target" => g.queue_player_target(
                i,
                TargetPosition::Player(c["target"].as_u64().unwrap() as usize),
            ),
            "queue-destination" => {
                g.assign_player_destination(i, g.map.grid.pos(c["tile"].as_i64().unwrap() as i32))
            }
            _ => panic!("command {c}"),
        }
    }
    #[test]
    fn command_stages_match_original_client_player_and_snowball_systems() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-command-reference.json")).unwrap();
        assert_eq!(data["cases"].as_array().unwrap().len(), 14);
        for case in data["cases"].as_array().unwrap() {
            let mut g = Snowball::new("command-reference");
            g.start();
            g.players.truncate(2);
            g.drops.clear();
            g.shots.clear();
            g.particles.clear();
            g.parachutes = Parachutes::default();
            g.all_time = 100;
            g.map.states.fill(1);
            g.map.obstacles.fill(-1);
            for i in case["blocked"].as_array().unwrap() {
                g.map.states[i.as_u64().unwrap() as usize] = 0;
            }
            for (i, p) in g.players.iter_mut().enumerate() {
                p.tile = 2080 + i as i32;
                p.arrived = !(i == 0 && case["air"].as_bool().unwrap());
                p.landed = p.arrived;
                p.setup = true;
                p.bot = false;
                p.active = true;
                p.alive = true;
                p.face = 0.;
                p.path.clear();
                p.pending_throw = false;
                p.pos = if p.arrived {
                    g.map.grid.pos(p.tile)
                } else {
                    V::default()
                };
                p.parent = if p.arrived {
                    Parent::Map
                } else {
                    Parent::Detached
                };
            }
            g.sounds.clear();
            compare(
                &snapshot(&g),
                &case["initial"],
                &format!("{} initial", case["name"]),
            );
            let mut previous = 0.;
            for (index, row) in case["frames"].as_array().unwrap().iter().enumerate() {
                g.sounds.clear();
                let at = row["at"].as_f64().unwrap();
                g.begin_frame((at - previous) / 1000., 1440., 800.);
                previous = at;
                let label = format!("{} frame {index}", case["name"]);
                compare(&snapshot(&g), &row["render"], &format!("{label} render"));
                for c in row["input"]["input"].as_array().into_iter().flatten() {
                    command(&mut g, c);
                }
                compare(&snapshot(&g), &row["afterInput"], &format!("{label} input"));
                // These explicit bot-boundary calls only assign queues/flags;
                // no source AI/physics decision is replaced or certified here.
                for c in row["input"]["bot"].as_array().into_iter().flatten() {
                    command(&mut g, c);
                }
                // The original oracle has no collision phase. Isolate the same
                // boundary while running actual production begin/finish code.
                g.collision = Collision::new(Bounds::rect(
                    V::default(),
                    g.map.grid.pixel_width(),
                    g.map.grid.pixel_height(),
                ));
                g.finish_frame(1440., 800.);
                compare(
                    &snapshot(&g),
                    &row["afterPlayer"],
                    &format!("{label} player"),
                );
            }
        }
    }
}

#[cfg(test)]
mod lod_tests {
    use super::*;
    use crate::snowball_lod::ClientRing;
    fn snapshot(g: &Snowball) -> serde_json::Value {
        let parent = |p: Parent| match p {
            Parent::Detached => serde_json::Value::Null,
            Parent::Map => serde_json::json!("map"),
            Parent::Canopy => serde_json::json!("canopy"),
            Parent::Ice(i) => serde_json::json!(format!("ice{i}")),
        };
        let scene_visible = g.client_ring.visible
            && g.client_ring.owner.is_some_and(|i| {
                let p = &g.players[i];
                p.lod.root_visible
                    && match p.parent {
                        Parent::Detached => false,
                        Parent::Map => true,
                        Parent::Canopy => g.parachutes.get(Id::Player(i)).is_some(),
                        Parent::Ice(slot) => g.removal.layer.contains(&slot),
                    }
            });
        serde_json::json!({"camera":[g.camera.x,g.camera.y],"actors":g.players.iter().map(|p|serde_json::json!({"position":[p.pos.x,p.pos.y,p.z],"arrived":p.arrived,"alive":p.alive,"active":p.active,"exiting":p.presence.exiting(),"gone":p.presence.gone(),"high":p.lod.high,"dirty":p.lod.needs_update,"visible":p.lod.root_visible,"requested":p.lod.model_requested,"parent":parent(p.parent)})).collect::<Vec<_>>(),"ring":{"owner":g.client_ring.owner.map(|i|i as i64).unwrap_or(-1),"visible":g.client_ring.visible,"sceneVisible":scene_visible}})
    }
    fn apply(g: &mut Snowball, c: &serde_json::Value, width: f64) {
        let i = c["player"].as_u64().unwrap_or(0) as usize;
        match c["op"].as_str().unwrap() {
            "position" => {
                g.players[i].pos = V::new(
                    c["position"][0].as_f64().unwrap(),
                    c["position"][1].as_f64().unwrap(),
                );
                g.players[i].z = c["position"][2].as_f64().unwrap();
            }
            "dead" => g.players[i].alive = !c["value"].as_bool().unwrap(),
            "exit" => {
                if c["value"].as_bool().unwrap() {
                    g.players[i].presence.exit(g.time)
                } else {
                    g.players[i].presence.exit_time = -1.;
                }
            }
            "erode" => {
                let tile = g.map.grid.position_index(g.players[i].pos);
                g.map.states[tile as usize] = 4;
            }
            "lod" => {
                let p = &mut g.players[i];
                p.lod.assign(p.alive, c["high"].as_bool().unwrap());
            }
            "teleport" => g.kill(i, false),
            "canopy-edge" => {
                let p = g.map.grid.pos(g.players[i].tile);
                g.camera = V::new(
                    p.x - (width + 256.) / 2. - c["epsilon"].as_f64().unwrap(),
                    p.y * -0.75,
                );
            }
            _ => panic!("LOD command {c}"),
        }
    }
    #[test]
    fn lod_and_shared_ring_match_original_core_system_stages() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-lod-reference.json")).unwrap();
        assert_eq!(data["cases"].as_array().unwrap().len(), 8);
        for case in data["cases"].as_array().unwrap() {
            let mut g = Snowball::new("lod-reference");
            g.start();
            g.players.truncate(2);
            g.drops.clear();
            g.shots.clear();
            g.particles.clear();
            g.parachutes = Parachutes::default();
            g.map.states.fill(1);
            g.map.obstacles.fill(-1);
            g.map.rings.resize(4, vec![]);
            g.all_time = 100;
            for (i, p) in g.players.iter_mut().enumerate() {
                p.tile = if i == 0 {
                    case["tile"].as_i64().unwrap() as i32
                } else {
                    2081
                };
                p.arrived = !(i == 0 && case["air"].as_bool().unwrap());
                p.landed = p.arrived;
                p.setup = false;
                p.active = true;
                p.bot = false;
                p.face = 0.;
                p.pos = if p.arrived {
                    g.map.grid.pos(p.tile)
                } else {
                    V::default()
                };
                p.z = 0.;
                p.parent = if p.arrived {
                    Parent::Map
                } else {
                    Parent::Detached
                };
            }
            let (mut width, mut height) = (1440., 800.);
            for (index, row) in case["frames"].as_array().unwrap().iter().enumerate() {
                let input = &row["input"];
                g.time = input["at"].as_f64().unwrap();
                g.wall = g.time;
                if input["size"].is_array() {
                    width = input["size"][0].as_f64().unwrap();
                    height = input["size"][1].as_f64().unwrap();
                }
                if input["camera"].is_array() {
                    g.camera = V::new(
                        input["camera"][0].as_f64().unwrap(),
                        input["camera"][1].as_f64().unwrap(),
                    );
                }
                for c in input["ops"].as_array().into_iter().flatten() {
                    apply(&mut g, c, width);
                }
                // The source harness runs these core stages with no collision
                // or LocalLevel/AI/loaded-model phase. Match that isolation.
                g.collision = Collision::new(Bounds::rect(
                    V::default(),
                    g.map.grid.pixel_width(),
                    g.map.grid.pixel_height(),
                ));
                g.finish_frame(width, height);
                let label = format!("{} frame {index}", case["name"]);
                crate::snowball::tests::assert_json_close(&snapshot(&g), &row["after"], &label);
                for c in input["post"].as_array().into_iter().flatten() {
                    apply(&mut g, c, width);
                }
                crate::snowball::tests::assert_json_close(&snapshot(&g), &row["post"], &label);
            }
        }
    }
    #[test]
    fn shared_ring_ownership_and_teardown_match_original_elf() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-lod-reference.json")).unwrap();
        let mut ring = ClientRing {
            owner: data["ringInitial"]["owner"]
                .as_i64()
                .filter(|i| *i >= 0)
                .map(|i| i as usize),
            visible: data["ringInitial"]["visible"].as_bool().unwrap(),
        };
        let mut client = 0;
        let mut states = [(false, true, false); 2]; // arrived, alive, exiting
        for row in data["ringFrames"].as_array().unwrap() {
            let input = &row["input"];
            let i = input["player"].as_u64().unwrap() as usize;
            if let Some(v) = input["arrived"].as_bool() {
                states[i].0 = v;
            }
            if let Some(v) = input["dead"].as_bool() {
                states[i].1 = !v;
            }
            if let Some(v) = input["exiting"].as_bool() {
                states[i].2 = v;
            }
            match input["op"].as_str().unwrap() {
                "setup" => ring.setup(i),
                "client" => client = i,
                "teardown" => {}
                "update" => ring.update(i, i == client, states[i].0, states[i].1, states[i].2),
                _ => panic!("ring command"),
            }
            assert_eq!(
                ring.owner.map(|n| n as i64).unwrap_or(-1),
                row["owner"].as_i64().unwrap(),
                "{input}"
            );
            assert_eq!(ring.visible, row["visible"].as_bool().unwrap(), "{input}");
        }
    }
}
