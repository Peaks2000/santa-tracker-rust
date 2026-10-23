//! Native translation of maker-control, maker-elf and index.js motion sequencing.
use crate::random::Random;
use serde_json::Value;
pub const CATEGORIES: [&str; 8] = [
    "body",
    "suit",
    "hair",
    "glasses",
    "ears",
    "hats",
    "accessories",
    "backgrounds",
];
pub const SAVED: [&str; 7] = [
    "body",
    "hair",
    "glasses",
    "ears",
    "hats",
    "accessories",
    "backgrounds",
];
pub const PROPS: [&str; 7] = [
    "skinTone",
    "hairColor",
    "suitColor",
    "glassesColor",
    "earsColor",
    "hatsColor",
    "accessoriesColor",
];
pub const MODES: [&str; 7] = [
    "skin", "hair", "color", "glasses", "color", "color", "color",
];
#[derive(Clone, Copy)]
struct Arm {
    target: f64,
    range: f64,
    rate: f64,
    prev_at: f64,
    prev: f64,
}
impl Arm {
    fn degrees(self, t: f64) -> f64 {
        let r = ((t - self.prev_at) * 1.5).min(1.).sin_angle();
        r * (self.target + (t * self.rate).sin() * self.range) + (1. - r) * self.prev
    }
}
trait QuarterSin {
    fn sin_angle(self) -> f64;
}
impl QuarterSin for f64 {
    fn sin_angle(self) -> f64 {
        (self * std::f64::consts::FRAC_PI_2).sin()
    }
}
pub struct ElfMaker {
    pub art: Value,
    pub choices: [usize; 7],
    pub colors: [String; 7],
    pub category: usize,
    pub time: f64,
    pub sounds: Vec<&'static str>,
    random: Random,
    arms: [Arm; 2],
    steps: Vec<(f64, Option<(f64, f64, f64)>, Option<(f64, f64, f64)>)>,
    step_at: f64,
    next_move: f64,
}
impl ElfMaker {
    pub fn new(seed: &str) -> Self {
        let mut g = Self {
            art: serde_json::from_str(include_str!("../data/elfmaker-art.json")).unwrap(),
            choices: [0; 7],
            colors: Default::default(),
            category: 0,
            time: 0.,
            sounds: vec![],
            random: Random::seeded(seed),
            arms: [Arm {
                target: 130.,
                range: 0.,
                rate: 1.,
                prev_at: 0.,
                prev: 130.,
            }; 2],
            steps: vec![],
            step_at: 0.,
            next_move: 0.,
        };
        g.random_colors();
        g.defaults();
        let left = g.random.next() < 0.5;
        g.start_move(if left { 0 } else { 2 });
        g.next_move = 10. + 10. * g.random.next();
        g
    }
    fn random_colors(&mut self) {
        for (i, mode) in MODES.iter().enumerate() {
            let o = self.art["options"][mode].as_array().unwrap();
            self.colors[i] = o[(self.random.next() * o.len() as f64) as usize]
                .as_str()
                .unwrap()
                .into();
        }
    }
    pub fn randomize(&mut self) {
        for (i, k) in SAVED.iter().enumerate() {
            let n = self.art["categories"][k].as_array().unwrap().len();
            let c = self.random.next() * (n + 1) as f64;
            self.choices[i] = if c >= n as f64 { 0 } else { c as usize };
        }
        self.random_colors();
        self.sounds
            .extend(["elfmaker_switch_item", "elfmaker_switch_item"]);
    }
    pub fn choose(&mut self, i: usize) {
        if let Some(k) = SAVED.iter().position(|s| *s == CATEGORIES[self.category]) {
            self.choices[k] = i;
            self.sounds.push("elfmaker_switch_item");
        }
    }
    pub fn color_property(&self) -> Option<usize> {
        match self.category {
            0 => Some(0),
            1 => Some(2),
            2 => Some(1),
            3 => Some(3),
            4 => Some(4),
            5 => Some(5),
            6 => Some(6),
            _ => None,
        }
    }
    pub fn select_category(&mut self, i: usize) {
        if self.category != i {
            self.category = i;
            self.sounds.push("elfmaker_switch_type");
        }
    }
    pub fn choose_color(&mut self, i: usize) {
        if let Some(p) = self.color_property() {
            let s = self.art["options"][MODES[p]][i].as_str().unwrap();
            self.colors[p] = s.into();
            self.sounds.push("elfmaker_switch_color");
        }
    }
    fn arm(&mut self, i: usize, c: (f64, f64, f64)) {
        let prev = self.arms[i].degrees(self.time);
        self.arms[i] = Arm {
            target: c.0,
            range: c.1,
            rate: c.2,
            prev_at: self.time,
            prev,
        };
    }
    fn defaults(&mut self) {
        self.arm(1, (140., 5., 1.25));
        self.arm(0, (140., 5., 1.5));
    }
    pub fn poke(&mut self) {
        if self.steps.is_empty() {
            let n = (self.random.next() * 8.) as usize;
            self.start_move(n);
        }
        self.next_move = self.time + 10. + 10. * self.random.next();
    }
    pub fn start_move(&mut self, n: usize) {
        let a = |d, l, r| (d, l, r);
        let num = |x| Some((x, 10., 1.));
        let wave = |x, r, rate| Some((x, r, rate));
        self.steps = match n {
            0 => vec![a(std::f64::consts::PI, wave(60., 20., 4.), None)],
            1 => vec![a(std::f64::consts::PI, wave(30., 2., 10.), None)],
            2 => vec![a(std::f64::consts::PI, None, wave(60., 20., 4.))],
            3 => vec![a(std::f64::consts::PI, None, wave(30., 2., 10.))],
            4 => [
                (30., 140.),
                (140., 30.),
                (80., 180.),
                (180., 80.),
                (40., 40.),
            ]
            .into_iter()
            .map(|(l, r)| a(0.8, num(l), num(r)))
            .collect(),
            5 => vec![
                a(1.5, wave(70., 10., 6.), wave(100., 10., 6.)),
                a(1.5, wave(100., 10., 6.), wave(70., 10., 6.)),
            ],
            6 => [(180., 100.), (100., 180.), (180., 100.), (100., 180.)]
                .into_iter()
                .map(|(l, r)| a(1.5, wave(l, 10., 6.), wave(r, 10., 6.)))
                .collect(),
            _ => vec![a(2.2, num(90.), num(90.)), a(0.5, num(70.), num(70.))],
        };
        self.apply_step();
    }
    fn apply_step(&mut self) {
        if let Some(&(d, l, r)) = self.steps.first() {
            // applyStep(false) updates _rightArm; render's leftArm uses _leftArm.
            if let Some(c) = l {
                self.arm(1, c)
            }
            if let Some(c) = r {
                self.arm(0, c)
            }
            self.step_at = self.time + d;
        }
    }
    pub fn advance(&mut self, dt: f64) {
        self.time += dt;
        if !self.steps.is_empty() && self.time >= self.step_at {
            self.steps.remove(0);
            if self.steps.is_empty() {
                self.defaults()
            } else {
                self.apply_step()
            }
        }
        if self.time >= self.next_move {
            self.poke()
        }
    }
    pub fn style(&self) -> String {
        let mut s = String::new();
        for (n, p, prop) in [
            ("suit", 2, "fill"),
            ("hats", 5, "fill"),
            ("glasses", 3, "fill"),
            ("ears", 4, "fill"),
            ("limb", 2, "stroke"),
            ("skin", 0, "fill"),
            ("hair", 1, "fill"),
            ("accessories", 6, "fill"),
        ] {
            for (i, c) in self.art["colors"][&self.colors[p]]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                s += &format!(
                    ".{n}{}{{{prop}:{}}}",
                    if i == 0 { String::new() } else { i.to_string() },
                    c.as_str().unwrap()
                );
            }
        }
        s += &format!(
            ".glasses-stroke{{fill:none;stroke:{};stroke-width:5px}}",
            self.art["colors"][&self.colors[3]][0].as_str().unwrap()
        );
        s
    }
    pub fn fragment(&self, choices: [usize; 7], setup: [f64; 4]) -> String {
        let b = &self.art["bodyTypes"][choices[0].min(6)];
        let limb = 18. + b["weight"].as_f64().unwrap() * 8.;
        let leg = b["height"].as_f64().unwrap() * 96. + 32.;
        let arm = b["height"].as_f64().unwrap() * 64. + 96.;
        let scale = limb / 20.;
        let part = |k: &str| {
            let i = SAVED.iter().position(|s| *s == k).unwrap();
            self.art["categories"][k][choices[i]]
                .as_str()
                .unwrap_or("")
                .to_owned()
        };
        fill(
            self.art["elfTemplate"].as_str().unwrap(),
            &[
                limb.to_string(),
                scale_at(scale, scale, 130., 428.65),
                (-(leg + 100.) / scale).to_string(),
                (-(leg + 100.) / scale).to_string(),
                (80. - leg).to_string(),
                setup[1].to_string(),
                part("hats"),
                scale_at(scale.sqrt(), scale.powf(0.25), 0., 202.7),
                self.art["body"].as_str().unwrap().into(),
                self.arm_svg(setup[2], setup[0], arm),
                self.arm_svg(setup[3], setup[0], arm),
                self.art["head"].as_str().unwrap().into(),
                part("hair"),
                part("glasses"),
                part("accessories"),
                part("ears"),
            ],
        )
    }
    fn arm_svg(&self, angle: f64, shrug: f64, mut len: f64) -> String {
        let r = angle.to_radians();
        len *= (r.sin() + 1.) / 2.;
        let x = r.sin() * len;
        let y = r.cos() * len;
        let bx = shrug * -len / 3.;
        let hx = -len * 0.75;
        let angle_at = (y.atan2(-x - hx)).to_degrees() - 90.;
        fill(
            self.art["armTemplate"].as_str().unwrap(),
            &[
                bx.to_string(),
                "-4".into(),
                hx.to_string(),
                "0".into(),
                (-x).to_string(),
                y.to_string(),
                (len / 2.).to_string(),
                (-x).to_string(),
                y.to_string(),
                angle_at.to_string(),
            ],
        )
    }
    pub fn svg(&self, viewbox: &str, preview: Option<[usize; 7]>) -> String {
        let setup = if preview.is_some() {
            [0., 0., 130., 60.]
        } else {
            [
                (self.time.cos() + 1.) / 2.,
                self.time.cos() * 5.,
                self.arms[0].degrees(self.time),
                self.arms[1].degrees(self.time),
            ]
        };
        let mut frag = self.fragment(preview.unwrap_or(self.choices), setup);
        let cycle = (self.time / 5.234) % 2.;
        let phase = if cycle > 1. { 2. - cycle } else { cycle };
        let blink = if phase < 0.98 {
            1.
        } else {
            1. - crate::ui::Ease::Default.at((phase - 0.98) / 0.02)
        };
        frag = frag.replace(
            "class=\"eyes blink\"",
            &format!("class=\"eyes\" transform=\"scale(1,{blink})\""),
        );
        format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{viewbox}\">{}<style>{}{}</style>{frag}</svg>",self.art["baseSvgDefs"].as_str().unwrap(),self.art["baseSvgStyle"].as_str().unwrap(),self.style())
    }
    pub fn restore(&mut self, encoded: &str) -> Result<(), String> {
        let decoded = decode64(encoded)?;
        let decoded = String::from_utf8(decoded).map_err(|_| "Invalid elf state")?;
        self.choices = [0; 7];
        self.random_colors();
        let mut colors = None;
        for (index, value) in decoded.split(',').enumerate() {
            if value == "|" {
                colors = Some(index + 1);
            } else if let Some(start) = colors {
                if let Some(c) = self.colors.get_mut(index - start) {
                    if self.art["colors"][value].is_array() {
                        *c = value.into();
                    }
                }
            } else if index < 7 {
                let n = value.parse::<usize>().unwrap_or(0);
                let len = self.art["categories"][SAVED[index]]
                    .as_array()
                    .unwrap()
                    .len();
                self.choices[index] = if n < len { n } else { 0 };
            }
        }
        Ok(())
    }
    pub fn save(&self) -> String {
        let mut v = self
            .choices
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        v.push("|".into());
        v.extend(self.colors.iter().cloned());
        base64(v.join(",").as_bytes())
    }
}
fn scale_at(x: f64, y: f64, cx: f64, cy: f64) -> String {
    format!("matrix({x}, 0, 0, {y}, {}, {})", cx - x * cx, cy - y * cy)
}
fn fill(t: &str, v: &[String]) -> String {
    let mut out = t.to_owned();
    for (i, s) in v.iter().enumerate() {
        out = out.replace(&format!("@{i}@"), s)
    }
    out
}
pub fn base64(bytes: &[u8]) -> String {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::new();
    for c in bytes.chunks(3) {
        let n = ((c[0] as u32) << 16)
            | ((c.get(1).copied().unwrap_or(0) as u32) << 8)
            | c.get(2).copied().unwrap_or(0) as u32;
        for i in 0..4 {
            o.push(if i > c.len() {
                '='
            } else {
                A[((n >> (18 - i * 6)) & 63) as usize] as char
            })
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn control_matches_original_randomization_and_serialization() {
        let cases: Value =
            serde_json::from_str(include_str!("../data/elfmaker-control-reference.json")).unwrap();
        let mut g = ElfMaker::new("test1234");
        for (i, v) in cases.as_array().unwrap().iter().enumerate() {
            if i > 0 {
                g.randomize();
            }
            assert_eq!(serde_json::json!(g.choices), v["choices"]);
            assert_eq!(serde_json::json!(g.colors), v["colors"]);
            assert_eq!(g.save(), v["serialized"].as_str().unwrap());
        }
    }
    #[test]
    fn dynamic_geometry_matches_source_svg_pixels() {
        let g = ElfMaker::new("test1234");
        let cases: Value =
            serde_json::from_str(include_str!("../data/elfmaker-draw-reference.json")).unwrap();
        for c in cases.as_array().unwrap() {
            let mut choices = [0; 7];
            for (i, k) in SAVED.iter().enumerate() {
                choices[i] = c["config"][k].as_u64().unwrap() as usize;
            }
            let st = &c["setup"];
            let setup = [
                st["shrugFactor"].as_f64().unwrap(),
                st["bodyDegrees"].as_f64().unwrap(),
                st["leftArmDegrees"].as_f64().unwrap(),
                st["rightArmDegrees"].as_f64().unwrap(),
            ];
            let wrap = |frag: &str| {
                format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"-40 -100 400 560\">{}<style>{}{}</style>{frag}</svg>",g.art["baseSvgDefs"].as_str().unwrap(),g.art["baseSvgStyle"].as_str().unwrap(),g.style())
            };
            let render = |svg: String| {
                let tree = resvg::usvg::Tree::from_str(&svg, &Default::default()).unwrap();
                let mut p = resvg::tiny_skia::Pixmap::new(400, 560).unwrap();
                resvg::render(&tree, Default::default(), &mut p.as_mut());
                p
            };
            let a = render(wrap(&g.fragment(choices, setup)));
            let b = render(wrap(c["svg"].as_str().unwrap()));
            let error: u64 = a
                .data()
                .iter()
                .zip(b.data())
                .map(|(a, b)| a.abs_diff(*b) as u64)
                .sum();
            assert!(error < 100, "SVG raster mismatch: {error}");
        }
    }
}

pub fn decode64(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut bits = 0u32;
    let mut count = 0;
    for b in s.bytes().filter(|b| !b.is_ascii_whitespace()) {
        let n = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => return Err("Invalid base64 elf state".into()),
        };
        bits = (bits << 6) | n as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8)
        }
    }
    Ok(out)
}
