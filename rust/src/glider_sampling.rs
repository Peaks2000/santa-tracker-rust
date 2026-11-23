//! Integer SVG backing scales for Glider's original maximum desktop viewports.
//! Logical layout/crops stay independent; oversized repeating scenery needs tiles.
use std::{collections::HashMap, sync::LazyLock};
#[derive(serde::Deserialize)]
struct Policy {
    max_scale: f32,
    collision: Collision,
    boxes: HashMap<String, [Option<f32>; 2]>,
}
#[derive(serde::Deserialize)]
struct Collision {
    source: String,
    crop: [f32; 4],
    background: [f32; 2],
    peak_scale: f32,
}
pub const COLLISION_KEY: &str = "native/glider/collision.svg";
pub fn collision_source() -> &'static str {
    &POLICY.collision.source
}
pub fn collision_background() -> [f32; 2] {
    POLICY.collision.background
}
pub fn collision_crop() -> [f32; 4] {
    POLICY.collision.crop
}
pub fn collision_peak_scale() -> f32 {
    POLICY.collision.peak_scale
}
static POLICY: LazyLock<Policy> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/glider-svg-sampling.json")).unwrap()
});
/// Convert actual scene scale/display density to the maximum-viewport policy's
/// multiplier. Increasing window width or display density can request an upgrade.
pub fn density_for_viewport(width: f32, density: f32) -> f32 {
    let (_, _, em) = crate::glider::Glider::dimensions(width as f64, 800.);
    density * (em / 10.) as f32 / POLICY.max_scale
}
pub fn selected(key: &str) -> bool {
    if key == COLLISION_KEY {
        return true;
    }
    key.strip_prefix("scenes/glider/img/")
        .is_some_and(|name| POLICY.boxes.contains_key(name))
}
pub fn fitted_maximum(key: &str, natural: [f32; 2]) -> Option<[f32; 2]> {
    if key == COLLISION_KEY {
        return Some(natural.map(|n| n * POLICY.collision.peak_scale * POLICY.max_scale));
    }
    let name = key.strip_prefix("scenes/glider/img/")?;
    let bounds = POLICY.boxes.get(name)?;
    let fit = bounds[0]
        .map_or(f32::INFINITY, |w| w / natural[0])
        .min(bounds[1].map_or(f32::INFINITY, |h| h / natural[1]));
    Some(natural.map(|n| n * fit * POLICY.max_scale))
}
pub fn scale(key: &str, natural: [f32; 2], density: f32, baseline: f32) -> f32 {
    let Some(fitted) = fitted_maximum(key, natural) else {
        return baseline;
    };
    // Integer multiples preserve the existing bitmap aspect/crop coordinates.
    // Existing atlas3x is retained when it already covers the requested density.
    baseline.max((fitted[0] / natural[0] * density).ceil())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collision_crop_scale_covers_source_peak_without_growing_the_full_atlas() {
        assert_eq!(POLICY.collision.background, [400., 600.]);
        assert_eq!(collision_crop(), [210., 400., 90., 100.]);
        assert_eq!(collision_peak_scale(), 3.);
        assert!(selected(COLLISION_KEY));
        for density in [0.25, 0.5, 1., 1.5, 2., 3.] {
            let factor = scale(COLLISION_KEY, [90., 100.], density, 3.);
            assert!(factor >= 3.75 * density);
            let atlas_factor = scale(
                "scenes/glider/img/glider-sprite.svg",
                [400., 600.],
                density,
                3.,
            );
            assert!(factor >= atlas_factor);
        }
    }
    #[test]
    fn viewport_sampling_multiplier_matches_actual_source_methods() {
        let source: serde_json::Value =
            serde_json::from_str(include_str!("../data/glider-svg-sampling.json")).unwrap();
        for case in source["viewport_cases"].as_array().unwrap() {
            let actual = density_for_viewport(
                case["width"].as_f64().unwrap() as f32,
                case["density"].as_f64().unwrap() as f32,
            );
            assert!(
                (actual as f64 - case["effective_density"].as_f64().unwrap()).abs() < 0.000001,
                "{case}: {actual}"
            );
        }
    }
    #[test]
    fn selected_original_viewports_are_covered_without_changing_aspect() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../static/scenes/glider/img");
        for name in POLICY.boxes.keys() {
            let key = format!("scenes/glider/img/{name}");
            let data = std::fs::read(root.join(name)).unwrap();
            let tree = resvg::usvg::Tree::from_data(&data, &Default::default()).unwrap();
            let natural = [tree.size().width(), tree.size().height()];
            assert!(
                natural.iter().all(|v| v.fract() == 0.),
                "review fractional source viewBox {name}"
            );
            let fitted = fitted_maximum(&key, natural).unwrap();
            for density in [0.25, 0.5, 0.75, 1., 1.25, 1.5, 2., 3.] {
                let baseline = if name == "glider-sprite.svg" { 3. } else { 1. };
                let factor = scale(&key, natural, density, baseline);
                assert_eq!(factor.fract(), 0.);
                assert!(factor >= baseline);
                for axis in 0..2 {
                    assert!(
                        natural[axis] * factor + 0.001 >= fitted[axis] * density,
                        "{key} axis{axis}"
                    );
                }
                assert_eq!(
                    (natural[0] * factor) / (natural[1] * factor),
                    natural[0] / natural[1]
                );
            }
        }
        for key in [
            "scenes/glider/img/bg-mountains.svg",
            "scenes/glider/img/stars-tile.svg",
            "scenes/glider/img/logo.svg",
            "scenes/boatload/img/boat1.svg",
        ] {
            assert!(!selected(key));
            assert_eq!(scale(key, [100., 100.], 3., 1.), 1.);
        }
    }
}
