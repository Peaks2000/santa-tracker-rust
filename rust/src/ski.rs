//! Elf Ski: physics.js, game-three.js/SceneDecorator, vec.js,
//! three/present.js and three/skitrail.js translated from the original source.
//! Copyright 2017–2020 Google LLC; Apache-2.0. Noise: Gustavson/Eastman/Gentle (public domain).
use crate::random::Random;
use serde::Serialize;
use std::collections::VecDeque;
const PI: f64 = std::f64::consts::PI;
const PERM: [usize; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,
];
pub fn noise(mut x: f64, mut y: f64) -> f64 {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    x -= ix as f64;
    y -= iy as f64;
    let (ix, iy) = ((ix & 255) as usize, (iy & 255) as usize);
    let dot = |x: f64, y: f64, i: usize| {
        let (xg, yg) = [
            (1., 1.),
            (-1., 1.),
            (1., -1.),
            (-1., -1.),
            (1., 0.),
            (-1., 0.),
            (1., 0.),
            (-1., 0.),
            (0., 1.),
            (0., -1.),
            (0., 1.),
            (0., -1.),
        ][i % 12];
        x * xg + y * yg
    };
    let p = |i: usize| PERM[i & 255];
    let fade = |t: f64| t * t * t * (t * (t * 6. - 15.) + 10.);
    let lerp = |a: f64, b: f64, t: f64| (1. - t) * a + t * b;
    lerp(
        lerp(
            dot(x, y, p(ix + p(iy))),
            dot(x - 1., y, p(ix + 1 + p(iy))),
            fade(x),
        ),
        lerp(
            dot(x, y - 1., p(ix + p(iy + 1))),
            dot(x - 1., y - 1., p(ix + 1 + p(iy + 1))),
            fade(x),
        ),
        fade(y),
    )
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Character {
    pub speed: f64,
    pub angle: f64,
    pub line: f64,
}
impl Default for Character {
    fn default() -> Self {
        Self {
            speed: 0.,
            angle: PI / 2.,
            line: 0.,
        }
    }
}
impl Character {
    pub fn tick(&mut self, delta: f64, target: Option<f64>) -> (f64, f64) {
        if let Some(target) = target.filter(|x| x.is_finite()) {
            let target = target.clamp(-PI / 2., PI / 2.);
            let by = 4. * delta * (1.32 - self.speed);
            self.angle += by * (target - self.angle);
            self.line = (target - self.angle).abs();
        } else {
            self.line = 0.;
        }
        let mut accel = self.angle.cos() - 0.5;
        if accel < 0. {
            accel = accel.tan();
        }
        self.speed = (self.speed + accel * delta).clamp(0., 0.66);
        (
            self.speed * self.angle.sin() * delta,
            self.speed * self.angle.cos() * delta,
        )
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Tree {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub sprite: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct Present {
    pub x: f64,
    pub y: f64,
    pub size: [f64; 3],
    pub color: usize,
    pub rotation: f64,
    pub time: f64,
    pub dir: f64,
    pub pickup_at: f64,
    pub scale: f64,
    pub lift: f64,
    pub attached: bool,
    pub cell: i64,
}
impl Present {
    fn new(random: &mut Random, x: f64, y: f64, cell: i64) -> Self {
        Self {
            x,
            y,
            size: [
                random.next() * 7. + 12.,
                random.next() * 7. + 12.,
                random.next() * 5. + 10.,
            ],
            color: (random.next() * 7.).trunc() as usize,
            rotation: random.next() * PI * 2.,
            time: random.next() * PI * 2.,
            dir: if random.next() > 0.5 { -1. } else { 1. },
            pickup_at: 0.,
            scale: 1.,
            lift: 0.,
            attached: true,
            cell,
        }
    }
    fn tick(&mut self, delta: f64) {
        self.time += delta;
        self.rotation += delta * PI * self.dir / 4.;
        self.lift = 24. + self.time.sin() * 4.;
        if self.pickup_at != 0. {
            let scale = 1. - (self.time - self.pickup_at) * 2.;
            if scale <= 0. {
                self.attached = false;
            } else {
                self.scale = scale;
            }
        }
    }
}
#[derive(Clone)]
struct Range {
    l: i64,
    r: i64,
    alloc: Vec<usize>,
}
pub struct Ski {
    pub character: Character,
    pub facing_angle: f64,
    pub x: f64,
    pub y: f64,
    pub camera_x: f64,
    pub camera_y: f64,
    pub trees: Vec<Option<Tree>>,
    pub presents: Vec<Present>,
    pub trail: VecDeque<(f64, f64)>,
    trail_last: (f64, f64),
    pub hit_tree: Option<f64>,
    pub tumble: [f64; 3],
    pub tumble_steps: Vec<[f64; 3]>,
    pub presents_found: u32,
    pub playing: bool,
    pub time: f64,
    pub score_time: f64,
    pub last_score: i64,
    pub sounds: Vec<&'static str>,
    pub width: f64,
    pub height: f64,
    random: Random,
    ox: f64,
    oy: f64,
    ranges: VecDeque<Range>,
    depth: i64,
    free: Vec<usize>,
    crash_reported: Option<f64>,
    skiing: bool,
    last_presents_found: u32,
}
impl Ski {
    pub fn new(seed: &str, width: f64, height: f64) -> Self {
        let mut random = Random::seeded(seed);
        let ox = random.next() * 1000. - 500.;
        let oy = random.next() * 1000. - 500.;
        let mut out = Self {
            character: Character::default(),
            facing_angle: PI / 2.,
            x: 0.,
            y: 800.,
            camera_x: 10.,
            camera_y: 800. + height / 4.,
            trees: vec![],
            presents: vec![],
            trail: std::iter::repeat_n((0., 800.), 80).collect(),
            trail_last: (0., 800.),
            hit_tree: None,
            tumble: [0.; 3],
            tumble_steps: vec![],
            presents_found: 0,
            playing: true,
            time: 0.,
            score_time: 0.,
            last_score: 0,
            sounds: vec!["music_start_ingame"],
            width,
            height,
            random,
            ox,
            oy,
            ranges: VecDeque::new(),
            depth: 0,
            free: vec![],
            crash_reported: None,
            skiing: false,
            last_presents_found: 0,
        };
        out.internal(0., 0.);
        out
    }
    fn n(&self, x: f64, y: f64) -> f64 {
        noise(x + self.ox, y + self.oy)
    }
    pub fn tree_for_cell(&self, x: i64, ay: i64) -> Option<(f64, f64)> {
        let y = ay - 10;
        if y < 0 {
            return None;
        }
        let (mut v, xf, yf) = (
            self.n(x as f64 / 3.27, y as f64 / 8.742),
            x as f64,
            y as f64,
        );
        if y < 5 {
            v -= (5 - y) as f64 * 0.1;
        }
        if v <= -(y as f64) * 0.001 && v > -0.5 {
            return None;
        }
        Some((
            96. * (xf + 0.5 + 2. * self.n(xf / 4.1222, yf / 8.2421)),
            96. * (ay as f64 + 0.5 + 2. * self.n(xf / 1.21, yf / 2.31)),
        ))
    }
    fn present_for_cell(&self, x: i64, y: i64) -> Option<(f64, f64)> {
        if y < 20 {
            return None;
        }
        let (x, y) = (x as f64, y as f64);
        let v = self.n(x / 12.24, y / 13.531);
        if !(0.0..=0.12).contains(&v) {
            return None;
        }
        Some((
            96. * (x + 0.5 + 2. * self.n(x / 3.1051, y / 3.951)),
            96. * (y + 0.5 + 2. * self.n(x / 6.71, y / 152.1535)),
        ))
    }
    fn decorate(&mut self, x: i64, y: i64, alloc: &mut Vec<usize>) {
        if let Some((px, py)) = self.tree_for_cell(x, y) {
            let sprite = (((0.5 + self.n(x as f64 / 1.00124, y as f64 / 0.1241)) * 6.).trunc()
                as i64)
                .clamp(0, 5) as usize;
            let id = if let Some(id) = self.free.pop() {
                id
            } else {
                let id = self.trees.len();
                assert!(id < 10000);
                self.trees.push(None);
                id
            };
            self.trees[id] = Some(Tree {
                id,
                x: px,
                y: py,
                sprite,
            });
            alloc.push(id);
        }
        if let Some((px, py)) = self.present_for_cell(x, y) {
            self.presents
                .push(Present::new(&mut self.random, px, py, y));
        }
    }
    /// GameThree.render updates the visible terrain separately from 60 Hz simulation.
    pub fn render_update(&mut self, width: f64, height: f64) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.internal(0., 0.);
        }
        if self.hit_tree.is_some() {
            return;
        }
        let high = ((self.y + height / 2. * 2.5) / 96.).ceil() as i64;
        let low = ((self.y - height / 2.) / 96.).floor() as i64;
        while self.depth < high {
            self.ranges.push_back(Range {
                l: 0,
                r: 0,
                alloc: vec![],
            });
            self.depth += 1;
        }
        let mut freed = false;
        while self.ranges.len() as i64 > high - low {
            let old = self.ranges.pop_front().unwrap();
            for id in old.alloc {
                self.trees[id] = None;
                self.free.push(id);
            }
            freed = true;
        }
        if freed {
            self.presents.retain(|p| {
                let cell = if p.cell == 0 { -1 } else { p.cell };
                cell >= low
            });
        }
        let lc = ((self.x - width / 2. * 2.5) / 96.).floor() as i64;
        let rc = ((self.x + width / 2. * 2.5) / 96.).ceil() as i64;
        let offset = self.depth - self.ranges.len() as i64;
        for i in 0..self.ranges.len() {
            let mut range = self.ranges[i].clone();
            while range.l > lc {
                range.l -= 1;
                self.decorate(range.l, i as i64 + offset, &mut range.alloc);
            }
            while range.r <= rc {
                self.decorate(range.r, i as i64 + offset, &mut range.alloc);
                range.r += 1;
            }
            self.ranges[i] = range;
        }
    }
    fn trail_push(&mut self) {
        let dist = (self.trail_last.0 - self.x).hypot(self.trail_last.1 - self.y);
        if dist == 0. {
            return;
        }
        if dist < 5. {
            *self.trail.back_mut().unwrap() = (self.x, self.y);
        } else {
            self.trail.pop_front();
            self.trail.push_back((self.x, self.y));
            self.trail_last = (self.x, self.y);
        }
    }
    fn internal(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
        self.trail_push();
        let angle = self.character.angle;
        self.facing_angle = angle;
        self.camera_x = self.x + angle.sin() * 10.;
        self.camera_y = self.y + angle.cos() * 50. + self.height / 4.;
        if self.character.speed > 0. {
            for p in &mut self.presents {
                if p.pickup_at == 0. && (p.x - self.x).hypot(p.y - self.y) < 24. {
                    p.pickup_at = p.time;
                    self.presents_found += 1;
                }
            }
        }
        if self.character.speed / 0.66 > 0.5 {
            let (cx, cy) = (
                (self.x / 96. + 0.5).floor() as i64,
                (self.y / 96. + 0.5).floor() as i64,
            );
            for i in -1..=1 {
                for j in -1..=1 {
                    if let Some((x, y)) = self.tree_for_cell(cx + i, cy + j) {
                        if (x - self.x).hypot(y - self.y) < 12. {
                            self.hit_tree = Some(0.);
                            return;
                        }
                    }
                }
            }
        }
    }
    pub fn score(&self) -> i64 {
        (((self.y - 850.) / 100.).floor() as i64 + self.presents_found as i64 * 25).max(0)
    }
    pub fn step(&mut self, delta: f64, target: Option<f64>) {
        self.time += delta;
        for p in &mut self.presents {
            p.tick(delta);
        }
        let was_hit = self.hit_tree.is_some();
        if let Some(hit) = self.hit_tree {
            if hit < 2. {
                let hit = (hit + delta).min(2.);
                self.hit_tree = Some(hit);
                let (dx, dy) = self.character.tick(delta * (1. - hit / 2.), None);
                self.x += dx * 400.;
                self.y += dy * 600.;
                self.trail_push();
                let rotations = [
                    -delta * self.random.next(),
                    delta * (self.random.next() - 0.5),
                    delta * (self.random.next() - 0.5),
                ];
                self.tumble_steps.push(rotations);
                for (i, v) in rotations.iter().enumerate() {
                    self.tumble[i] += v;
                }
            }
        } else {
            let (dx, dy) = if self.playing {
                self.character.tick(delta, target)
            } else {
                (0., 0.)
            };
            if dx != 0. || dy != 0. {
                self.internal(dx * 400., dy * 600.);
            }
        }
        let skiing = self.character.speed != 0. && self.playing;
        if skiing != self.skiing {
            self.skiing = skiing;
            self.sounds.push(if skiing {
                "elfski_loop_start"
            } else {
                "elfski_loop_stop"
            });
        }
        if !was_hit {
            if self.presents_found != self.last_presents_found {
                self.sounds.push("generic_score");
                self.last_presents_found = self.presents_found;
            }
            self.last_score = self.score();
            self.score_time = (self.time - 1. / 60.).max(0.);
        } else {
            if self.crash_reported.is_none() {
                self.crash_reported = Some(self.time);
                self.sounds.push("elfski_crash");
            }
            if self.playing && self.time - self.crash_reported.unwrap() > 3. {
                self.playing = false;
                self.sounds.push("music_ingame_gameover");
            }
        }
    }
    pub fn advance(&mut self, duration: f64, target: Option<f64>) {
        let mut left = duration;
        while left > 0. {
            let dt = left.min(1. / 60.);
            self.step(dt, target);
            self.render_update(self.width, self.height);
            left -= dt;
        }
    }
    pub fn rotation(&self) -> [f64; 4] {
        let a = (self.facing_angle + PI / 2.) / 2.;
        let mut q = [0., a.sin(), 0., a.cos()];
        if q[1] < 0. {
            for v in &mut q {
                *v = -*v;
            }
        }
        let mul = |a: [f64; 4], b: [f64; 4]| {
            [
                a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
                a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
                a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
                a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
            ]
        };
        for r in &self.tumble_steps {
            for axis in 0..3 {
                let mut b = [0., 0., 0., (r[axis] / 2.).cos()];
                b[axis] = (r[axis] / 2.).sin();
                q = mul(q, b);
            }
        }
        q
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"rotation":self.rotation(),"character":self.character,"x":self.x,"y":self.y,"camera":[self.camera_x,self.camera_y],"score":self.score(),"hit":self.hit_tree,"presents_found":self.presents_found,"trees":self.trees,"presents":self.presents,"trail":self.trail})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_original_javascript_world_physics_and_animation() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../data/ski-reference.json")).unwrap();
        fn compare(a: &serde_json::Value, b: &serde_json::Value, path: &str) {
            match (a, b) {
                (serde_json::Value::Number(a), serde_json::Value::Number(b)) => assert!(
                    (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-7,
                    "{path}: {a} != {b}"
                ),
                (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                    assert_eq!(a.len(), b.len(), "{path}");
                    for (i, (a, b)) in a.iter().zip(b).enumerate() {
                        compare(a, b, &format!("{path}/{i}"));
                    }
                }
                (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                    for (k, b) in b {
                        compare(&a[k], b, &format!("{path}/{k}"));
                    }
                }
                _ => assert_eq!(a, b, "{path}"),
            }
        }
        let expected: serde_json::Value =
            serde_json::from_str(include_str!("../data/ski-pickup-reference.json")).unwrap();
        let mut pickup = Ski::new("test1234", 1520., 880.);
        pickup.render_update(1520., 880.);
        pickup.y = 2200.;
        pickup.internal(0., 0.);
        pickup.render_update(1520., 880.);
        let p = &pickup.presents[0];
        pickup.x = p.x;
        pickup.y = p.y;
        pickup.character.speed = 0.1;
        pickup.internal(0., 0.);
        for (i, state) in expected.as_array().unwrap().iter().enumerate() {
            if i > 0 {
                for _ in 0..if i < 3 { 10 } else { 20 } {
                    pickup.step(1. / 60., None);
                    pickup.render_update(1520., 880.);
                }
            }
            let mut actual = pickup.snapshot();
            for t in actual["trees"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .filter(|t| !t.is_null())
            {
                for k in ["x", "y"] {
                    t[k] = serde_json::json!(t[k].as_f64().unwrap() as f32 as f64);
                }
            }
            compare(&actual, state, &format!("pickup/{i}"));
        }
        for case in cases.as_array().unwrap() {
            let mut game = Ski::new("test1234", 1520., 880.);
            game.render_update(1520., 880.);
            for step in case["steps"].as_array().unwrap() {
                game.advance(step["duration"].as_f64().unwrap(), step["target"].as_f64());
                let mut actual = game.snapshot();
                // Original Points stores positions in Float32Array GPU attributes.
                for tree in actual["trees"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .filter(|t| !t.is_null())
                {
                    for key in ["x", "y"] {
                        tree[key] = serde_json::json!(tree[key].as_f64().unwrap() as f32 as f64);
                    }
                }
                compare(&actual, &step["state"], case["name"].as_str().unwrap());
            }
        }
    }
}
