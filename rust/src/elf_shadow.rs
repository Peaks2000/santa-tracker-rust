//! GPU equivalent of the original CSS drop-shadow(4px 4px 2px).
//! Keep the original SVG geometry and motion; avoid a CPU filter every frame.
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams},
    prelude::*,
};
pub struct ElfShadow {
    material: Material,
}
impl ElfShadow {
    pub fn new() -> Result<Self, String> {
        let material=load_material(ShaderSource::Glsl{vertex:r#"#version 100
attribute vec3 position;attribute vec2 texcoord;uniform mat4 Model;uniform mat4 Projection;varying vec2 uv;void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;}
"#,fragment:r#"#version 100
precision highp float;varying vec2 uv;uniform sampler2D Texture;uniform vec2 Texel;
void main(){float a=0.0;for(int y=-6;y<=6;y++){for(int x=-6;x<=6;x++){vec2 p=vec2(float(x),float(y));float w=exp(-dot(p,p)*0.125);a+=texture2D(Texture,uv+p*Texel).a*w;}}gl_FragColor=vec4(0.0,0.0,0.0,a*(0.125/25.08129));}
"#},MaterialParams{pipeline_params:PipelineParams{color_blend:Some(BlendState::new(Equation::Add,BlendFactor::One,BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),alpha_blend:Some(BlendState::new(Equation::Add,BlendFactor::One,BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),..Default::default()},uniforms:vec![UniformDesc::new("Texel",UniformType::Float2)],..Default::default()}).map_err(|e|e.to_string())?;
        Ok(Self { material })
    }
    pub fn draw(&mut self, texture: &Texture2D, r: Rect, _premult: &Material) {
        self.material
            .set_uniform("Texel", vec2(1. / texture.width(), 1. / texture.height()));
        gl_use_material(&self.material);
        draw_texture_ex(
            texture,
            r.x + 4.,
            r.y + 4.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(r.size()),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
}
