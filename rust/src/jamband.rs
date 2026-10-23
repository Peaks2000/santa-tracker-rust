//! Original instrument placement, musical bar scheduling and band serialization.
use crate::random::Random;
pub const NAMES: [&str; 10] = [
    "Bass", "Drums", "Bell", "Boombox", "Guitar", "Rap", "Sax", "Synth", "Tamb", "Xylo",
];
pub const DRAWER: [usize; 10] = [5, 4, 1, 0, 7, 2, 6, 3, 8, 9];
pub const PATTERN: [i32; 6] = [0, 1, 0, 1, 0, 1];
pub const VOLUME: [f32; 6] = [0.6, 0.8, 1., 0.8, 0.6, 1.];
#[derive(Debug, Clone)]
pub enum SoundEvent {
    Track {
        instrument: usize,
        pattern: i32,
        volume: f32,
    },
    Preview(usize),
}
pub struct Jamband {
    pub stages: [Option<usize>; 6],
    pub playing: [Option<i32>; 10],
    pub waiting: [bool; 10],
    pub started_at: [f64; 10],
    pub waiting_at: [f64; 10],
    pending: [Option<(f64, i32, f32)>; 10],
    pub time: f64,
    pub count: usize,
    pub sounds: Vec<SoundEvent>,
    random: Random,
}
impl Jamband {
    pub fn new(seed: &str) -> Self {
        Self {
            stages: [None; 6],
            playing: [None; 10],
            waiting: [false; 10],
            started_at: [0.; 10],
            waiting_at: [0.; 10],
            pending: [None; 10],
            time: 0.,
            count: 0,
            sounds: vec![],
            random: Random::seeded(seed),
        }
    }
    pub fn location(&self, i: usize) -> Option<usize> {
        self.stages.iter().position(|s| *s == Some(i))
    }
    pub fn next_bar(t: f64) -> f64 {
        (((t / 0.5).floor().max(0.) / 4.).trunc() + 1.) * 2.
    }
    fn queue(&mut self, i: usize, s: usize) {
        self.pending[i] = Some((Self::next_bar(self.time), PATTERN[s], VOLUME[s]));
        if !self.waiting[i] {
            self.waiting_at[i] = self.time;
        }
        self.waiting[i] = true;
    }
    fn returned(&mut self, i: usize) {
        self.pending[i] = None;
        self.playing[i] = None;
        self.waiting[i] = false;
        self.sounds.push(SoundEvent::Track {
            instrument: i,
            pattern: -1,
            volume: 1.,
        });
        self.count = self.stages.iter().flatten().count();
    }
    pub fn preview(&mut self, i: usize) {
        if self.location(i).is_none() {
            self.sounds.push(SoundEvent::Preview(i));
        }
    }
    pub fn drop(&mut self, i: usize, target: Option<usize>) {
        if i >= 10 {
            return;
        }
        let old = self.location(i);
        if let Some(s) = target.filter(|s| *s < 6) {
            if let Some(other) = self.stages[s].filter(|o| *o != i) {
                if let Some(old) = old {
                    self.stages[old] = Some(other);
                    self.queue(other, old);
                } else {
                    self.stages[s] = None;
                    self.returned(other);
                }
            }
            if let Some(old) = old {
                if old != s && self.stages[old] == Some(i) {
                    self.stages[old] = None;
                }
            }
            self.stages[s] = Some(i);
            self.queue(i, s);
        } else {
            if let Some(s) = old {
                self.stages[s] = None;
            }
            self.returned(i);
        }
    }
    pub fn advance(&mut self, dt: f64) {
        self.time += dt;
        for i in 0..10 {
            if let Some((at, p, v)) = self.pending[i] {
                if self.time + 1e-12 >= at {
                    self.pending[i] = None;
                    self.sounds.push(SoundEvent::Track {
                        instrument: i,
                        pattern: p,
                        volume: v,
                    });
                    if self.location(i).is_some() {
                        if self.playing[i].is_none() {
                            self.started_at[i] = self.time;
                        }
                        self.playing[i] = Some(p);
                    }
                    self.count = self.stages.iter().flatten().count();
                }
            }
        }
    }
    pub fn reset(&mut self) {
        for i in 0..10 {
            if let Some(s) = self.location(i) {
                self.stages[s] = None;
            }
            self.returned(i);
        }
    }
    pub fn randomize(&mut self) {
        self.reset();
        let mut v: [usize; 10] = std::array::from_fn(|i| i);
        for i in (1..10).rev() {
            let j = (self.random.next() * (i + 1) as f64).floor() as usize;
            v.swap(i, j);
        }
        for (s, i) in v.into_iter().take(6).enumerate() {
            self.drop(i, Some(s));
        }
    }
    pub fn save(&self) -> String {
        if self.stages.iter().all(Option::is_none) {
            String::new()
        } else {
            self.stages
                .iter()
                .map(|s| s.map_or(-1, |i| i as i32).to_string())
                .collect::<Vec<_>>()
                .join(",")
        }
    }
    pub fn restore(&mut self, s: &str) {
        self.reset();
        for (stage, v) in s.split(',').take(6).enumerate() {
            if let Ok(i) = v.trim().parse::<usize>() {
                if i < 10 {
                    self.drop(i, Some(stage));
                }
            }
        }
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"band":self.save(),"count":self.count,"playing":self.playing,"waiting":self.waiting})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_original_placement_and_bar_callbacks() {
        let steps: serde_json::Value =
            serde_json::from_str(include_str!("../data/jamband-reference.json")).unwrap();
        let mut g = Jamband::new("test1234");
        for row in steps.as_array().unwrap() {
            let a = &row["action"];
            match a[0].as_str().unwrap() {
                "initial" => {}
                "drop" => g.drop(
                    a[1].as_u64().unwrap() as usize,
                    a[2].as_u64().map(|x| x as usize),
                ),
                "advance" => g.advance(a[1].as_f64().unwrap()),
                "restore" => g.restore(a[1].as_str().unwrap()),
                "randomize" => g.randomize(),
                "reset" => g.reset(),
                _ => unreachable!(),
            };
            assert_eq!(g.snapshot(), row["state"], "{a}");
        }
    }
}
