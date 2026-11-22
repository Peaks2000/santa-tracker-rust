//! Original ParachuteSystem queue stages and stored, per-update transforms.
//! Copyright 2020 Google LLC, Apache-2.0.
use crate::snowball::V;
use crate::snowball_collision::Id;

#[derive(Clone, Copy, Debug)]
pub struct Canopy {
    pub entity: Id,
    pub pos: V,
    pub z: f64,
    pub rotation: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Carried(Id, f64),
    Arrived(Id),
    Detached(Id),
}
#[derive(Default)]
pub struct Parachutes {
    pub undropped: Vec<Id>,
    pub dropping: Vec<Id>,
    pub removed: Vec<Id>,
    pub canopies: Vec<Canopy>,
}
impl Parachutes {
    pub fn drop_entity(&mut self, id: Id) {
        // Only these scene actors have an Arrival component.
        if matches!(id, Id::Player(_) | Id::Drop(_)) {
            self.undropped.push(id);
        }
    }
    pub fn remove_entity(&mut self, id: Id) {
        if let Some(i) = self.undropped.iter().position(|&other| other == id) {
            self.undropped.remove(i);
        }
        if let Some(i) = self.dropping.iter().position(|&other| other == id) {
            self.dropping.remove(i);
        }
        self.removed.push(id);
    }
    pub fn teardown(&mut self) {
        // Source forEach mutates the same array via splice. Retain its skipped
        // successors and deferred detach, rather than draining the queues.
        let mut i = 0;
        while i < self.undropped.len() {
            self.remove_entity(self.undropped[i]);
            i += 1;
        }
        i = 0;
        while i < self.dropping.len() {
            self.remove_entity(self.dropping[i]);
            i += 1;
        }
    }
    pub fn get(&self, id: Id) -> Option<&Canopy> {
        self.canopies.iter().find(|c| c.entity == id)
    }
    /// state returns the actor's tile position and current Arrival.droppedTick.
    /// Queued carries reset that tick before the same update advances descent.
    pub fn update(
        &mut self,
        tick: f64,
        cell_size: f64,
        frame_count: f64,
        drop_height: f64,
        state: impl Fn(Id) -> (V, f64),
    ) -> Vec<Event> {
        let mut events = vec![];
        for id in std::mem::take(&mut self.removed) {
            if let Some(i) = self.canopies.iter().position(|c| c.entity == id) {
                self.canopies.remove(i);
                events.push(Event::Detached(id));
            }
        }
        let new = std::mem::take(&mut self.undropped);
        for &id in &new {
            let (pos, _) = state(id);
            self.canopies.push(Canopy {
                entity: id,
                pos,
                z: 0.,
                rotation: 0.,
            });
            self.dropping.push(id);
            events.push(Event::Carried(id, tick));
        }
        let mut i = 0;
        while i < self.dropping.len() {
            let id = self.dropping[i];
            let (_, previous_tick) = state(id);
            let dropped = if new.contains(&id) {
                tick
            } else {
                previous_tick
            };
            let delta = tick - dropped;
            let position = drop_height - delta / frame_count * drop_height;
            let canopy = self
                .canopies
                .iter_mut()
                .find(|c| c.entity == id)
                .expect("dropping canopy");
            canopy.z = position + 64. + cell_size / 4.;
            canopy.rotation = 0.1 * (position / (0.35 * drop_height) * std::f64::consts::PI).sin();
            if delta >= frame_count {
                self.dropping.remove(i);
                self.canopies.retain(|c| c.entity != id);
                events.push(Event::Arrived(id));
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
    use crate::snowball::Grid;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    fn id(label: &str) -> Id {
        let n = label[1..].parse().unwrap();
        match &label[..1] {
            "p" => Id::Player(n),
            "d" => Id::Drop(n),
            "s" => Id::Shot(n),
            _ => unreachable!(),
        }
    }
    fn label(id: Id) -> String {
        match id {
            Id::Player(i) => format!("p{i}"),
            Id::Drop(i) => format!("d{i}"),
            Id::Shot(i) => format!("s{i}"),
            _ => unreachable!(),
        }
    }
    #[test]
    fn original_parachute_queues_carry_arrival_removal_and_teardown() {
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-arrival-reference.json")).unwrap();
        for case in data["cases"].as_array().unwrap() {
            let mut para = Parachutes::default();
            let mut actors: HashMap<Id, ([f64; 3], f64, bool, i32)> = case["objects"]
                .as_array()
                .unwrap()
                .iter()
                .map(|o| {
                    (
                        id(o["id"].as_str().unwrap()),
                        (
                            [77., 88., 99.],
                            f64::INFINITY,
                            false,
                            o["tile"].as_i64().unwrap() as i32,
                        ),
                    )
                })
                .collect();
            let snapshot = |p: &Parachutes, a: &HashMap<Id, ([f64; 3], f64, bool, i32)>| {
                json!({
                    "undropped":p.undropped.iter().copied().map(label).collect::<Vec<_>>(),
                    "dropping":p.dropping.iter().copied().map(label).collect::<Vec<_>>(),
                    "removed":p.removed.iter().copied().map(label).collect::<Vec<_>>(),
                    "canopies":p.canopies.iter().map(|c|json!({"id":label(c.entity),"pos":[c.pos.x,c.pos.y,c.z],"rotation":c.rotation})).collect::<Vec<_>>(),
                    "actors":case["objects"].as_array().unwrap().iter().map(|o|{
                        let key=id(o["id"].as_str().unwrap());let (pos,tick,arrived,_)=a[&key];
                        json!({"id":label(key),"pos":pos,"dropped":tick.is_finite().then_some(tick),
                            "arrived":if matches!(key,Id::Shot(_)){None}else{Some(arrived)},"parent":p.get(key).map(|_|"parachute")})
                    }).collect::<Vec<_>>(),"lod":p.canopies.len()
                })
            };
            for frame in case["frames"].as_array().unwrap() {
                let input = &frame["input"];
                if let Some(drop) = input["drop"].as_array() {
                    for e in drop {
                        para.drop_entity(id(e.as_str().unwrap()));
                    }
                }
                if let Some(remove) = input["remove"].as_array() {
                    for e in remove {
                        para.remove_entity(id(e.as_str().unwrap()));
                    }
                }
                if input["teardown"].as_bool().unwrap_or(false) {
                    para.teardown();
                }
                let expected = |key: &str| {
                    let mut v = frame[key].clone();
                    for c in v["canopies"].as_array_mut().unwrap() {
                        for k in ["graphic", "carried", "sample"] {
                            c.as_object_mut().unwrap().remove(k);
                        }
                    }
                    v
                };
                crate::snowball::tests::assert_json_close(
                    &snapshot(&para, &actors),
                    &expected("before"),
                    case["name"].as_str().unwrap(),
                );
                let events = para.update(input["tick"].as_f64().unwrap(), 64., 300., 420., |e| {
                    let (_, tick, _, tile) = actors[&e];
                    (Grid::default().pos(tile), tick)
                });
                for event in events {
                    match event {
                        Event::Carried(e, tick) => {
                            let actor = actors.get_mut(&e).unwrap();
                            actor.0 = [0.; 3];
                            actor.1 = tick;
                        }
                        Event::Arrived(e) => actors.get_mut(&e).unwrap().2 = true,
                        Event::Detached(_) => {}
                    }
                }
                crate::snowball::tests::assert_json_close(
                    &snapshot(&para, &actors),
                    &expected("after"),
                    case["name"].as_str().unwrap(),
                );
            }
        }
    }
}
