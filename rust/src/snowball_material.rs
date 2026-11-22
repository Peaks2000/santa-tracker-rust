//! Snowball's material shader configuration differs from Elf Ski's renderer.
//! Both glTF base-color maps are sRGB, but only Elf Ski enables gamma output.

const SKI_FRAGMENT: &str = include_str!("../assets/shaders/ski-standard.frag");
const SKI_OUTPUT: &str = "vec4 linearToOutputTexel( vec4 value ) { return LinearToGamma( value, float( GAMMA_FACTOR ) ); }";
const SNOWBALL_OUTPUT: &str =
    "vec4 linearToOutputTexel( vec4 value ) { return LinearToLinear( value ); }";

pub fn elf_fragment() -> String {
    // Preserve source sRGB texture decoding and the shared lighting code.
    // Snowball RenderSystem uses the renderer's gammaOutput=false default.
    assert_eq!(SKI_FRAGMENT.matches(SKI_OUTPUT).count(), 1);
    SKI_FRAGMENT
        .replace("#define USE_MAP", "#define USE_MAP\n#define USE_COLOR")
        .replace(SKI_OUTPUT, SNOWBALL_OUTPUT)
}

use macroquad::{
    miniquad::{
        BlendFactor, BlendState, BlendValue, Comparison, CullFace, Equation, FrontFaceOrder,
    },
    prelude::PipelineParams,
};

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Elf,
    Tiles,
    Obstacles,
    Gift,
    Canopy,
    PlayerRing,
    ArrivalRing,
    Destination,
    Snowball,
    Iceberg,
    Trail,
    Impact,
}
pub fn front_only(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Elf
            | Kind::Tiles
            | Kind::Obstacles
            | Kind::Gift
            | Kind::Canopy
            | Kind::PlayerRing
            | Kind::ArrivalRing
    )
}
pub fn depth_test(kind: Kind) -> bool {
    !matches!(
        kind,
        Kind::ArrivalRing | Kind::Destination | Kind::Trail | Kind::Impact
    )
}
/// Source straight-alpha transparent materials; additive ice uses the same
/// SourceAlpha/One factors for RGB AND alpha. A disabled depth test also means
/// no depth writes in GL, even when the source material's depthWrite is true.
pub fn pipeline(kind: Kind) -> PipelineParams {
    let additive = matches!(kind, Kind::Iceberg);
    PipelineParams {
        cull_face: if front_only(kind) {
            CullFace::Back
        } else {
            CullFace::Nothing
        },
        front_face_order: FrontFaceOrder::CounterClockwise,
        depth_test: if depth_test(kind) {
            Comparison::LessOrEqual
        } else {
            Comparison::Always
        },
        depth_write: depth_test(kind),
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            if additive {
                BlendFactor::One
            } else {
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha)
            },
        )),
        alpha_blend: Some(BlendState::new(
            Equation::Add,
            if additive {
                BlendFactor::Value(BlendValue::SourceAlpha)
            } else {
                BlendFactor::One
            },
            if additive {
                BlendFactor::One
            } else {
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha)
            },
        )),
        ..Default::default()
    }
}
// Native vertex order TL, TR, BR, BL; source PlaneBufferGeometry is TL, TR, BL, BR.
pub const PLANE_CORNERS: [[f32; 2]; 4] = [[-0.5, 0.5], [0.5, 0.5], [0.5, -0.5], [-0.5, -0.5]];
pub const PLANE_UV: [[f32; 2]; 4] = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
pub fn plane_indices(base: u16) -> [u16; 6] {
    [base, base + 3, base + 1, base + 3, base + 2, base + 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn kind(name: &str) -> Kind {
        match name {
            "elf" => Kind::Elf,
            "tiles" => Kind::Tiles,
            "obstacles" => Kind::Obstacles,
            "gift" => Kind::Gift,
            "canopy" => Kind::Canopy,
            "player_ring" => Kind::PlayerRing,
            "arrival_ring" => Kind::ArrivalRing,
            "destination" => Kind::Destination,
            "snowball" => Kind::Snowball,
            "iceberg" => Kind::Iceberg,
            "trail" => Kind::Trail,
            "impact" => Kind::Impact,
            _ => panic!("unknown material {name}"),
        }
    }
    fn factor(gl: u64) -> BlendFactor {
        match gl {
            1 => BlendFactor::One,
            770 => BlendFactor::Value(BlendValue::SourceAlpha),
            771 => BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            _ => panic!("unsupported source blend factor {gl}"),
        }
    }
    #[test]
    fn production_pipelines_match_original_materials_and_backend_functions() {
        let reference: Value = serde_json::from_str(include_str!(
            "../data/snowball-material-state-reference.json"
        ))
        .unwrap();
        for row in reference["rows"].as_array().unwrap() {
            let name = row["name"].as_str().unwrap();
            let actual = pipeline(kind(name));
            let state = &row["backend"];
            assert_eq!(
                actual.cull_face,
                if state["cull"].as_bool().unwrap() {
                    CullFace::Back
                } else {
                    CullFace::Nothing
                },
                "{name}"
            );
            assert_eq!(actual.front_face_order, FrontFaceOrder::CounterClockwise);
            assert_eq!(state["front"], 2305, "{name}");
            assert_eq!(
                actual.depth_test,
                if state["depthTest"].as_bool().unwrap() {
                    Comparison::LessOrEqual
                } else {
                    Comparison::Always
                },
                "{name}"
            );
            assert_eq!(state["depthFunc"], 3, "{name}");
            assert_eq!(
                actual.depth_write,
                state["depthTest"].as_bool().unwrap() && state["depthWrite"].as_bool().unwrap(),
                "{name}"
            );
            assert_eq!(state["blend"], true, "{name}");
            assert_eq!(state["equation"], 32774, "{name}");
            let factors = state["factors"].as_array().unwrap();
            assert_eq!(
                actual.color_blend,
                Some(BlendState::new(
                    Equation::Add,
                    factor(factors[0].as_u64().unwrap()),
                    factor(factors[1].as_u64().unwrap())
                )),
                "{name}"
            );
            assert_eq!(
                actual.alpha_blend,
                Some(BlendState::new(
                    Equation::Add,
                    factor(factors[2].as_u64().unwrap()),
                    factor(factors[3].as_u64().unwrap())
                )),
                "{name}"
            );
        }
    }
    #[test]
    fn native_plane_triangles_match_original_three_topology() {
        let reference: Value = serde_json::from_str(include_str!(
            "../data/snowball-material-state-reference.json"
        ))
        .unwrap();
        let triangles = reference["plane"]["triangles"].as_array().unwrap();
        for base in [0, 16] {
            for (index, expected) in plane_indices(base).into_iter().zip(triangles) {
                let index = (index - base) as usize;
                let position = [
                    PLANE_CORNERS[index][0] as f64,
                    PLANE_CORNERS[index][1] as f64,
                    0.,
                ];
                let uv = PLANE_UV[index].map(f64::from);
                for (values, key) in [(position.as_slice(), "position"), (uv.as_slice(), "uv")] {
                    for (value, expected) in values.iter().zip(expected[key].as_array().unwrap()) {
                        assert_eq!(*value, expected.as_f64().unwrap());
                    }
                }
            }
        }
    }

    #[test]
    fn scene_output_encodings_match_original_three_program_parameters() {
        let reference: Value =
            serde_json::from_str(include_str!("../data/snowball-encoding-reference.json")).unwrap();
        let snowball = elf_fragment();
        for case in reference["cases"].as_array().unwrap() {
            let output = case["outputFunction"].as_str().unwrap();
            let map = case["mapFunction"].as_str().unwrap();
            let shader = match case["name"].as_str().unwrap() {
                "snowball" => snowball.as_str(),
                "elfski" => SKI_FRAGMENT,
                other => panic!("unknown scene {other}"),
            };
            assert_eq!(shader.matches(output).count(), 1, "{}", case["name"]);
            assert_eq!(shader.matches(map).count(), 1, "{}", case["name"]);
        }
        assert!(!snowball.contains(SKI_OUTPUT));
        assert!(!SKI_FRAGMENT.contains(SNOWBALL_OUTPUT));
    }
}
