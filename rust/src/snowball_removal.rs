//! Original EntityRemovalSystem and Iceberg lifecycle.
//! Copyright 2020 Google LLC, Apache-2.0.
use crate::snowball::{Grid, V};
use crate::snowball_collision::Id;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Parent {
    Detached,
    Map,
    Canopy,
    Ice(usize),
}
#[derive(Clone, Copy, Debug)]
pub struct Presence {
    pub present: bool,
    pub exit_time: f64,
}
impl Default for Presence {
    fn default() -> Self {
        Self {
            present: true,
            exit_time: -1.,
        }
    }
}
impl Presence {
    pub fn exiting(self) -> bool {
        self.exit_time > -1.
    }
    pub fn gone(self) -> bool {
        !self.present && !self.exiting()
    }
    pub fn exit(&mut self, now: f64) {
        if !self.exiting() {
            self.exit_time = now;
        }
    }
    pub fn finish(&mut self) {
        self.present = false;
        self.exit_time = -1.;
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Ice {
    pub pos: V,
    pub z: f64,
    pub rotation: f64,
    pub opacity: f64,
    pub start_tick: f64,
    pub radius: Option<f64>,
}
impl Default for Ice {
    fn default() -> Self {
        Self {
            pos: V::default(),
            z: 0.,
            rotation: 0.,
            opacity: 0.,
            start_tick: 0.,
            radius: None,
        }
    }
}
#[derive(Clone, Copy)]
pub struct ActorState {
    pub pos: V,
    pub exit_time: f64,
    pub visibility: bool,
}
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Attached(Id, usize),
    Gone(Id),
    Teleport {
        id: Id,
        z: f64,
        opacity: Option<f64>,
        finished: bool,
    },
}
#[derive(Default)]
pub struct Removal {
    pub teleporting: Vec<Id>,
    pub freezing: Vec<Id>,
    pub frozen: Vec<Id>,
    pub bindings: Vec<(Id, usize)>,
    pub layer: Vec<usize>,
    pub bodies: Vec<Ice>,
    pub free: Vec<usize>,
}
impl Removal {
    pub fn freeze(&mut self, id: Id, now: f64, presence: Option<&mut Presence>) {
        if let Some(p) = presence {
            if !p.exiting() {
                p.exit(now);
                self.freezing.push(id);
            }
        }
    }
    pub fn teleport(&mut self, id: Id, now: f64, presence: Option<&mut Presence>) {
        if let Some(p) = presence {
            if !p.exiting() {
                p.exit(now);
                self.teleporting.push(id);
            }
        }
    }
    pub fn teardown(&mut self) {
        // Iceberg has no onFreed reset: its transform/uniform state survives.
        for &(_, slot) in &self.bindings {
            self.free.push(slot);
        }
        self.bindings.clear();
        self.layer.clear();
        self.teleporting.clear();
        self.freezing.clear();
        self.frozen.clear();
    }
    pub fn body(&self, slot: usize) -> Option<&Ice> {
        self.layer.contains(&slot).then(|| &self.bodies[slot])
    }
    pub fn update(
        &mut self,
        grid: Grid,
        rings: Option<usize>,
        tick: f64,
        now: f64,
        width: f64,
        height: f64,
        state: impl Fn(Id) -> ActorState,
    ) -> Vec<Event> {
        let mut events = vec![];
        while let Some(id) = self.freezing.pop() {
            let slot = self.free.pop().unwrap_or_else(|| {
                self.bodies.push(Ice::default());
                self.bodies.len() - 1
            });
            let b = &mut self.bodies[slot];
            b.start_tick = tick;
            if b.radius != Some(grid.size / 2.) {
                b.radius = Some(grid.size / 2.);
                b.opacity = 0.;
            }
            b.pos = grid
                .pos(grid.position_index(state(id).pos))
                .add(V::new(0., -32.));
            b.z = 32.; // indexToPosition writes z=0 before the source adds 32.
            self.bindings.push((id, slot));
            self.frozen.push(id);
            self.layer.push(slot);
            events.push(Event::Attached(id, slot));
        }
        let Some(rings) = rings else {
            return events;
        };
        let limit = V::new(width / 2., height / 2.).len() + (rings as f64 + 2.) * grid.size;
        let mut i = 0;
        while i < self.frozen.len() {
            let id = self.frozen[i];
            let slot = self.bindings.iter().find(|(e, _)| *e == id).unwrap().1;
            let b = &mut self.bodies[slot];
            let delta = tick - b.start_tick;
            b.pos = b.pos.add(b.pos.norm().norm().mul(1. / 3.));
            let scale = (delta / 75.).min(1.);
            let diminish = scale * scale * scale;
            b.rotation = (delta / 500.).sin() * std::f64::consts::PI;
            b.z = (delta / 10.).cos() * (grid.size / 2.) * (1. - diminish * 0.9);
            b.opacity = diminish;
            if b.pos.len() > limit {
                self.frozen.remove(i);
                self.bindings.retain(|(e, _)| *e != id);
                self.layer.retain(|&s| s != slot);
                self.free.push(slot);
                events.push(Event::Gone(id));
            } else {
                i += 1;
            }
        }
        let mut i = 0;
        while i < self.teleporting.len() {
            let id = self.teleporting[i];
            let a = state(id);
            let elapsed = now - a.exit_time - 1500.;
            if elapsed < 0. {
                i += 1;
                continue;
            }
            let t = (elapsed / 700.).min(1.);
            events.push(Event::Teleport {
                id,
                z: t * t * grid.size / 1.5,
                opacity: a.visibility.then_some(1. - t),
                finished: t == 1.,
            });
            if t == 1. {
                self.teleporting.remove(i);
            } else {
                i += 1;
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    #[derive(Clone, Copy)]
    struct Actor {
        pos: [f64; 3],
        presence: Option<Presence>,
        opacity: Option<f64>,
        parent: Parent,
    }
    fn id(s: &str) -> Id {
        let n = s[1..].parse().unwrap();
        match &s[..1] {
            "p" => Id::Player(n),
            "d" => Id::Drop(n),
            "s" => Id::Shot(n),
            _ => unreachable!(),
        }
    }
    fn label(e: Id) -> String {
        match e {
            Id::Player(i) => format!("p{i}"),
            Id::Drop(i) => format!("d{i}"),
            Id::Shot(i) => format!("s{i}"),
            _ => unreachable!(),
        }
    }
    #[test]
    fn original_removal_queues_presence_iceberg_pool_and_teleport() {
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-removal-reference.json")).unwrap();
        for case in data["cases"].as_array().unwrap() {
            let mut removal = Removal::default();
            let grid = Grid::default();
            let mut actors: HashMap<_, _> = case["objects"]
                .as_array()
                .unwrap()
                .iter()
                .map(|o| {
                    (
                        id(o["id"].as_str().unwrap()),
                        Actor {
                            pos: std::array::from_fn(|i| o["pos"][i].as_f64().unwrap()),
                            presence: (o["presence"] != false).then(Presence::default),
                            opacity: o["visibility"].as_bool().unwrap_or(false).then_some(1.),
                            parent: Parent::Map,
                        },
                    )
                })
                .collect();
            let snapshot = |r: &Removal, a: &HashMap<Id, Actor>| {
                json!({
                    "teleporting":r.teleporting.iter().copied().map(label).collect::<Vec<_>>(),"freezing":r.freezing.iter().copied().map(label).collect::<Vec<_>>(),"frozen":r.frozen.iter().copied().map(label).collect::<Vec<_>>(),
                    "bindings":r.bindings.iter().map(|&(e,s)|json!([label(e),s])).collect::<Vec<_>>(),"layer":r.layer,"free":r.free,
                    "bodies":r.bodies.iter().map(|b|json!({"pos":[b.pos.x,b.pos.y,b.z],"rotation":b.rotation,"opacity":b.opacity,"start":b.start_tick,"radius":b.radius})).collect::<Vec<_>>(),
                    "actors":case["objects"].as_array().unwrap().iter().map(|o|{let e=id(o["id"].as_str().unwrap());let a=a[&e];json!({"id":label(e),"pos":a.pos,"present":a.presence.map(|p|p.present),"exit":a.presence.map(|p|p.exit_time),"gone":a.presence.map(Presence::gone),"opacity":a.opacity,"parent":match a.parent{Parent::Map=>Some("map".into()),Parent::Ice(i)=>Some(format!("ice{i}")),Parent::Detached=>None,Parent::Canopy=>unreachable!()}})}).collect::<Vec<_>>()
                })
            };
            let (mut map, mut rings, mut w, mut h) = (true, 4, 1440., 800.);
            for f in case["frames"].as_array().unwrap() {
                let input = &f["input"];
                let now = input["at"].as_f64().unwrap();
                let tick = input["tick"].as_f64().unwrap();
                if let Some(m) = input["map"].as_bool() {
                    map = m;
                }
                if let Some(n) = input["rings"].as_u64() {
                    rings = n as usize;
                }
                if input["size"].is_array() {
                    w = input["size"][0].as_f64().unwrap();
                    h = input["size"][1].as_f64().unwrap();
                }
                for key in ["reset", "freeze", "teleport"] {
                    if let Some(entries) = input[key].as_array() {
                        for e in entries {
                            let e = id(e.as_str().unwrap());
                            let p = &mut actors.get_mut(&e).unwrap().presence;
                            match key {
                                "reset" => *p = Some(Presence::default()),
                                "freeze" => removal.freeze(e, now, p.as_mut()),
                                _ => removal.teleport(e, now, p.as_mut()),
                            }
                        }
                    }
                }
                if input["teardown"].as_bool().unwrap_or(false) {
                    removal.teardown();
                }
                let expected = |key: &str| {
                    let mut v = f[key].clone();
                    for b in v["bodies"].as_array_mut().unwrap() {
                        b.as_object_mut().unwrap().remove("matrix");
                    }
                    v
                };
                crate::snowball::tests::assert_json_close(
                    &snapshot(&removal, &actors),
                    &expected("before"),
                    &format!("{} before {now}", case["name"]),
                );
                let events = removal.update(grid, map.then_some(rings), tick, now, w, h, |e| {
                    let a = actors[&e];
                    ActorState {
                        pos: V::new(a.pos[0], a.pos[1]),
                        exit_time: a.presence.unwrap().exit_time,
                        visibility: a.opacity.is_some(),
                    }
                });
                for event in events {
                    match event {
                        Event::Attached(e, s) => {
                            let a = actors.get_mut(&e).unwrap();
                            a.pos = [0., 0., -16.];
                            a.parent = Parent::Ice(s);
                        }
                        Event::Gone(e) => {
                            let a = actors.get_mut(&e).unwrap();
                            a.parent = Parent::Detached;
                            a.presence.as_mut().unwrap().finish();
                        }
                        Event::Teleport {
                            id,
                            z,
                            opacity,
                            finished,
                        } => {
                            let a = actors.get_mut(&id).unwrap();
                            a.pos[2] = z;
                            if let Some(o) = opacity {
                                a.opacity = Some(o);
                            }
                            if finished {
                                a.presence.as_mut().unwrap().finish();
                            }
                        }
                    }
                }
                crate::snowball::tests::assert_json_close(
                    &snapshot(&removal, &actors),
                    &expected("after"),
                    &format!("{} after {now}", case["name"]),
                );
            }
        }
    }
}
