//! Direct decision order of the source Bot.update. Controllers outlive removed
//! elves; full pooled actor identity and BotSystem queue ownership are separate.
use crate::{snowball::V, snowball_collision::Id};

#[derive(Clone, Copy, Default)]
pub struct State {
    pub last_target: Option<usize>,
    pub last_throw: f64,
}
/// Source Bot owns this state independently from its pooled Elf reference.
#[derive(Clone, Copy)]
pub struct Controller {
    pub player: usize,
    pub state: State,
}
#[derive(Clone, Copy)]
pub struct Player {
    pub id: usize,
    pub position: [f64; 3],
    pub arrived: bool,
    pub destination_reached: bool,
}
#[derive(Default)]
pub struct Decision {
    pub destination: Option<V>,
    pub target: Option<usize>,
}

pub fn update<R, W, Q, P>(
    state: &mut State,
    player: Player,
    tick: f64,
    random: &mut R,
    wander: W,
    query: Q,
    position: P,
) -> Decision
where
    R: FnMut() -> f64,
    W: FnOnce(&mut R) -> V,
    Q: FnOnce() -> Vec<Id>,
    P: Fn(usize) -> [f64; 3],
{
    let mut decision = Decision::default();
    if !player.arrived {
        return decision;
    }
    if player.destination_reached && random() > 0.985 {
        decision.destination = Some(wander(random));
    }
    let nearby = query();
    let mut next = state
        .last_target
        .filter(|id| nearby.contains(&Id::Player(*id)));
    if next.is_none() {
        for candidate in nearby {
            let Id::Player(id) = candidate else { continue };
            let target = position(id);
            // Use the source Vector3 square/sum/sqrt order, including its
            // underflow-to-zero behavior. Health/presence do not filter targets.
            if id == player.id || length(target) == 0. {
                continue;
            }
            // Source draws before checking range; rejected far targets consume
            // a value. A retained target bypasses these selection filters.
            if random() > 0.25
                && length(std::array::from_fn(|i| player.position[i] - target[i])) < 320.
            {
                next = Some(id);
                break;
            }
        }
    }
    if let Some(id) = next {
        if tick - state.last_throw > 40. {
            if random() > 0.5 {
                decision.target = Some(id);
            }
            // A failed throw coin still advances cooldown.
            state.last_throw = tick;
        }
    }
    state.last_target = next;
    decision
}
fn length([x, y, z]: [f64; 3]) -> f64 {
    (x * x + y * y + z * z).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snowball::{Grid, Map};
    use serde_json::{json, Value};

    #[test]
    fn decisions_random_order_and_nearby_tiles_match_original_bot() {
        let reference: Value =
            serde_json::from_str(include_str!("../data/snowball-bot-reference.json")).unwrap();
        assert_eq!(reference["cases"].as_array().unwrap().len(), 35);
        let mut total_draws = 0;
        for case in reference["cases"].as_array().unwrap() {
            let input = &case["input"];
            let position = |v: &Value| std::array::from_fn(|i| v[i].as_f64().unwrap());
            let own_position = position(&input["position"]);
            let targets = input["targets"].as_array().unwrap();
            let mut map = Map::new(Grid::default(), 1);
            if let Some(tile) = input["originTile"].as_u64() {
                map.set_state(
                    tile as usize,
                    input["originState"].as_u64().unwrap() as u8,
                    0.,
                );
                map.obstacles[tile as usize] = -1;
            }
            let mut state = State {
                last_target: input["lastTarget"].as_u64().map(|v| v as usize),
                last_throw: input["lastThrow"].as_f64().unwrap(),
            };
            let mut draws = 0;
            let queries = std::cell::Cell::new(0usize);
            let mut random = || {
                let value = input["random"][draws]
                    .as_f64()
                    .expect("random tape exhausted");
                draws += 1;
                value
            };
            let decision = update(
                &mut state,
                Player {
                    id: 1,
                    position: own_position,
                    arrived: input["arrived"].as_bool().unwrap(),
                    destination_reached: !input["path"].as_bool().unwrap(),
                },
                input["tick"].as_f64().unwrap(),
                &mut random,
                |random| {
                    map.grid.pos(
                        map.nearby_with_random(
                            map.grid
                                .position_index(V::new(own_position[0], own_position[1])),
                            6,
                            random,
                        ),
                    )
                },
                || {
                    queries.set(queries.get() + 1);
                    targets
                        .iter()
                        .enumerate()
                        .map(|(i, t)| {
                            t["id"]
                                .as_u64()
                                .map_or(Id::Tree(i), |id| Id::Player(id as usize))
                        })
                        .collect()
                },
                |id| {
                    position(
                        &targets
                            .iter()
                            .find(|t| t["id"].as_u64() == Some(id as u64))
                            .unwrap()["position"],
                    )
                },
            );
            let mut commands = vec![];
            if let Some(p) = decision.destination {
                commands.push(json!({"kind":"move","id":1,"index":map.grid.position_index(p),"position":[p.x,p.y,0.]}));
            }
            if let Some(id) = decision.target {
                let reference = targets
                    .iter()
                    .position(|t| t["id"].as_u64() == Some(id as u64))
                    .unwrap();
                commands.push(json!({"kind":"target","id":1,"position":targets[reference]["position"],"reference":reference}));
            }
            crate::snowball::tests::assert_json_close(
                &json!({"draws":draws,"queries":queries.get(),"commands":commands,"lastThrow":state.last_throw,"lastTarget":state.last_target}),
                &case["output"], case["name"].as_str().unwrap(),
            );
            total_draws += draws;
        }
        assert_eq!(total_draws, 303);
    }
}
