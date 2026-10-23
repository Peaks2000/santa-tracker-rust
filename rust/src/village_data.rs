use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Keyframe {
    pub at: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub sx: f32,
    pub sy: f32,
    #[serde(default)]
    pub clip_top: f32,
    #[serde(default = "one")]
    pub clip_bottom: f32,
}
fn one() -> f32 {
    1.
}
impl Default for Keyframe {
    fn default() -> Self {
        Self {
            at: 0.,
            x: 0.,
            y: 0.,
            rotation: 0.,
            sx: 1.,
            sy: 1.,
            clip_top: 0.,
            clip_bottom: 1.,
        }
    }
}
#[derive(Clone, Deserialize)]
pub struct Sprite {
    pub name: String,
    pub layer: u32,
    pub asset: String,
    pub x: f32,
    pub center: bool,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub animation: String,
    pub duration: f32,
    pub delay: f32,
    pub ease: String,
    pub mobile_y: f32,
}
#[derive(Clone, Deserialize)]
pub struct Card {
    pub id: String,
    pub title: String,
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Deserialize)]
pub struct Module {
    pub id: String,
    pub color: String,
    pub parts: Vec<String>,
    pub no_divider: bool,
    pub cards: Vec<Card>,
}
#[derive(Deserialize)]
pub struct VillageData {
    pub sprites: Vec<Sprite>,
    pub frames: HashMap<String, Vec<Keyframe>>,
    pub modules: Vec<Module>,
    pub strings: HashMap<String, String>,
    pub colors: HashMap<String, String>,
    #[serde(rename = "sceneNames")]
    pub scene_names: HashMap<String, String>,
}
impl VillageData {
    pub fn original() -> Self {
        serde_json::from_str(include_str!("../data/village.json"))
            .expect("exported original village data")
    }
}
pub fn ease_in_out(x: f32) -> f32 {
    fn cubic(t: f32, a: f32, b: f32) -> f32 {
        3. * (1. - t) * (1. - t) * t * a + 3. * (1. - t) * t * t * b + t * t * t
    }
    let mut low = 0.;
    let mut high = 1.;
    for _ in 0..24 {
        let t = (low + high) * 0.5;
        if cubic(t, 0.42, 0.58) < x {
            low = t;
        } else {
            high = t;
        }
    }
    cubic((low + high) * 0.5, 0., 1.)
}
pub fn sample(track: &[Keyframe], phase: f32, ease: &str) -> Keyframe {
    if track.is_empty() {
        return Keyframe::default();
    }
    if phase <= track[0].at {
        return track[0];
    }
    for pair in track.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if phase <= b.at {
            let mut p = ((phase - a.at) / (b.at - a.at)).clamp(0., 1.);
            if ease == "ease-in-out" {
                p = ease_in_out(p);
            }
            let lerp = |a: f32, b: f32| a + (b - a) * p;
            return Keyframe {
                at: phase,
                x: lerp(a.x, b.x),
                y: lerp(a.y, b.y),
                rotation: lerp(a.rotation, b.rotation),
                sx: lerp(a.sx, b.sx),
                sy: lerp(a.sy, b.sy),
                clip_top: lerp(a.clip_top, b.clip_top),
                clip_bottom: lerp(a.clip_bottom, b.clip_bottom),
            };
        }
    }
    *track.last().unwrap()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_original_skier_keyframes() {
        let data = VillageData::original();
        let track = &data.frames["modvil-village__elfski"];
        for key in track {
            let actual = sample(track, key.at, "linear");
            assert!((actual.x - key.x).abs() < 0.001);
            assert!((actual.y - key.y).abs() < 0.001);
        }
        assert_eq!(track[2].x, -750.);
        assert_eq!(track[2].y, 190.);
    }
    #[test]
    fn original_css_easing_is_symmetric() {
        for i in 0..100 {
            let x = i as f32 / 100.;
            assert!((ease_in_out(x) + ease_in_out(1. - x) - 1.).abs() < 0.00001);
        }
    }
}
