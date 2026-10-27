//! Snowball Storm local scene, translated from the original 2020 Google source.
//! Copyright 2020 Google LLC, Apache-2.0. Browser/network scaffolding is separate.
use crate::random::Random;
use crate::snowball_collision::{Bounds, Id, Quad};
use serde::{Deserialize, Serialize};
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
    pub anim: String,
    pub anim_start: f64,
    pub throwing: Option<f64>,
    pub fall: usize,
    pub ice: V,
    pub ice_start: f64,
}
impl Player {
    fn animation(&mut self, name: &str, now: f64) {
        if self.anim != name {
            self.anim = name.into();
            self.anim_start = now
        }
    }
}
#[derive(Clone)]
pub struct Shot {
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
    pub freeze_at: f64,
    pub ice: V,
}
#[derive(Clone)]
pub struct Particle {
    pub pos: V,
    pub z: f64,
    pub direction: V,
    pub born: f64,
    pub size: f64,
    pub random: f64,
    pub trail: bool,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Splash,
    Intro,
    Playing,
}
pub struct Snowball {
    pub map: Map,
    pub players: Vec<Player>,
    pub drops: Vec<Drop>,
    pub shots: Vec<Shot>,
    pub particles: Vec<Particle>,
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
    camera_target: Option<V>,
}
impl Snowball {
    pub fn new(seed: &str) -> Self {
        let mut random = Random::seeded(seed);
        let map_seed = (random.next() * 4294967296.) as u32;
        Self {
            map: Map::new(Grid::default(), map_seed),
            players: vec![],
            drops: vec![],
            shots: vec![],
            particles: vec![],
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
            camera_target: None,
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
        self.pending_destination = None;
        self.camera_target = None;
        self.knocked_out = 0;
        self.hud_knockouts = 0;
        self.hud_time = 0.;
        self.all_time = 0;
        self.players.clear();
        self.drops.clear();
        self.shots.clear();
        self.particles.clear();
        self.last_erode = 0.;
        self.last_drop = 0.;
        self.last_bot = 0.;
        self.seed = (self.random.next() * 4294967296.) as u32;
        self.map = Map::new(Grid::default(), self.seed);
        self.add_player(false);
        for _ in 0..25 {
            self.add_drop();
            self.add_player(true)
        }
        let p = self.map.grid.pos(self.players[0].tile);
        self.camera = V::new(p.x, -0.75 * (p.y - 10.));
        self.camera_velocity = V::default();
        self.sounds.push("music_start_ingame")
    }
    fn add_player(&mut self, bot: bool) {
        let tile = self.map.habitable();
        let color = (self.random.next() * 6.) as usize;
        let skin = (self.random.next() * 4.) as usize;
        let hair = (self.random.next() * 4.) as usize;
        let pony = self.random.next() >= 0.5;
        self.players.push(Player {
            pos: V::default(),
            z: 0.,
            tile,
            dropped: (self.time * 60. / 1000.).floor() + 1.,
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
            anim: "elf_rig_idle".into(),
            anim_start: self.time,
            throwing: None,
            fall: 1,
            ice: V::default(),
            ice_start: 0.,
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
            tile,
            pos: V::default(),
            dropped: (self.time * 0.06).floor(),
            arrived: false,
            color,
            scale,
            rotation: 0.,
            spin: None,
            gone: false,
            frozen: false,
            z: 32.,
            freeze_at: 0.,
            ice: V::default(),
        })
    }
    pub fn destination(&mut self, index: usize, pos: V) {
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
            self.throw(index, pos)
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
        self.players[index].face = (target.y - origin.y).atan2(target.x - origin.x) + FRAC_PI_2;
        self.players[index].throwing = Some(self.time);
        if index == 0 {
            self.sounds.push("snowball_throw")
        }
    }
    fn kill(&mut self, index: usize, frozen: bool) {
        let p = &mut self.players[index];
        if !p.alive {
            return;
        }
        p.alive = false;
        p.frozen = frozen;
        p.death = self.time;
        p.fall = (self.random.next() * 3.).floor() as usize + 1;
        p.animation(&format!("elf_rig_fall_down_{}", p.fall), self.time);
        if frozen {
            p.ice = self
                .map
                .grid
                .pos(self.map.grid.position_index(p.pos))
                .add(V::new(0., -32.));
            p.ice_start = self.time
        }
    }
    pub fn advance(&mut self, seconds: f64, width: f64, height: f64) {
        for _ in 0..(seconds * 60.).round() as usize {
            self.step(1. / 60., width, height)
        }
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
            if self.players[0].alive {
                self.destination(0, destination)
            } else {
                self.camera_target = Some(destination)
            }
        }
        if let Some(p) = self.players.first() {
            let target = if p.alive && p.arrived {
                Some(p.pos)
            } else if !p.alive {
                self.camera_target
            } else {
                None
            };
            if let Some(target) = target {
                let delta = V::new(target.x, -0.75 * target.y).sub(self.camera);
                let tether = 0.05 * width.max(height);
                if delta.len() >= tether {
                    self.camera_velocity = self.camera_velocity.add(delta.norm().mul(0.15))
                } /* source's else uses global window.length (0), subtracting zero */
                if self.camera_velocity.len() > 0.1 {
                    self.camera_velocity.x += if self.camera_velocity.x > 0. {
                        -0.1
                    } else {
                        0.1
                    };
                    self.camera_velocity.y += if self.camera_velocity.y > 0. {
                        -0.1
                    } else {
                        0.1
                    }
                } else {
                    self.camera_velocity = V::default()
                }
                self.camera = self.camera.add(self.camera_velocity)
            }
        }
        true
    }
    /// Original render/input precede all simulation systems. Keep one update
    /// per presented frame; tick is wall time, not a fixed-step catch-up loop.
    pub fn finish_frame(&mut self, width: f64, height: f64) {
        let grid = self.map.grid;
        let tick = (self.time * 0.06).floor();
        // Collision notifications precede snowball movement; bounds/LOD constrain checks.
        let mut quad = Quad::new(Bounds::rect(
            V::default(),
            grid.pixel_width(),
            grid.pixel_height(),
        ));
        for (i, p) in self.players.iter().enumerate() {
            if p.alive && !p.gone {
                quad.add(Id::Player(i), Bounds::rect(p.pos, 15., 45.));
            }
        }
        for (i, d) in self.drops.iter().enumerate() {
            if d.arrived && !d.gone && d.spin.is_none() {
                quad.add(Id::Drop(i), Bounds::circle(d.pos, 10.));
            }
        }
        for (i, s) in self.shots.iter().enumerate() {
            if s.born >= 0. && !s.gone {
                quad.add(
                    Id::Shot(i),
                    Bounds::circle(s.pos, if s.big { 20. } else { 6. }),
                );
            }
        }
        let mut kills = vec![];
        for s in &mut self.shots {
            if s.gone || s.born < 0. {
                continue;
            }
            let limit = V::new(self.camera.x, -self.camera.y * 4. / 3.);
            if (s.pos.x - limit.x).abs() > width / 2. + 128.
                || (s.pos.y - limit.y).abs() > height * 2. / 3. + 128.
            {
                continue;
            }
            let radius = if s.big { 20. } else { 6. };
            let mut hit = false;
            let nearby = quad.near(Bounds::circle(s.pos, radius));
            for (i, p) in self.players.iter().enumerate() {
                if !nearby.contains(&Id::Player(i)) {
                    continue;
                }
                if i == s.thrower || !p.alive || !p.arrived {
                    continue;
                }
                if circle_rect(s.pos, radius, p.pos, 7.5, 22.5) {
                    kills.push(i);
                    hit = true
                }
            }
            // Static Tree colliders are registered after bounds creation. The
            // original system never inserts them into its quad-tree in local mode.
            for (di, d) in self.drops.iter().enumerate() {
                if !nearby.contains(&Id::Drop(di)) {
                    continue;
                }
                if d.arrived && !d.gone && d.spin.is_none() && s.pos.sub(d.pos).len() < radius + 10.
                {
                    hit = true
                }
            }
            if hit {
                s.gone = true;
                self.sounds.push("snowball_hit")
            }
        }
        for i in kills {
            self.kill(i, false)
        }
        let mut removed = vec![];
        let mut i = 0;
        while i < self.shots.len() {
            if self.shots[i].gone {
                removed.push(self.shots.remove(i));
                continue;
            }
            let s = &mut self.shots[i];
            if s.born < 0. {
                s.born = tick
            } else {
                s.advance(self.time * 0.06)
            }
            if s.gone {
                removed.push(self.shots.remove(i)); /* source splice skips next */
                i += 1
            } else {
                i += 1
            }
        }
        for s in removed {
            for _ in 0..if s.big { 7 } else { 5 } {
                self.particles.push(Particle {
                    pos: s.pos,
                    z: s.z,
                    direction: s.target.sub(s.origin).norm(),
                    born: self.time,
                    size: if s.big { 75. } else { 40. },
                    random: self.random.next(),
                    trail: false,
                })
            }
        }
        for s in &self.shots {
            self.particles.push(Particle {
                pos: s.pos,
                z: s.z + 1.,
                direction: V::default(),
                born: self.time,
                size: if s.big { 10. } else { 6. },
                random: 0.,
                trail: true,
            })
        }
        self.particles
            .retain(|p| self.time - p.born < if p.trail { 700. } else { 300. });
        for d in &mut self.drops {
            if !d.arrived && tick - d.dropped >= 300. {
                d.arrived = true;
                d.pos = grid.pos(d.tile)
            }
            if !d.arrived || d.gone {
                continue;
            }
            if d.frozen {
                d.ice = d.ice.add(d.ice.norm().mul(1. / 3.));
                if d.ice.len()
                    > V::new(width / 2., height / 2.).len()
                        + (self.map.rings.len() as f64 + 2.) * 64.
                {
                    d.gone = true
                }
            } else if let Some(start) = d.spin {
                let t = ((self.time - start) / 300.).min(1.);
                let delta = -((t - 1.).powi(4) + (t - 1.).powi(3));
                d.rotation += delta * 2. * PI;
                d.z += delta * 64.;
                for s in &mut d.scale {
                    *s *= 1f64.max(1. + delta)
                }
                if t == 1. {
                    d.gone = true
                }
            } else if self.map.state(grid.position_index(d.pos)) == 4 {
                d.frozen = true;
                d.gone = true
            } else {
                d.rotation += 0.01;
                for (i, p) in self.players.iter_mut().enumerate() {
                    if p.alive && p.arrived && circle_rect(d.pos, 10., p.pos, 7.5, 22.5) {
                        p.powerups.collect();
                        d.spin = Some(self.time);
                        if i == 0 {
                            self.sounds.push("snowball_pickup")
                        }
                        break;
                    }
                }
            }
        }
        if self.players.first().is_some_and(|p| p.alive) {
            self.hud_knockouts = self.knocked_out;
            self.hud_time = self.time / 1000.;
        }
        // Bots consume randomness per rendered update, not a catch-up fixed loop.
        let mut moves = vec![];
        let mut throws = vec![];
        for i in 1..self.players.len() {
            let p = &self.players[i];
            if !p.arrived || p.gone {
                continue;
            }
            if p.path.is_empty() && self.random.next() > 0.985 {
                let tile = self
                    .map
                    .nearby(grid.position_index(p.pos), 6, &mut self.random);
                moves.push((i, grid.pos(tile)))
            }
            let nearby = quad.near(Bounds::rect(p.pos, 200., 200.));
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
                        throws.push((i, self.players[j].pos))
                    }
                    self.players[i].last_throw = tick
                }
            }
            self.players[i].last_target = target;
        }
        for (i, p) in moves {
            self.destination(i, p)
        }
        for (i, p) in throws {
            self.throw(i, p)
        }
        let mut freezing = vec![];
        for (i, p) in self.players.iter_mut().enumerate() {
            if p.gone {
                continue;
            }
            if !p.arrived {
                if tick - p.dropped >= 300. {
                    p.arrived = true;
                    p.pos = grid.pos(p.tile)
                } else {
                    continue;
                }
            }
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
                if self.map.state(grid.position_index(p.pos)) == 4 {
                    freezing.push(i)
                }
            } else {
                let elapsed = self.time - p.death;
                if p.frozen {
                    p.ice = p.ice.add(p.ice.norm().mul(1. / 3.));
                    let island = (self.map.rings.len() as f64 + 2.) * 64.;
                    if p.ice.len() > V::new(width / 2., height / 2.).len() + island {
                        p.gone = true;
                        self.knocked_out += 1;
                        self.sounds.push("snowball_frozen")
                    }
                } else {
                    let duration = [0., 1000. / 3., 1000. * 5. / 12., 375.][p.fall];
                    if elapsed >= duration && p.anim != format!("elf_rig_fall_down_{}_idle", p.fall)
                    {
                        p.animation(&format!("elf_rig_fall_down_{}_idle", p.fall), self.time)
                    }
                    if elapsed >= 2200. {
                        p.gone = true;
                        self.knocked_out += 1;
                        self.sounds.push("snowball_frozen")
                    }
                }
            }
            if i == 0 && !p.alive && (p.gone || self.time - p.death > 2500.) {
                self.gameover = true
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
mod tests {
    use super::*;
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
