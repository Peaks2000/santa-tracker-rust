//! Native renderer for the original santa-santa component (no map dependency).
use crate::{assets::Assets, ui_render::premultiplied_material};
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams},
    prelude::*,
};
use santa_tracker::{
    image_alpha::premultiply,
    tracker_santa::{self, Chimney, DIRECTIONS},
};

pub struct SantaView {
    chimney: [Chimney; 2],
    frames: [Vec<Texture2D>; 2],
    premult: Material,
    shadow: Material,
    group: Option<RenderTarget>,
    bounce_clock: tracker_santa::SleighClock,
}
impl SantaView {
    pub fn new(assets: &mut Assets) -> Result<Self, String> {
        for dir in DIRECTIONS {
            for layer in ["back", "santa", "front"] {
                assets.load(&format!("img/sleigh/sleigh-{dir}-{layer}.svg"))?;
            }
        }
        assets.load("img/sleigh/effects.svg")?;
        let load = |mode| -> Result<Chimney, String> {
            let bytes = std::fs::read(assets.root.join(format!("img/sleigh/chimney{mode}.gif")))
                .map_err(|e| e.to_string())?;
            Chimney::decode(&bytes)
        };
        let chimney = [load(0)?, load(1)?];
        let frames = std::array::from_fn(|m| {
            chimney[m]
                .frames
                .iter()
                .map(|rgba| {
                    let mut bytes = rgba.clone();
                    premultiply(&mut bytes);
                    let texture =
                        Texture2D::from_rgba8(chimney[m].width, chimney[m].height, &bytes);
                    texture.set_filter(FilterMode::Linear);
                    texture
                })
                .collect()
        });
        let shadow = load_material(ShaderSource::Glsl{vertex:r#"#version 100
attribute vec3 position;attribute vec2 texcoord;uniform mat4 Model;uniform mat4 Projection;varying vec2 uv;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;}"#,fragment:r#"#version 100
precision highp float;varying vec2 uv;uniform sampler2D Texture;uniform vec2 Texel;
void main(){float a=0.0;float total=0.0;for(int y=-3;y<=3;y++){for(int x=-3;x<=3;x++){vec2 p=vec2(float(x),float(y));float w=exp(-dot(p,p)*0.5);a+=texture2D(Texture,uv+p*Texel).a*w;total+=w;}}gl_FragColor=vec4(0.0,0.0,0.0,a/total*0.25);}"#},MaterialParams{pipeline_params:PipelineParams{color_blend:Some(BlendState::new(Equation::Add,BlendFactor::One,BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),alpha_blend:Some(BlendState::new(Equation::Add,BlendFactor::One,BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),..Default::default()},uniforms:vec![UniformDesc::new("Texel",UniformType::Float2)],..Default::default()}).map_err(|e|e.to_string())?;
        Ok(Self {
            chimney,
            frames,
            premult: premultiplied_material()?,
            shadow,
            group: None,
            bounce_clock: Default::default(),
        })
    }
    /// Separate animation clocks: showing the sleigh after a stop restarts CSS bounce.
    /// GIF clock must remain alive across hide/show (not reset per destination).
    pub fn draw(
        &mut self,
        assets: &Assets,
        center: Vec2,
        viewport_width: f64,
        heading: f64,
        stop: bool,
        bounce_ms: f64,
        gif_ms: f64,
    ) {
        let bounce_elapsed = self.bounce_clock.elapsed(bounce_ms, stop);
        let (w, h) = tracker_santa::outer_size(viewport_width);
        let (w, h) = (w as f32, h as f32);
        // Enough transparent guard pixels for the group drop-shadow, which must
        // operate on the composited silhouette, not separately on each layer.
        let size = vec2(w + 32., h + 48.);
        if self
            .group
            .as_ref()
            .is_none_or(|g| g.texture.width() != size.x * 2. || g.texture.height() != size.y * 2.)
        {
            self.group = Some(render_target((size.x * 2.) as u32, (size.y * 2.) as u32));
            self.group
                .as_ref()
                .unwrap()
                .texture
                .set_filter(FilterMode::Linear);
        }
        let group = self.group.as_ref().unwrap();
        push_camera_state();
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., size.x, size.y));
        camera.render_target = Some(group.clone());
        set_camera(&camera);
        clear_background(BLANK);
        gl_use_material(&self.premult);
        let origin = vec2(16., 16.);
        if stop {
            if let Some(mode) = tracker_santa::chimney_mode(heading) {
                let t = &self.frames[mode][self.chimney[mode].frame_at(gif_ms)];
                // background-size:contain inside a 95x83 box (GIF 383x333).
                let scale = (95. / t.width()).min(83. / t.height());
                let s = vec2(t.width(), t.height()) * scale;
                draw_texture_ex(
                    t,
                    origin.x + (w - s.x) / 2.,
                    origin.y + (h - s.y) / 2.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(s),
                        ..Default::default()
                    },
                );
            }
        } else {
            let (santa_y, other_y) = tracker_santa::bounce(bounce_elapsed);
            if let Some(dir) = tracker_santa::direction(heading) {
                for (layer, y) in [("back", other_y), ("santa", santa_y), ("front", other_y)] {
                    let t = assets.texture(&format!("img/sleigh/sleigh-{dir}-{layer}.svg"));
                    let s = vec2(w * 0.85, w * 0.85 * t.height() / t.width());
                    draw_texture_ex(
                        t,
                        origin.x + (w - s.x) / 2.,
                        origin.y + (h - s.y) / 2. + y as f32,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(s),
                            ..Default::default()
                        },
                    );
                    if layer == "santa" {
                        self.effects(assets, origin + vec2(0., y as f32), w, h);
                    }
                }
            } else {
                // A missing directional background does not remove ::after.
                self.effects(assets, origin + vec2(0., santa_y as f32), w, h);
            }
        }
        gl_use_default_material();
        pop_camera_state();
        let dest = center - vec2(w, h) / 2. - origin;
        self.shadow
            .set_uniform("Texel", vec2(1. / size.x, 1. / size.y));
        gl_use_material(&self.shadow);
        let offset = if stop {
            vec2(0., 1.)
        } else if viewport_width >= 768. {
            vec2(3., 6.)
        } else {
            vec2(2., 4.)
        };
        draw_texture_ex(
            &group.texture,
            dest.x + offset.x,
            dest.y + offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_material(&self.premult);
        draw_texture_ex(
            &group.texture,
            dest.x,
            dest.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
    fn effects(&self, assets: &Assets, origin: Vec2, w: f32, h: f32) {
        let texture = assets.texture("img/sleigh/effects.svg");
        // CSS background has auto size: viewBox-only SVG defaults to its square
        // background positioning area, rather than a fixed 165px image. Crop to
        // the pseudo-element's box (backgrounds never overflow that box).
        let bh = h * 0.52;
        let side = w.min(bh);
        draw_texture_ex(
            texture,
            origin.x + (w - side) / 2.,
            origin.y + h * 0.66,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(side, side)),
                ..Default::default()
            },
        );
    }
}
