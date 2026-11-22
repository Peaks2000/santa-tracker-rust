//! Original TrailEffect and SnowsplatEffect fixed buffers and update ordering.
use crate::snowball::V;

#[derive(Clone, Default, Debug)]
pub struct Particle {
    pub pos: V,
    pub z: f64,
    pub direction: V,
    pub born: f64,
    pub size: f64,
    pub random: f64,
    pub trail: bool,
    pub color: [f32; 3],
}
#[derive(Clone, Debug)]
pub struct TrailSample {
    pub position: [f64; 3],
    pub size: f64,
    pub color: u32,
    pub show: bool,
}
#[derive(Clone, Debug)]
pub struct SplatSample {
    pub position: [f64; 3],
    pub direction: [f64; 2],
    pub size: f64,
    pub quantity: usize,
}

pub struct Effects {
    pub time: f64,
    pub trails: Vec<Particle>,
    pub splats: Vec<Particle>,
    pub next_trail: usize,
    pub next_splat: usize,
    pub registered: Vec<usize>,
    pending: Vec<usize>,
}
fn float(value: f64) -> f64 {
    value as f32 as f64
}
impl Effects {
    pub fn new(mut random: impl FnMut() -> f64) -> Self {
        Self {
            time: 0.,
            trails: vec![
                Particle {
                    trail: true,
                    ..Particle::default()
                };
                500
            ],
            splats: (0..2000)
                .map(|_| Particle {
                    random: float(random()),
                    ..Particle::default()
                })
                .collect(),
            next_trail: 0,
            next_splat: 0,
            registered: vec![],
            pending: vec![],
        }
    }
    pub fn add(&mut self, id: usize) {
        self.registered.push(id);
    }
    pub fn remove(&mut self, id: usize) {
        // JS splice(indexOf(id), 1) removes the LAST entry when id is absent.
        if let Some(i) = self
            .registered
            .iter()
            .position(|&x| x == id)
            .or_else(|| self.registered.len().checked_sub(1))
        {
            self.registered.remove(i);
        }
    }
    /// Key identifies a Splat component, not its entity. Resolve at flush so
    /// queued duplicate references observe the component's latest values.
    pub fn show_for(&mut self, component: usize) {
        self.pending.push(component);
    }
    /// Source effect teardown is inherited no-op: attributes/cursors survive.
    pub fn teardown(&mut self) {}

    pub fn update(
        &mut self,
        time: f64,
        trail: impl Fn(usize) -> Option<TrailSample>,
        splat: impl Fn(usize) -> Option<SplatSample>,
    ) {
        self.time = time;
        for &id in &self.registered {
            let Some(sample) = trail(id).filter(|s| s.show) else {
                continue;
            };
            let p = &mut self.trails[self.next_trail];
            p.pos = V::new(float(sample.position[0]), float(sample.position[1]));
            p.z = float(sample.position[2] + 1.);
            p.size = float(sample.size);
            p.born = float(time);
            p.color = [16, 8, 0].map(|shift| ((sample.color >> shift) & 255) as f32 / 255.);
            self.next_trail = (self.next_trail + 1) % self.trails.len();
        }
        while let Some(id) = self.pending.pop() {
            let Some(sample) = splat(id) else { continue };
            let remaining = self.splats.len() - self.next_splat;
            let start = if remaining < sample.quantity {
                0
            } else {
                self.next_splat
            };
            // Typed-array out-of-bounds writes are ignored in the original.
            for p in self.splats.iter_mut().skip(start).take(sample.quantity) {
                p.pos = V::new(float(sample.position[0]), float(sample.position[1]));
                p.z = float(sample.position[2]);
                p.direction = V::new(float(sample.direction[0]), float(sample.direction[1]));
                p.born = float(time);
                p.size = float(sample.size);
                // The constructor's random attribute is never regenerated.
            }
            // Preserve source wrap quirk: advance OLD cursor, not start+count.
            self.next_splat = (self.next_splat + sample.quantity) % self.splats.len();
        }
    }
    pub fn write_particles(&self, time: f64, out: &mut Vec<Particle>) {
        out.clear();
        out.extend(
            self.trails
                .iter()
                .chain(&self.splats)
                .filter(|p| {
                    p.size != 0.
                        && time as f32 - (p.born as f32) < if p.trail { 700. } else { 300. }
                })
                .cloned(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;
    fn vector<const N: usize>(v: &Value) -> [f64; N] {
        std::array::from_fn(|i| v[i].as_f64().unwrap())
    }
    fn trail_row(p: &Particle) -> [f64; 8] {
        [
            p.pos.x,
            p.pos.y,
            p.z,
            p.color[0] as f64,
            p.color[1] as f64,
            p.color[2] as f64,
            p.size,
            p.born,
        ]
    }
    fn splat_row(p: &Particle) -> [f64; 8] {
        [
            p.pos.x,
            p.pos.y,
            p.z,
            p.direction.x,
            p.direction.y,
            p.size,
            p.born,
            p.random,
        ]
    }
    #[test]
    fn complete_fixed_buffers_match_original_effect_updates() {
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-effects-reference.json")).unwrap();
        let mut samples = data["random"].as_array().unwrap().iter();
        let mut effects = Effects::new(|| samples.next().unwrap().as_f64().unwrap());
        assert!(samples.next().is_none());
        let mut trails = HashMap::<usize, TrailSample>::new();
        let mut splats = HashMap::<usize, SplatSample>::new();
        let mut expected_trails = vec![[0.; 8]; 500];
        let mut expected_splats: Vec<[f64; 8]> = data["random"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                let mut row = [0.; 8];
                row[7] = v.as_f64().unwrap();
                row
            })
            .collect();
        for row in data["rows"].as_array().unwrap() {
            let input = &row["input"];
            for op in input["ops"].as_array().unwrap() {
                let id = op["id"].as_u64().unwrap_or(0) as usize;
                match op["op"].as_str().unwrap() {
                    "trail" => {
                        trails.insert(
                            id,
                            TrailSample {
                                position: vector(&op["position"]),
                                size: op["size"].as_f64().unwrap(),
                                color: op["color"].as_u64().unwrap() as u32,
                                show: op["show"].as_bool().unwrap(),
                            },
                        );
                        effects.add(id);
                    }
                    "add" => effects.add(id),
                    "set" => trails.get_mut(&id).unwrap().show = op["show"].as_bool().unwrap(),
                    "null-trail" => {
                        trails.remove(&id);
                    }
                    "remove" => effects.remove(id),
                    "bulk" => {
                        let start = op["start"].as_u64().unwrap() as usize;
                        for i in 0..op["count"].as_u64().unwrap() as usize {
                            trails.insert(
                                start + i,
                                TrailSample {
                                    position: [i as f64 + 0.125, -(i as f64) - 0.25, i as f64 / 3.],
                                    size: if i % 2 == 1 { 10. } else { 6. },
                                    color: 0xaaccff,
                                    show: true,
                                },
                            );
                            effects.add(start + i);
                        }
                    }
                    "clear-trails" => {
                        for id in effects.registered.clone() {
                            effects.remove(id)
                        }
                    }
                    "splat" => {
                        splats.insert(
                            id,
                            SplatSample {
                                position: vector(&op["position"]),
                                direction: vector(&op["direction"]),
                                size: op["size"].as_f64().unwrap(),
                                quantity: op["quantity"].as_u64().unwrap() as usize,
                            },
                        );
                    }
                    "queue" => effects.show_for(id),
                    "queue-null" => {} // showFor skips absent Splat components.
                    "mutate" => splats.get_mut(&id).unwrap().position = vector(&op["position"]),
                    "teardown" => effects.teardown(),
                    other => panic!("unknown operation {other}"),
                }
            }
            effects.update(
                input["at"].as_f64().unwrap(),
                |id| trails.get(&id).cloned(),
                |id| splats.get(&id).cloned(),
            );
            for (key, expected) in [
                ("trailChanges", &mut expected_trails),
                ("splatChanges", &mut expected_splats),
            ] {
                for change in row[key].as_array().unwrap() {
                    expected[change[0].as_u64().unwrap() as usize] = vector(&change[1]);
                }
            }
            assert_eq!(effects.time, row["trailTime"].as_f64().unwrap(), "{input}");
            assert_eq!(effects.time, row["splatTime"].as_f64().unwrap(), "{input}");
            assert_eq!(
                effects.next_trail,
                row["nextTrail"].as_u64().unwrap() as usize,
                "{input}"
            );
            assert_eq!(
                effects.next_splat,
                row["nextSplat"].as_u64().unwrap() as usize,
                "{input}"
            );
            assert_eq!(
                serde_json::json!(effects.registered),
                row["registered"],
                "{input}"
            );
            for (i, p) in effects.trails.iter().enumerate() {
                assert_eq!(trail_row(p), expected_trails[i], "trail {i} {input}");
            }
            for (i, p) in effects.splats.iter().enumerate() {
                assert_eq!(splat_row(p), expected_splats[i], "impact {i} {input}");
            }
        }
    }
}
