//! Source modvil ratioForElement and prepareModules/desired-parts policy.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Normal,
    Small,
    Mobile,
}
impl Mode {
    pub fn for_width(width: f64) -> Self {
        if width <= 414. {
            Self::Mobile
        } else if width <= 768. {
            Self::Small
        } else {
            Self::Normal
        }
    }
}
pub fn wanted(parts: &[String], mode: Mode, load: bool, desktop: bool) -> Vec<Option<String>> {
    let mut want = parts
        .iter()
        .map(|p| {
            if matches!(mode, Mode::Small) && !p.starts_with("static-") {
                None
            } else {
                Some(p.clone())
            }
        })
        .collect::<Vec<_>>();
    if matches!(mode, Mode::Mobile) && !desktop {
        want.fill(None);
        want.push(Some("mobile".into()));
    } else {
        want.push(None);
    }
    if !load {
        want.fill(None);
    }
    if !want.iter().any(|p| p.as_deref() == Some("cards")) {
        want.push(Some("cards".into()));
    }
    want
}
pub fn ratio(scroll: f64, viewport: f64, document: f64, top: f64, height: f64) -> f64 {
    let start = 0f64.max(top - viewport);
    let end = (document - viewport).min(top + height);
    let ratio = (scroll - start) / (end - start);
    if ratio <= 0. {
        0.
    } else if ratio >= 1. {
        1.
    } else {
        (((ratio - 0.5) * std::f64::consts::PI).sin() + 1.) * 0.5
    }
}
pub fn visible(scroll: f64, viewport: f64, top: f64, height: f64) -> bool {
    top < scroll + viewport + 24. && top + height >= scroll - 24.
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_ratio_visibility_and_part_modes_match() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/module-viewport-reference.json")).unwrap();
        for case in data["ratios"].as_array().unwrap() {
            let i = &case["input"];
            let n = |k: &str| i[k].as_f64().unwrap();
            let actual = ratio(
                n("scroll"),
                n("viewport"),
                n("document"),
                n("top"),
                n("height"),
            );
            if let Some(expected) = case["ratio"].as_f64() {
                assert!((actual - expected).abs() < 1e-12, "{i}: {actual}");
            } else {
                assert!(actual.is_nan());
            }
            assert_eq!(
                visible(n("scroll"), n("viewport"), n("top"), n("height")),
                case["visible"].as_bool().unwrap()
            );
        }
        for case in data["parts"].as_array().unwrap() {
            let i = &case["input"];
            let parts: Vec<String> = serde_json::from_value(i["parts"].clone()).unwrap();
            let mode = Mode::for_width(i["width"].as_f64().unwrap());
            let actual = wanted(
                &parts,
                mode,
                i["load"].as_bool().unwrap(),
                i["desktop"].as_bool().unwrap(),
            );
            assert_eq!(serde_json::to_value(actual).unwrap(), case["output"], "{i}");
        }
    }
}
