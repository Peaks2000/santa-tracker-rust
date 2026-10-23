//! Native CSS geometry: antialiased rounded shapes and animated box shadows.
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};
use santa_tracker::ui::{Ease, Transition};

const VERTEX: &str = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;attribute vec4 color0;
uniform mat4 Model;uniform mat4 Projection;varying vec2 uv;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;}
"#;
const FRAGMENT: &str = r#"#version 100
precision highp float;
varying vec2 uv;uniform vec2 Size;uniform float Radius;uniform float Padding;uniform float Blur;uniform vec4 Tint;
void main(){vec2 p=(uv-0.5)*(Size+2.0*Padding);vec2 q=abs(p)-Size*0.5+Radius;
float distance=length(max(q,vec2(0.0)))+min(max(q.x,q.y),0.0)-Radius;
float alpha=1.0-smoothstep(-0.5,0.5,distance);
if(Blur>0.0){float sigma=max(Blur*0.5,0.001);float z=distance/sigma;
float v=abs(z);float t=1.0/(1.0+0.2316419*v);float tail=0.39894228*exp(-0.5*v*v)*t*(0.31938153+t*(-0.35656378+t*(1.78147794+t*(-1.82125598+t*1.33027443))));
alpha=z>0.0?tail:1.0-tail;}
gl_FragColor=vec4(Tint.rgb,Tint.a*alpha);}
"#;
pub struct Shapes {
    material: Material,
    image_material: Material,
}
impl Shapes {
    pub fn new() -> Result<Self, String> {
        let uniforms = vec![
            UniformDesc::new("Size", UniformType::Float2),
            UniformDesc::new("Radius", UniformType::Float1),
            UniformDesc::new("Padding", UniformType::Float1),
            UniformDesc::new("Blur", UniformType::Float1),
            UniformDesc::new("Tint", UniformType::Float4),
        ];
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            MaterialParams {
                uniforms,
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    alpha_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::One,
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())?;
        let image_material=load_material(ShaderSource::Glsl {vertex:r#"#version 100
attribute vec3 position;attribute vec2 texcoord;uniform mat4 Model;uniform mat4 Projection;varying vec2 uv;varying vec2 point;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;point=position.xy;}"#,fragment:r#"#version 100
precision highp float;varying vec2 uv;varying vec2 point;uniform sampler2D Texture;uniform vec4 Bounds;uniform float Radius;uniform float Opacity;uniform float Gradient;
void main(){vec2 q=abs(point-Bounds.xy-Bounds.zw*0.5)-Bounds.zw*0.5+Radius;float d=length(max(q,vec2(0.0)))+min(max(q.x,q.y),0.0)-Radius;vec4 c=texture2D(Texture,uv);float y=(point.y-Bounds.y)/Bounds.w;c.rgb*=1.0-Gradient*0.33*clamp((y-0.3)/0.4,0.0,1.0);c.a*=Opacity*(1.0-smoothstep(-0.5,0.5,d));gl_FragColor=c;}"#},MaterialParams {uniforms:vec![UniformDesc::new("Bounds",UniformType::Float4),UniformDesc::new("Radius",UniformType::Float1),UniformDesc::new("Opacity",UniformType::Float1),UniformDesc::new("Gradient",UniformType::Float1)],pipeline_params:PipelineParams {color_blend:Some(BlendState::new(Equation::Add,BlendFactor::Value(BlendValue::SourceAlpha),BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),..Default::default()},..Default::default()}).map_err(|e|e.to_string())?;
        Ok(Self {
            material,
            image_material,
        })
    }
    pub fn rounded(&self, r: Rect, radius: f32, c: Color, blur: f32) {
        if r.w <= 0. || r.h <= 0. || c.a <= 0. {
            return;
        }
        let padding = (blur * 2.).max(1.);
        self.material.set_uniform("Size", r.size());
        self.material
            .set_uniform("Radius", radius.min(r.w / 2.).min(r.h / 2.));
        self.material.set_uniform("Padding", padding);
        self.material.set_uniform("Blur", blur);
        self.material.set_uniform("Tint", vec4(c.r, c.g, c.b, c.a));
        gl_use_material(&self.material);
        draw_rectangle(
            r.x - padding,
            r.y - padding,
            r.w + 2. * padding,
            r.h + 2. * padding,
            WHITE,
        );
        gl_use_default_material();
    }
    pub fn cover(&self, texture: &Texture2D, r: Rect, radius: f32, opacity: f32, gradient: bool) {
        self.image_material
            .set_uniform("Bounds", vec4(r.x, r.y, r.w, r.h));
        self.image_material.set_uniform("Radius", radius);
        self.image_material.set_uniform("Opacity", opacity);
        self.image_material
            .set_uniform("Gradient", if gradient { 1f32 } else { 0f32 });
        let scale = (r.w / texture.width()).max(r.h / texture.height());
        let w = r.w / scale;
        let h = r.h / scale;
        gl_use_material(&self.image_material);
        draw_texture_ex(
            texture,
            r.x,
            r.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(r.size()),
                source: Some(Rect::new(
                    (texture.width() - w) / 2.,
                    (texture.height() - h) / 2.,
                    w,
                    h,
                )),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
    pub fn circle(&self, center: Vec2, radius: f32, c: Color, blur: f32) {
        self.rounded(
            Rect::new(
                center.x - radius,
                center.y - radius,
                radius * 2.,
                radius * 2.,
            ),
            radius,
            c,
            blur,
        );
    }
}
pub struct Button {
    pub press: Transition<1>,
    pub color: Transition<3>,
    pub hover: bool,
    pub captured: bool,
}
#[derive(Default)]
pub struct Interaction {
    pub entered: bool,
    pub pressed: bool,
    pub clicked: bool,
}
impl Button {
    pub fn new(color: Color) -> Self {
        Self {
            press: Transition::new([0.]),
            color: Transition::new([color.r as f64, color.g as f64, color.b as f64]),
            hover: false,
            captured: false,
        }
    }
    pub fn update(
        &mut self,
        rect: Rect,
        now: f64,
        enabled: bool,
        keyboard_down: bool,
        keyboard_click: bool,
    ) -> Interaction {
        let hover = enabled && rect.contains(Vec2::from(mouse_position()));
        let entered = hover && !self.hover;
        self.hover = hover;
        let pressed = enabled && hover && is_mouse_button_pressed(MouseButton::Left);
        if pressed {
            self.captured = true;
        }
        let clicked =
            enabled && self.captured && hover && is_mouse_button_released(MouseButton::Left)
                || keyboard_click;
        if is_mouse_button_released(MouseButton::Left) || !enabled {
            self.captured = false;
        }
        let down =
            enabled && (self.captured && is_mouse_button_down(MouseButton::Left) || keyboard_down);
        self.press.set(
            now,
            [f64::from(down)],
            if down { 0.05 } else { 0.25 },
            Ease::In,
        );
        Interaction {
            entered,
            pressed,
            clicked,
        }
    }
    pub fn theme(&mut self, color: Color, now: f64) {
        self.color.set(
            now,
            [color.r as f64, color.g as f64, color.b as f64],
            0.25,
            Ease::In,
        );
    }
    pub fn draw(&self, shapes: &Shapes, rect: Rect, now: f64, focused: bool, opacity: f32) -> Vec2 {
        let p = self.press.value(now)[0] as f32;
        let colors = self.color.value(now);
        let color = Color::new(
            colors[0] as f32,
            colors[1] as f32,
            colors[2] as f32,
            opacity,
        );
        let delta = vec2(-3., -5.) * (1. - p);
        let center = rect.center() + delta;
        let radius = rect.w / 2.;
        let mut shadow = Color::new(0., 0., 0., (0.24 - 0.08 * p) * opacity);
        shapes.circle(
            center + vec2(8., 7.) * (1. - p),
            radius + 2. * (1. - p),
            shadow,
            p,
        );
        shapes.circle(center + vec2(3., 5.) * (1. - p), radius, color, 0.);
        shadow.a = 0.3 * opacity;
        shapes.circle(center + vec2(3., 5.) * (1. - p), radius, shadow, 0.);
        shapes.circle(center, radius, color, 0.);
        if focused {
            shapes.circle(center, radius, Color::new(0., 0., 0., 0.25 * opacity), 0.);
            shapes.circle(center, radius - 2., color, 0.);
        }
        delta
    }
}

/// Browser-style interpolation for native Lottie surfaces. RGB is already
/// multiplied by alpha before filtering; apply opacity once, then source-over.
pub fn premultiplied_material() -> Result<Material, String> {
    load_material(ShaderSource::Glsl {vertex:r#"#version 100
attribute vec3 position;attribute vec2 texcoord;attribute vec4 color0;uniform mat4 Model;uniform mat4 Projection;varying vec2 uv;varying vec4 tint;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;tint=color0/255.0;}"#,fragment:r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;uniform sampler2D Texture;
void main(){vec4 c=texture2D(Texture,uv);gl_FragColor=vec4(c.rgb*tint.rgb*tint.a,c.a*tint.a);}"#},MaterialParams {pipeline_params:PipelineParams {color_blend:Some(BlendState::new(Equation::Add,BlendFactor::One,BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),alpha_blend:Some(BlendState::new(Equation::Add,BlendFactor::One,BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),..Default::default()},..Default::default()}).map_err(|e|e.to_string())
}

/// The native window is opaque, like the original page. Default sprite/font
/// blending can lower destination alpha; don't let a desktop compositor or PNG
/// reader apply that alpha to already-composited scene RGB a second time.
pub struct OpaqueSurface {
    material: Material,
}
impl OpaqueSurface {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            material: load_material(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: r#"#version 100
precision lowp float;void main(){gl_FragColor=vec4(0.0,0.0,0.0,1.0);}"#,
                },
                MaterialParams {
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::Zero,
                            BlendFactor::One,
                        )),
                        alpha_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::One,
                            BlendFactor::Zero,
                        )),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?,
        })
    }
    pub fn draw(&self) {
        gl_use_material(&self.material);
        draw_rectangle(0., 0., screen_width(), screen_height(), WHITE);
        gl_use_default_material();
    }
}
