//! Original quad-tree ordering and containment (including its circle-Y quirk).
use crate::snowball::V;
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    Player(usize),
    Shot(usize),
    Drop(usize),
    Tree(usize),
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
            !(b.pos.x - b.half.x < l
                || b.pos.x + b.half.x > r
                || b.pos.y - b.half.y < bottom
                || b.pos.y + b.half.y > top)
        }
    }
    pub fn intersects(self, b: Self) -> bool {
        match (self.circle, b.circle) {
            (true, true) => self.pos.sub(b.pos).len() < self.half.x + b.half.x,
            (true, false) => {
                crate::snowball::circle_rect(self.pos, self.half.x, b.pos, b.half.x, b.half.y)
            }
            (false, true) => b.intersects(self),
            (false, false) => {
                !(b.pos.x - b.half.x > self.pos.x + self.half.x
                    || b.pos.x + b.half.x < self.pos.x - self.half.x
                    || b.pos.y + b.half.y < self.pos.y - self.half.y
                    || b.pos.y - b.half.y > self.pos.y + self.half.y)
            }
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
    pub fn clear_dynamic(&mut self) {
        self.objects.retain(|(id, _)| matches!(id, Id::Tree(_)));
        for node in &mut self.nodes {
            node.clear_dynamic();
        }
        // Source tests this.nodes.length rather than each child's contents:
        // once split, even empty child nodes survive clearDynamic().
    }
    #[cfg(test)]
    fn node_count(&self) -> usize {
        1 + self.nodes.iter().map(Quad::node_count).sum::<usize>()
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

/// The active scene's handlers change actor state, but do not mutate collider
/// registration or XY bounds during notification. Keep the source Set order and
/// directed notification order; dispatch its handlers after this measurement.
pub struct Collision {
    pub order: Vec<Id>,
    pub quad: Quad,
}
impl Collision {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            order: vec![],
            quad: Quad::new(bounds),
        }
    }
    pub fn add(&mut self, id: Id) {
        if !self.order.contains(&id) {
            self.order.push(id);
        }
    }
    pub fn remove(&mut self, id: Id) {
        self.order.retain(|&other| other != id);
        // removeCollidable does not remove the object from the current quad.
    }
    pub fn set_bounds(&mut self, b: Bounds, bounds: &HashMap<Id, Bounds>) {
        self.quad = Quad::new(b);
        for &id in &self.order {
            self.quad.add(id, bounds[&id]);
        }
    }
    pub fn update(&mut self, limit: Bounds, bounds: &HashMap<Id, Bounds>) -> Vec<(Id, Id)> {
        self.quad.clear_dynamic();
        for &id in &self.order {
            if !matches!(id, Id::Tree(_)) {
                self.quad.add(id, bounds[&id]);
            }
        }
        let mut measured: HashMap<Id, HashSet<Id>> = HashMap::new();
        let mut notifications = vec![];
        for &id in &self.order {
            let collider = bounds[&id];
            if !limit.contains(collider) {
                continue;
            }
            for near in self.quad.near(collider) {
                if near == id {
                    continue;
                }
                // Preserve the original's asymmetric first-entry branch. The
                // initial empty set is NOT populated until another encounter.
                if let Some(set) = measured.get_mut(&near) {
                    if !set.insert(id) {
                        continue;
                    }
                } else {
                    measured.insert(near, HashSet::new());
                }
                if measured.get(&id).is_some_and(|set| set.contains(&near)) {
                    continue;
                }
                if collider.intersects(bounds[&near]) {
                    notifications.push((id, near));
                    notifications.push((near, id));
                }
            }
        }
        notifications
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistent_registration_queries_and_notifications_match_original() {
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../data/snowball-collision-lifecycle-reference.json"
        ))
        .unwrap();
        for case in data["cases"].as_array().unwrap() {
            let static_ids: HashSet<_> = case["objects"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|o| o["static"].as_bool().unwrap())
                .map(|o| o["id"].as_u64().unwrap() as usize)
                .collect();
            let id = |n: usize| {
                if static_ids.contains(&n) {
                    Id::Tree(n)
                } else {
                    Id::Shot(n)
                }
            };
            let number = |id| match id {
                Id::Tree(n) | Id::Shot(n) => n,
                _ => unreachable!(),
            };
            let rect = |a: &serde_json::Value| {
                Bounds::rect(
                    V::new(a[2].as_f64().unwrap(), a[3].as_f64().unwrap()),
                    a[0].as_f64().unwrap(),
                    a[1].as_f64().unwrap(),
                )
            };
            let mut collision = Collision::new(Bounds::rect(V::default(), 1000., 800.));
            let mut world_bounds = Bounds::rect(V::default(), 1000., 800.);
            let mut limit = None;
            for (frame_index, frame) in case["frames"].as_array().unwrap().iter().enumerate() {
                let bounds: HashMap<_, _> = frame["objects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|o| {
                        (
                            id(o["id"].as_u64().unwrap() as usize),
                            Bounds {
                                pos: V::new(
                                    o["pos"][0].as_f64().unwrap(),
                                    o["pos"][1].as_f64().unwrap(),
                                ),
                                half: V::new(
                                    o["half"][0].as_f64().unwrap(),
                                    o["half"][1].as_f64().unwrap(),
                                ),
                                circle: o["circle"].as_bool().unwrap(),
                            },
                        )
                    })
                    .collect();
                let input = &frame["input"];
                if input["bounds"].is_array() {
                    world_bounds = rect(&input["bounds"]);
                    collision.set_bounds(world_bounds, &bounds);
                }
                if let Some(removes) = input["remove"].as_array() {
                    for n in removes {
                        collision.remove(id(n.as_u64().unwrap() as usize));
                    }
                }
                if let Some(adds) = input["add"].as_array() {
                    for n in adds {
                        collision.add(id(n.as_u64().unwrap() as usize));
                    }
                }
                if input["limit"].is_array() {
                    limit = Some(rect(&input["limit"]));
                }
                let notifications: Vec<_> = collision
                    .update(limit.unwrap_or(world_bounds), &bounds)
                    .into_iter()
                    .map(|(a, b)| [number(a), number(b)])
                    .collect();
                assert_eq!(
                    serde_json::json!(notifications),
                    frame["events"],
                    "{} frame {frame_index}",
                    case["name"]
                );
                assert_eq!(
                    collision.quad.node_count(),
                    frame["nodes"].as_u64().unwrap() as usize
                );
                assert_eq!(
                    serde_json::json!(collision
                        .order
                        .iter()
                        .copied()
                        .map(number)
                        .collect::<Vec<_>>()),
                    frame["order"]
                );
                for near in frame["near"].as_array().unwrap() {
                    let actual: Vec<_> = collision
                        .quad
                        .near(bounds[&id(near["id"].as_u64().unwrap() as usize)])
                        .into_iter()
                        .map(number)
                        .collect();
                    assert_eq!(
                        serde_json::json!(actual),
                        near["near"],
                        "{} frame {frame_index}",
                        case["name"]
                    );
                }
            }
        }
    }
    #[test]
    fn tree_notifications_match_original_collision_system() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-collision-reference.json"))
                .unwrap();
        for case in data["cases"].as_array().unwrap() {
            let shots: Vec<_> = case["shots"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| {
                    Bounds::circle(
                        V::new(s[0].as_f64().unwrap(), s[1].as_f64().unwrap()),
                        s[2].as_f64().unwrap(),
                    )
                })
                .collect();
            let mut collision = Collision::new(Bounds::rect(V::default(), 1000., 800.));
            let mut bounds = HashMap::new();
            for (i, p) in case["centers"].as_array().unwrap().iter().enumerate() {
                collision.add(Id::Tree(i));
                bounds.insert(
                    Id::Tree(i),
                    Bounds::circle(V::new(p[0].as_f64().unwrap(), p[1].as_f64().unwrap()), 14.),
                );
            }
            for (i, &b) in shots.iter().enumerate() {
                collision.add(Id::Shot(i));
                bounds.insert(Id::Shot(i), b);
            }
            let events: Vec<_> = collision
                .update(Bounds::rect(V::default(), 200., 200.), &bounds)
                .into_iter()
                .filter_map(|(id, other)| match (id, other) {
                    (Id::Shot(i), Id::Tree(tree)) => Some([tree, i]),
                    _ => None,
                })
                .collect();
            assert_eq!(
                serde_json::json!(events),
                case["events"],
                "{}",
                case["name"]
            );
        }
    }
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
