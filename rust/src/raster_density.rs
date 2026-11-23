//! Physical sampling dimensions for dynamic vector artwork. Layout remains in
//! logical pixels; integer bitmap dimensions must cover the physical extent.
pub fn size(width: f32, height: f32, density: f32) -> (u32, u32) {
    (
        (width * density).ceil().max(1.) as u32,
        (height * density).ceil().max(1.) as u32,
    )
}
/// One logical pixel in UV coordinates; independent of backing bitmap density.
pub fn logical_texel(width: f32, height: f32) -> (f32, f32) {
    (1. / width, 1. / height)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractional_and_high_density_backing_covers_the_same_logical_rectangle() {
        for (w, h, d, expected) in [
            (450., 630., 1., (450, 630)),
            (450., 630., 2., (900, 1260)),
            (450.25, 630.5, 1.25, (563, 789)),
            (120., 140., 2., (240, 280)),
            (120., 140., 1.5, (180, 210)),
        ] {
            let actual = size(w, h, d);
            assert_eq!(actual, expected);
            assert!(actual.0 as f32 >= w * d && actual.1 as f32 >= h * d);
            assert!((actual.0 as f32 - w * d) < 1. && (actual.1 as f32 - h * d) < 1.);
            let step = logical_texel(w, h);
            // Gaussian tap offsets represent the same CSS distances whether
            // sampling a 1x or high-density backing bitmap.
            assert!((step.0 * w - 1.).abs() < 1e-6 && (step.1 * h - 1.).abs() < 1e-6);
        }
    }
}
