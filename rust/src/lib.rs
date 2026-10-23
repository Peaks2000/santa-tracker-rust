pub mod animation;
pub mod audio;
pub mod image_alpha;
pub mod jetpack;
pub mod navigation;
pub mod presentdrop;
pub mod random;
pub mod runner;
pub mod selfie;
pub mod ski;
pub mod ui;
pub mod village_data;
pub mod weather;

#[cfg(test)]
mod tests {
    use super::runner::Runner;
    use serde_json::Value;
    fn compare(actual: &Value, expected: &Value, path: &str) {
        match (actual, expected) {
            (Value::Number(a), Value::Number(b)) => {
                let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
                assert!((a - b).abs() < 1e-7, "{path}: {a} != {b}");
            }
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path} array lengths");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}[{i}]"));
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(a.len(), b.len(), "{path} object lengths");
                for (key, b) in b {
                    compare(&a[key], b, &format!("{path}.{key}"));
                }
            }
            _ => assert_eq!(actual, expected, "{path}"),
        }
    }
    #[test]
    fn runner_matches_original_javascript_scenarios() {
        let scenarios: Value =
            serde_json::from_str(include_str!("../data/runner-reference.json")).unwrap();
        for scenario in scenarios.as_array().unwrap() {
            let mut game = Runner::new("test1234");
            game.immediate_levels = true;
            for (i, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
                match step["action"].as_str().unwrap() {
                    "advance" => game.advance(step["value"].as_f64().unwrap()),
                    "jump" => game.jump(),
                    "down" => game.down(),
                    "end_slide" => game.end_slide(),
                    _ => unreachable!(),
                }
                compare(
                    &game.snapshot(),
                    &step["state"],
                    &format!("{} step {i}", scenario["name"]),
                );
            }
        }
    }
    #[test]
    fn jetpack_matches_original_scenarios() {
        let cases: Value =
            serde_json::from_str(include_str!("../data/jetpack-reference.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let mut game = super::jetpack::Jetpack::new("test1234", 1440., 800.);
            for (i, step) in case["steps"].as_array().unwrap().iter().enumerate() {
                game.advance(
                    step["duration"].as_f64().unwrap(),
                    (
                        step["input"][0].as_f64().unwrap(),
                        step["input"][1].as_f64().unwrap(),
                    ),
                );
                compare(
                    &game.snapshot(),
                    &step["state"],
                    &format!("jetpack {} step {i}", case["name"]),
                );
            }
        }
    }
    #[test]
    fn selfie_matches_original_physics_and_tools() {
        use super::selfie::{Selfie, Tool};
        let steps: Value =
            serde_json::from_str(include_str!("../data/selfie-reference.json")).unwrap();
        let mut game = Selfie::new("test1234");
        for (i, step) in steps.as_array().unwrap().iter().enumerate() {
            let action = step["action"].as_str().unwrap();
            game.tool = match action {
                "clip" => Tool::Clipper,
                "dryer" => Tool::Dryer,
                "grow" => Tool::Grow,
                "spray" => Tool::Spray(2),
                "clean" => Tool::Clean,
                "decorate" => Tool::Decoration(1),
                _ => Tool::Hand,
            };
            game.pointer(
                step["x"].as_f64().unwrap(),
                step["y"].as_f64().unwrap(),
                step["down"].as_bool().unwrap(),
                1.,
                false,
            );
            for _ in 0..step["frames"].as_u64().unwrap() {
                game.step();
            }
            if action == "grow" {
                game.grow();
            }
            compare(
                &game.snapshot(),
                &step["state"],
                &format!("selfie step {i} {action}"),
            );
        }
    }
}
