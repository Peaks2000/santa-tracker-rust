//! Living ClientSystem arrival marker and MainLevel camera tracking.
//! Source: Google Santa Tracker (2020), Apache-2.0. Input/network/death are separate.
use crate::snowball::V;

#[derive(Clone, Copy, Debug)]
pub struct ArrivalMarker {
    pub position: [f64; 3],
    pub rotation: f64,
}
#[derive(Default)]
pub struct Arrival {
    pub marker: Option<ArrivalMarker>,
}
impl Arrival {
    /// ClientSystem returns before touching the marker when no player or dead.
    /// Once attached, its transform stays fixed even if the assigned tile changes.
    pub fn update(
        &mut self,
        player: Option<(bool, bool, i32, f64, f64)>,
        tile_position: V,
        camera: &mut V,
    ) {
        let Some((alive, arrived, tile, dolly_y, rotation)) = player else {
            return;
        };
        if !alive {
            return;
        }
        if arrived {
            self.marker = None;
        } else if self.marker.is_none() && tile > -1 {
            self.marker = Some(ArrivalMarker {
                position: [tile_position.x, tile_position.y + dolly_y, 19.],
                rotation,
            });
            *camera = V::new(tile_position.x, tile_position.y * -0.75);
        }
    }
    pub fn teardown(&mut self) {
        self.marker = None;
    }
}
/// Source TetheredCameraTracker: acceleration and friction are per update.
/// Its inner-tether branch uses the browser's unqualified `length` (0 here).
pub fn track(camera: &mut V, velocity: &mut V, target: V, width: f64, height: f64) {
    let delta = V::new(target.x, -0.75 * target.y).sub(*camera);
    let tether = 0.05 * width.max(height);
    if delta.len() >= tether {
        *velocity = velocity.add(delta.norm().mul(0.15));
    }
    if velocity.len() > 0.1 {
        velocity.x += if velocity.x > 0. { -0.1 } else { 0.1 };
        velocity.y += if velocity.y > 0. { -0.1 } else { 0.1 };
    } else {
        *velocity = V::default();
    }
    *camera = camera.add(*velocity);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arrival_and_camera_match_original_client_and_main_level() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-client-reference.json")).unwrap();
        let close = |a: f64, b: f64| assert!((a - b).abs() < 1e-9, "{a} != {b}");
        for case in data["cases"].as_array().unwrap() {
            let mut arrival = Arrival::default();
            let mut camera = V::new(
                case["camera"][0].as_f64().unwrap(),
                case["camera"][1].as_f64().unwrap(),
            );
            let mut velocity = V::default();
            let (mut alive, mut arrived, mut tile, mut dolly_y) = (true, false, -1, -10.);
            let (mut exiting, mut gone, mut present) = (false, false, true);
            let mut pos = V::default();
            let mut game = crate::snowball::Snowball::new("client-camera-oracle");
            game.start();
            let prototype = game.players[0].clone();
            let (mut width, mut height) = (1440., 800.);
            for frame in case["frames"].as_array().unwrap() {
                let input = &frame["input"];
                if let Some(v) = input["tile"].as_i64() {
                    tile = v as i32;
                }
                if let Some(v) = input["arrived"].as_bool() {
                    arrived = v;
                }
                if let Some(v) = input["alive"].as_bool() {
                    alive = v;
                }
                if let Some(v) = input["exiting"].as_bool() {
                    exiting = v;
                }
                if let Some(v) = input["gone"].as_bool() {
                    gone = v;
                }
                if let Some(v) = input["noPlayer"].as_bool() {
                    present = !v;
                }
                if let Some(v) = input["dollyY"].as_f64() {
                    dolly_y = v;
                }
                if let Some(v) = input["width"].as_f64() {
                    width = v;
                }
                if let Some(v) = input["height"].as_f64() {
                    height = v;
                }
                if input["pos"].is_array() {
                    pos = V::new(
                        input["pos"][0].as_f64().unwrap(),
                        input["pos"][1].as_f64().unwrap(),
                    );
                }
                game.players.clear();
                if present {
                    let mut player = prototype.clone();
                    player.alive = alive;
                    player.arrived = arrived;
                    player.pos = pos;
                    player.presence.present = !gone;
                    player.presence.exit_time = if exiting { 100. } else { -1. };
                    game.players.push(player);
                    if input["pick"].is_array() {
                        game.pick_destination(V::new(
                            input["pick"][0].as_f64().unwrap(),
                            input["pick"][1].as_f64().unwrap(),
                        ));
                    }
                }
                game.camera = camera;
                game.camera_velocity = velocity;
                assert!(game.begin_frame(0., width, height));
                camera = game.camera;
                velocity = game.camera_velocity;
                close(camera.x, frame["beforeClient"][0].as_f64().unwrap());
                close(camera.y, frame["beforeClient"][1].as_f64().unwrap());
                if input["teardown"].as_bool() == Some(true) {
                    arrival.teardown();
                    present = false;
                } else {
                    arrival.update(
                        present.then_some((
                            alive,
                            arrived,
                            tile,
                            dolly_y,
                            std::f64::consts::PI / 2.25,
                        )),
                        crate::snowball::Grid::default().pos(tile),
                        &mut camera,
                    );
                }
                close(camera.x, frame["camera"][0].as_f64().unwrap());
                close(camera.y, frame["camera"][1].as_f64().unwrap());
                close(velocity.x, frame["velocity"][0].as_f64().unwrap());
                close(velocity.y, frame["velocity"][1].as_f64().unwrap());
                match arrival.marker {
                    None => assert!(frame["marker"].is_null()),
                    Some(marker) => {
                        for i in 0..3 {
                            close(
                                marker.position[i],
                                frame["marker"]["position"][i].as_f64().unwrap(),
                            );
                        }
                        close(
                            marker.rotation,
                            frame["marker"]["rotation"].as_f64().unwrap(),
                        );
                    }
                }
            }
        }
    }
}
