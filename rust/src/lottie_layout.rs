//! Selected xMidYMid meet/slice branch of bundled CanvasRenderer resize.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Fit {
    Meet,
    Slice,
}
impl Fit {
    pub fn for_village_asset(key: &str) -> Self {
        if key.starts_with("scenes/modvil/img/modules/") {
            Self::Slice
        } else {
            Self::Meet
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Layout {
    pub width: u32,
    pub height: u32,
    pub sx: f64,
    pub sy: f64,
    pub tx: f64,
    pub ty: f64,
}
pub fn layout(intrinsic: [f64; 2], wrapper: [f64; 2], density: f64, fit: Fit) -> Layout {
    let [w, h] = intrinsic;
    let [ew, eh] = wrapper;
    let ar = w / h;
    let er = ew / eh;
    let meet = matches!(fit, Fit::Meet);
    let scale = if ar > er && meet || ar < er && !meet {
        ew / (w / density)
    } else {
        eh / (h / density)
    };
    let tx = if ar < er && meet || ar > er && !meet {
        (ew - w * (eh / h)) / 2. * density
    } else {
        0.
    };
    let ty = if ar > er && meet || ar < er && !meet {
        ((eh - h * (ew / w)) / 2.) * density
    } else {
        0.
    };
    Layout {
        width: (ew * density) as u32,
        height: (eh * density) as u32,
        sx: scale,
        sy: scale,
        tx,
        ty,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_canvas_renderer_resize_matrices_and_backing_dimensions_match() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/lottie-layout-reference.json")).unwrap();
        for case in data["cases"].as_array().unwrap() {
            let i = &case["input"];
            let actual = layout(
                serde_json::from_value(i["intrinsic"].clone()).unwrap(),
                serde_json::from_value(i["wrapper"].clone()).unwrap(),
                i["density"].as_f64().unwrap(),
                serde_json::from_value(i["fit"].clone()).unwrap(),
            );
            let expected: Layout = serde_json::from_value(case["output"].clone()).unwrap();
            assert_eq!(
                (actual.width, actual.height),
                (expected.width, expected.height)
            );
            for (a, b) in [
                (actual.sx, expected.sx),
                (actual.sy, expected.sy),
                (actual.tx, expected.tx),
                (actual.ty, expected.ty),
            ] {
                assert!((a - b).abs() < 1e-10, "{i}: {a} != {b}");
            }
        }
        assert!(matches!(
            Fit::for_village_asset("scenes/modvil/img/modules/gym/1.json"),
            Fit::Slice
        ));
        assert!(matches!(
            Fit::for_village_asset("img/card/jetpack.json"),
            Fit::Meet
        ));
    }
}
