//! Original quad-tree ordering and containment (including its circle-Y quirk).
use crate::snowball::V;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    Player(usize),
    Shot(usize),
    Drop(usize),
}
#[derive(Clone, Copy)]
pub struct Bounds {
    pub pos: V,
    pub half: V,
    pub circle: bool,
}
impl Bounds {
    pub fn rect(pos: V, w: f64, h: f64) -> Self {
        Self {
            pos,
            half: V::new(w / 2., h / 2.),
            circle: false,
        }
    }
    pub fn circle(pos: V, r: f64) -> Self {
        Self {
            pos,
            half: V::new(r, r),
            circle: true,
        }
    }
    pub fn contains(self, b: Self) -> bool {
        let l = self.pos.x - self.half.x;
        let r = self.pos.x + self.half.x;
        let bottom = self.pos.y - self.half.y;
        let top = self.pos.y + self.half.y;
        if b.circle {
            !(b.pos.x - b.half.x < l
                || b.pos.x + b.half.x > r
                || b.pos.y - b.half.x > top
                || b.pos.y + b.half.x < bottom)
        } else {
            b.pos.x - b.half.x >= l
                && b.pos.x + b.half.x <= r
                && b.pos.y - b.half.y >= bottom
                && b.pos.y + b.half.y <= top
        }
    }
}
pub struct Quad {
    bounds: Bounds,
    depth: usize,
    objects: Vec<(Id, Bounds)>,
    nodes: Vec<Quad>,
}
impl Quad {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            depth: 0,
            objects: vec![],
            nodes: vec![],
        }
    }
    fn quadrant(&self, b: Bounds) -> Option<usize> {
        self.nodes.iter().position(|n| n.bounds.contains(b))
    }
    fn insert(&mut self, id: Id, b: Bounds) {
        if let Some(i) = self.quadrant(b) {
            self.nodes[i].add(id, b)
        } else {
            self.objects.push((id, b))
        }
    }
    pub fn add(&mut self, id: Id, b: Bounds) {
        self.insert(id, b);
        if self.objects.len() > 10 && self.depth < 6 {
            if self.nodes.is_empty() {
                for x in [-1., 1.] {
                    for y in [-1., 1.] {
                        self.nodes.push(Self {
                            bounds: Bounds::rect(
                                self.bounds.pos.add(V::new(
                                    x * self.bounds.half.x / 2.,
                                    y * self.bounds.half.y / 2.,
                                )),
                                self.bounds.half.x,
                                self.bounds.half.y,
                            ),
                            depth: self.depth + 1,
                            objects: vec![],
                            nodes: vec![],
                        })
                    }
                }
            }
            let mut i = 0;
            while i < self.objects.len() {
                let n = self.objects.len();
                let (id, b) = self.objects.remove(0);
                self.insert(id, b);
                if self.objects.len() == n {
                    i += 1
                }
            }
        }
    }
    pub fn near(&self, b: Bounds) -> Vec<Id> {
        let mut out = self
            .quadrant(b)
            .map(|i| self.nodes[i].near(b))
            .unwrap_or_default();
        out.extend(self.objects.iter().map(|(id, _)| *id));
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_quad_tree_candidate_order() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-reference.json")).unwrap();
        let bounds = |i: usize| {
            let pos = V::new(
                ((i * 93) % 900) as f64 - 450.,
                ((i * 137) % 700) as f64 - 350.,
            );
            if i % 3 == 0 {
                Bounds::circle(pos, 10.)
            } else {
                Bounds::rect(pos, 15., 45.)
            }
        };
        let mut q = Quad::new(Bounds::rect(V::default(), 1000., 800.));
        for i in 0..40 {
            q.add(Id::Player(i), bounds(i))
        }
        for row in data["quadQueries"].as_array().unwrap() {
            let i = row["id"].as_u64().unwrap() as usize;
            let actual: Vec<_> = q
                .near(bounds(i))
                .iter()
                .map(|id| match id {
                    Id::Player(i) => *i,
                    _ => unreachable!(),
                })
                .collect();
            assert_eq!(serde_json::json!(actual), row["near"], "query {i}")
        }
    }
}
