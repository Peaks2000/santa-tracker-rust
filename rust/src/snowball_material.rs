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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

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
