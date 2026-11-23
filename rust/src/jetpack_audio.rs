//! Original jp_player parameter processes flattened over cXIa's two sources.
use crate::{audio::Command, jetpack::PlayerSound};
pub fn commands(event: &PlayerSound) -> Option<Command> {
    let value = event.args[0];
    match event.name {
        "jp_player_moving" => {
            let gain = if value < 0.5 { 0.5 } else { value };
            Some(Command::GroupBusGain {
                ids: vec!["XgMa".into(), "Kzah".into()],
                gain: gain as f32,
            })
        }
        "jp_player_position" => {
            let pan = if value > 0.75 {
                0.75
            } else if value < -0.75 {
                -0.75
            } else {
                value
            };
            Some(Command::GroupPanSource {
                ids: vec!["XgMa".into(), "Kzah".into()],
                pan: pan as f32,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_klang_actions_match_gain_pan_and_bus_membership() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../data/jetpack-audio-parameters.json")).unwrap();
        let mut expected: Vec<_> = reference["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                assert_eq!(s["channels"], 2);
                s["id"].as_str().unwrap().to_owned()
            })
            .collect();
        expected.sort();
        for case in reference["cases"].as_array().unwrap() {
            let name = if case["name"] == "jp_player_moving" {
                "jp_player_moving"
            } else {
                "jp_player_position"
            };
            let event = PlayerSound {
                name,
                args: [case["value"].as_f64().unwrap()],
            };
            let (mut ids, value, target) = match commands(&event).unwrap() {
                Command::GroupBusGain { ids, gain } => {
                    (ids, gain, case["gain"].as_f64().unwrap() as f32)
                }
                Command::GroupPanSource { ids, pan } => {
                    (ids, pan, case["pan"].as_f64().unwrap() as f32)
                }
                _ => panic!("wrong command for {name}"),
            };
            ids.sort();
            assert_eq!(ids, expected);
            assert_eq!(value, target, "{case}");
        }
    }
}
