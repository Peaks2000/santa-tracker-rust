//! Shared production scene compositing and disabled-state filter.
use macroquad::prelude::*;
use santa_tracker::ui::{Ease, Transition};
const VERTEX: &str = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;uniform mat4 Model;uniform mat4 Projection;varying vec2 uv;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;}"#;
const FRAGMENT: &str = r#"#version 100
precision highp float;varying vec2 uv;uniform sampler2D Texture;uniform vec2 Texel;uniform float Amount;
void main(){vec3 sum=vec3(0.0);float total=0.0;for(int x=-2;x<=2;x++){for(int y=-2;y<=2;y++){float weight=exp(-float(x*x+y*y)*0.5);sum+=texture2D(Texture,uv+vec2(float(x),float(y))*Texel*4.0*Amount).rgb*weight;total+=weight;}}vec3 c=sum/total;float gray=dot(c,vec3(0.2126,0.7152,0.0722));c=mix(c,vec3(gray),0.7*Amount)*(1.0-0.5*Amount);gl_FragColor=vec4(c,1.0);}"#;
pub struct SceneFilter {
    target: Option<RenderTarget>,
    filtered: Option<RenderTarget>,
    material: Material,
    opaque_copy: Material,
    amount: Transition<1>,
    cached: f32,
}
impl SceneFilter {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            target: None,
            filtered: None,
            // The scene is opaque, but Macroquad's default sprite blending
            // also blends alpha with SourceAlpha. Edge texels therefore leave
            // alpha below one in the offscreen target. Applying that alpha a
            // second time at presentation produces a dark outline. Copy RGB
            // opaquely, just as an opaque browser scene is composited.
            opaque_copy: load_material(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: r#"#version 100
precision highp float;varying vec2 uv;uniform sampler2D Texture;
void main(){gl_FragColor=vec4(texture2D(Texture,uv).rgb,1.0);}"#,
                },
                MaterialParams::default(),
            )
            .map_err(|e| e.to_string())?,
            material: load_material(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: FRAGMENT,
                },
                MaterialParams {
                    uniforms: vec![
                        UniformDesc::new("Texel", UniformType::Float2),
                        UniformDesc::new("Amount", UniformType::Float1),
                    ],
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?,
            amount: Transition::new([0.]),
            cached: -1.,
        })
    }
    pub fn begin(&mut self) {
        let w = screen_width() as u32;
        let h = screen_height() as u32;
        if self
            .target
            .as_ref()
            .is_none_or(|t| t.texture.width() != w as f32 || t.texture.height() != h as f32)
        {
            self.target = Some(render_target_ex(
                w,
                h,
                RenderTargetParams {
                    depth: true,
                    ..Default::default()
                },
            ));
            self.filtered = Some(render_target(w, h));
            self.cached = -1.;
        }
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., w as f32, h as f32));
        camera.render_target = self.target.clone();
        set_camera(&camera);
        clear_background(BLACK);
    }
    pub fn finish(&mut self, disabled: bool) {
        let now = get_time();
        self.amount.set(
            now,
            [f64::from(disabled)],
            if disabled { 0.75 } else { 0.33 },
            Ease::Default,
        );
        let amount = self.amount.value(now)[0] as f32;
        let source = &self.target.as_ref().unwrap().texture;
        let filtered = self.filtered.as_ref().unwrap();
        if amount > 0.001 && (!disabled || amount < 1. || self.cached != amount) {
            let mut camera =
                Camera2D::from_display_rect(Rect::new(0., 0., screen_width(), screen_height()));
            camera.render_target = Some(filtered.clone());
            set_camera(&camera);
            self.material
                .set_uniform("Texel", vec2(1. / screen_width(), 1. / screen_height()));
            self.material.set_uniform("Amount", amount);
            gl_use_material(&self.material);
            draw_texture_ex(
                source,
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    flip_y: true,
                    ..Default::default()
                },
            );
            gl_use_default_material();
            self.cached = amount;
        }
        set_default_camera();
        let scale = 1. + amount * 0.05;
        let size = vec2(screen_width(), screen_height()) * scale;
        gl_use_material(&self.opaque_copy);
        draw_texture_ex(
            if amount > 0.001 {
                &filtered.texture
            } else {
                source
            },
            (screen_width() - size.x) / 2.,
            (screen_height() - size.y) / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
}
