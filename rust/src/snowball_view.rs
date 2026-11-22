//! Original Snowball Storm art, native hex meshes, skinned elves and splash.
use crate::{assets::Assets, snowball_model::ElfModel};
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Comparison, Equation},
    prelude::*,
};
use santa_tracker::snowball::{Phase, Snowball, V};
use santa_tracker::snowball_collision::Id;
use santa_tracker::snowball_parachute::Canopy;
use santa_tracker::snowball_removal::{Ice, Parent};
use serde_json::Value;
use std::{collections::HashMap, f32::consts::PI};
const VERT: &str = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;attribute vec4 color0;attribute vec4 normal;varying vec2 uv;varying vec4 tint;varying vec4 data;
void main(){gl_Position=vec4(position,1.0);uv=texcoord;tint=color0/255.0;data=normal;}"#;
const FRAG: &str = r#"#version 100
precision highp float;uniform sampler2D Texture;uniform vec3 diffuse;uniform float opacity;varying vec2 uv;varying vec4 tint;varying vec4 data;
void main(){if(data.z==-1.){gl_FragColor=vec4(diffuse,opacity);return;}vec4 c=texture2D(Texture,uv)*tint;if(data.z==-2.||data.z==-3.)c.a*=data.w;if(data.z!=-2.&&c.a<0.1)discard;gl_FragColor=c;}"#;
pub struct SnowballView {
    model: Option<ElfModel>,
    shapes: crate::ui_render::Shapes,
    clip: Material,
    marker_mesh: Mesh,
    destination_mesh: Mesh,
    tile: Material,
    particle: Material,
    iceberg: Material,
    projectile: Material,
    lit: Material,
    textures: HashMap<(usize, usize), Texture2D>,
    hex: Mesh,
    present: Mesh,
    gifts: Vec<Texture2D>,
    pub pressed: bool,
    button_down: bool,
    powerup_slots: [santa_tracker::ui::Transition<3>; 2],
    powerup_icons: [[santa_tracker::ui::Transition<4>; 3]; 2],
}
fn geometry(data: &str) -> Mesh {
    let v: Value = serde_json::from_str(data).unwrap();
    let p = v["position"].as_array().unwrap();
    let uv = v["uv"].as_array();
    Mesh {
        vertices: p
            .chunks_exact(3)
            .enumerate()
            .map(|(i, p)| {
                Vertex::new(
                    p[0].as_f64().unwrap() as f32,
                    p[1].as_f64().unwrap() as f32,
                    p[2].as_f64().unwrap() as f32,
                    uv.map_or(0., |uv| uv[i * 2].as_f64().unwrap() as f32),
                    uv.map_or(0., |uv| 1. - uv[i * 2 + 1].as_f64().unwrap() as f32),
                    WHITE,
                )
            })
            .collect(),
        indices: v["indices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u16)
            .collect(),
        texture: None,
    }
}
impl SnowballView {
    pub fn new() -> Result<Self, String> {
        let clip = load_material(
            ShaderSource::Glsl {
                vertex: VERT,
                fragment: FRAG,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("diffuse", UniformType::Float3),
                    UniformDesc::new("opacity", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
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
        let vertex_data = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;attribute vec4 color0;attribute vec4 normal;
varying vec2 uv;varying vec4 tint;varying vec4 data;
void main(){gl_Position=vec4(position,1.0);uv=texcoord;tint=color0/255.0;data=normal;}"#;
        let make = |fragment: &str, additive: bool, depth: bool, uniforms: Vec<UniformDesc>| {
            load_material(
                ShaderSource::Glsl {
                    vertex: vertex_data,
                    fragment,
                },
                MaterialParams {
                    uniforms,
                    pipeline_params: PipelineParams {
                        depth_test: if depth {
                            Comparison::LessOrEqual
                        } else {
                            Comparison::Always
                        },
                        depth_write: depth,
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
                            BlendFactor::One,
                            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                        )),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())
        };
        let tile = make(
            r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;varying vec4 data;uniform float Time;
void main(){vec4 color=data.w==5.0?vec4(.94,.97,1.,1.):vec4(1.);if(data.y<.5)color=vec4(.8,.87,.86,1.)+data.y/7.;if(data.w==2.){float t=.15+abs(sin(Time/300.))*.15;color.rgb*=vec3(1.-.45*t,1.-.75*t,1.);}gl_FragColor=vec4(color.rgb,color.a*uv.x);}"#,
            false,
            true,
            vec![UniformDesc::new("Time", UniformType::Float1)],
        )?;
        let particle = make(
            r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;varying vec4 data;uniform sampler2D Texture;uniform float Time;uniform vec3 diffuse;uniform float opacity;
void main(){if(data.z==-1.){gl_FragColor=vec4(diffuse,opacity);return;}if(data.w>.5){float scale=data.y/6.;float fade=(Time-data.x)/(700.*scale);float show=.5-length(uv-vec2(.5));gl_FragColor=vec4(tint.rgb,max(0.,(show-fade)/(3.*scale)));}else{float elapsed=min((Time-data.x)/300.,1.);vec4 c=texture2D(Texture,uv);if(c.a>.5)c.a=elapsed;gl_FragColor=c;}}"#,
            false,
            false,
            vec![
                UniformDesc::new("Time", UniformType::Float1),
                UniformDesc::new("diffuse", UniformType::Float3),
                UniformDesc::new("opacity", UniformType::Float1),
            ],
        )?;
        // Reuse the existing equal-depth and no-depth straight-alpha pipelines.
        // Macroquad has a fixed 32-pipeline limit across all eagerly loaded scenes.
        // Marker data.z is -1; ordinary clip geometry uses 0 and particles use
        // their nonnegative random value, so their shader branches stay unchanged.
        for material in [&clip, &particle] {
            material.set_uniform("diffuse", vec3(34. / 255., 204. / 255., 136. / 255.));
            material.set_uniform("opacity", 0.5f32);
        }
        let iceberg = make(
            r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;varying vec4 data;uniform float Time;
void main(){float alpha=pow(length(abs(uv-vec2(.5))),3.);if(data.y<0.)alpha=min(0.,.5-data.y);gl_FragColor=vec4(.6,.9,1.,(alpha*.9+.1)*data.w);}"#,
            true,
            true,
            vec![UniformDesc::new("Time", UniformType::Float1)],
        )?;
        let projectile = make(
            r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;varying vec4 data;uniform sampler2D Texture;
void main(){float time=data.y;float a=time/(1000.+5000.*data.x)*3.14159*2.;vec2 p=vec2(uv.x,1.-uv.y)-.5;vec2 q=mat2(cos(a),-sin(a),sin(a),cos(a))*p+.5;vec4 c=texture2D(Texture,vec2(q.x,1.-q.y));c-=vec4(.05,.015,0.,0.)*cos(time/30.*data.x);gl_FragColor=c;}"#,
            false,
            true,
            vec![],
        )?;
        let vertex = include_str!("../assets/shaders/ski-standard.vert")
            .replace("#define USE_MAP", "#define USE_MAP\n#define USE_COLOR")
            .replace("attribute vec3 color;", "attribute vec4 color0;");
        let vertex = vertex.replace(
            "vColor.xyz = color.xyz;",
            "vColor.xyz = color0.rgb / 255.0;",
        );
        let frag = santa_tracker::snowball_material::elf_fragment();
        let lit = crate::ski_view::lit(&vertex, &frag, false)?;
        let gifts = [
            (0xfade4b, 0xbe584a),
            (0xbe584a, 0xfade4b),
            (0xe68f49, 0x4eb3ec),
            (0xfadf4b, 0x4eb3ec),
            (0x87488f, 0x67b783),
            (0x87488f, 0xfadf4b),
            (0x4eb3ea, 0x87488f),
        ]
        .into_iter()
        .map(|(a, b)| {
            let mut im = Image::gen_image_color(128, 128, Color::from_hex(a));
            for y in 0..128 {
                for x in 0..128 {
                    if (24..40).contains(&x) || (24..40).contains(&y) {
                        im.set_pixel(x, y, Color::from_hex(b))
                    }
                }
            }
            let t = Texture2D::from_image(&im);
            crate::assets::configure_texture("scenes/snowball/gift.png", &t);
            t
        })
        .collect();
        Ok(Self {
            model: None,
            shapes: crate::ui_render::Shapes::new()?,
            clip,
            marker_mesh: geometry(include_str!("../data/snowball-marker-geometry.json")),
            destination_mesh: geometry(include_str!(
                "../data/snowball-destination-marker-geometry.json"
            )),
            tile,
            particle,
            iceberg,
            projectile,
            lit,
            textures: HashMap::new(),
            hex: geometry(include_str!("../data/snowball-hex-geometry.json")),
            present: geometry(include_str!("../data/ski-present-geometry.json")),
            gifts,
            pressed: false,
            button_down: false,
            powerup_slots: [
                santa_tracker::ui::Transition::new([51.2, 266.189, 1.]),
                santa_tracker::ui::Transition::new([147.2, 266.189, 0.5]),
            ],
            powerup_icons: std::array::from_fn(|_| {
                std::array::from_fn(|_| santa_tracker::ui::Transition::new([1., 0., 0., 0.]))
            }),
        })
    }
    fn elf_texture(&mut self, color: usize, skin: usize, hair: usize, pony: bool) -> Texture2D {
        let key = (color, skin);
        if let Some(t) = self.textures.get(&key) {
            return t.clone();
        }
        use santa_tracker::snowball_texture::{elf_rgba, HAIR, MAJOR, SKIN};
        let im = Image {
            bytes: elf_rgba(MAJOR[color], SKIN[skin], HAIR[hair], pony),
            width: 512,
            height: 512,
        };
        let t = Texture2D::from_image(&im);
        crate::assets::configure_texture("scenes/snowball/elf.png", &t);
        self.textures.insert(key, t.clone());
        t
    }
    #[allow(dead_code)] // Isolated reference renderer injection, not live gameplay.
    pub fn set_texture(&mut self, color: usize, skin: usize, texture: Texture2D) {
        self.textures.insert((color, skin), texture);
    }
    pub fn input(&mut self, g: &mut Snowball, enabled: bool) {
        if !enabled {
            self.pressed = false;
            self.button_down = false;
            return;
        }
        let mouse = Vec2::from(mouse_position());
        let center = vec2(screen_width() / 2., screen_height() / 2.);
        if g.phase == Phase::Splash {
            let scale = if screen_width() <= 600. { 0.75 } else { 1. };
            let over = mouse.distance(center) <= 140. * scale;
            if is_mouse_button_pressed(MouseButton::Left) && over {
                self.button_down = true
            }
            self.pressed = self.button_down && is_mouse_button_down(MouseButton::Left) && over;
            if is_mouse_button_released(MouseButton::Left) {
                if self.button_down && over {
                    g.jump()
                }
                self.button_down = false;
                self.pressed = false
            }
            return;
        }
        if g.phase != Phase::Playing
            || g.gameover
            || !is_mouse_button_released(MouseButton::Left)
            || crate::chrome::over_controls(mouse, 2)
        {
            return;
        }
        let vp = Self::vp(g);
        let inverse = vp.inverse();
        let ndc = vec2(
            2. * mouse.x / screen_width() - 1.,
            1. - 2. * mouse.y / screen_height(),
        );
        let a = inverse.project_point3(ndc.extend(-1.));
        let b = inverse.project_point3(ndc.extend(1.));
        let direction = b - a;
        // Original 100x100 hit planes at z=18, elf hits precede map picking.
        for (i, p) in g.players.iter().enumerate() {
            if i == 0 || !p.arrived || p.gone {
                continue;
            }
            let t = (18. - a.z) / direction.z;
            let hit = a + direction * t;
            if (hit.x - p.pos.x as f32).abs() <= 50. && (hit.y - p.pos.y as f32).abs() <= 50. {
                let target = p.pos;
                g.target_position(target);
                return;
            }
        }
        let t = (8. - a.z) / direction.z;
        let hit = a + direction * t;
        if hit.x.abs() <= g.map.grid.pixel_width() as f32 / 2.
            && hit.y.abs() <= g.map.grid.pixel_height() as f32 / 2.
        {
            g.pick_destination(V::new((hit.x - 2.) as f64, (hit.y + 2.) as f64));
        }
    }
    pub fn vp(g: &Snowball) -> Mat4 {
        world_vp(g.camera, screen_width(), screen_height())
    }
    fn canopy_matrix(c: &Canopy) -> Mat4 {
        Mat4::from_translation(vec3(c.pos.x as f32, c.pos.y as f32, c.z as f32))
            * Mat4::from_rotation_y(c.rotation as f32)
    }
    fn carried_matrix(c: &Canopy) -> Mat4 {
        Self::canopy_matrix(c) * Mat4::from_translation(vec3(0., 0., -84.))
    }
    fn ice_matrix(b: &Ice) -> Mat4 {
        Mat4::from_translation(vec3(b.pos.x as f32, b.pos.y as f32, b.z as f32))
            * Mat4::from_rotation_z(b.rotation as f32)
    }
    fn actor_base(g: &Snowball, id: Id, parent: Parent, local: Vec3) -> Option<Mat4> {
        let parent = match parent {
            Parent::Detached => return None,
            Parent::Map => Mat4::IDENTITY,
            Parent::Canopy => Self::carried_matrix(g.parachutes.get(id)?),
            Parent::Ice(slot) => Self::ice_matrix(g.removal.body(slot)?),
        };
        Some(parent * Mat4::from_translation(local))
    }
    fn deform(g: &Snowball, index: usize, mut p: Vec3, time: f64) -> (Vec3, f32) {
        let elapsed = time as f32 - g.map.times[index] as f32;
        let mut alpha = 1.;
        match g.map.states[index] {
            3 => p.x += (elapsed / 70. * PI).sin(),
            4 => {
                alpha = (1. - elapsed / 300.).max(0.);
                p.z = (p.z - (elapsed / 50.).powi(2)) / (1. + elapsed / 600.).min(2.)
            }
            _ => {}
        }
        (p, alpha)
    }
    fn quad(mesh: &mut Mesh, vp: Mat4, model: Mat4, size: Vec2, uv: Rect, color: Color) {
        let n = mesh.vertices.len() as u16;
        for (p, t) in [
            (vec2(-0.5, 0.5), vec2(uv.x, uv.y)),
            (vec2(0.5, 0.5), vec2(uv.x + uv.w, uv.y)),
            (vec2(0.5, -0.5), vec2(uv.x + uv.w, uv.y + uv.h)),
            (vec2(-0.5, -0.5), vec2(uv.x, uv.y + uv.h)),
        ] {
            mesh.vertices.push(Vertex::new2(
                vp.transform_point3(model.transform_point3((p * size).extend(0.))),
                t,
                color,
            ));
        }
        mesh.indices.extend([n, n + 1, n + 2, n, n + 2, n + 3]);
    }
    pub fn draw(&mut self, assets: &Assets, g: &Snowball) -> Result<(), String> {
        if g.phase != Phase::Playing || g.wall < g.splash_until {
            self.splash(assets, g);
            return Ok(());
        }
        if self.model.is_none() {
            self.model = Some(ElfModel::load(&assets.root.join("scenes/snowball/models"))?)
        }
        // Source texture cache is populated in allocation order, even offscreen.
        for p in &g.players {
            self.elf_texture(p.color, p.skin, p.hair, p.pony);
        }
        clear_background(Color::from_hex(0x71a7db));
        let vp = Self::vp(g);
        gl_use_material(&self.clip);
        // Source cylinder coordinates and shader orientation, sharing exact edges.
        let mut tiles = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: None,
        };
        let mut trees = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: Some(assets.scene("snowball", "tiles.png").clone()),
        };
        for i in 0..g.map.states.len() {
            let state = g.map.states[i];
            if state == 0 {
                continue;
            }
            let at = g.map.grid.pos(i as i32);
            let origin = vec3(at.x as f32, at.y as f32, 0.);
            let center = vp.transform_point3(origin);
            if center.x.abs() > 1.3 || center.y.abs() > 1.5 {
                continue;
            }
            if tiles.vertices.len() + self.hex.vertices.len() > 60000 {
                self.tile.set_uniform("Time", g.map_shader_time as f32);
                gl_use_material(&self.tile);
                draw_batched(&tiles);
                gl_use_material(&self.clip);
                tiles.vertices.clear();
                tiles.indices.clear();
            }
            let first = tiles.vertices.len() as u16;
            let mut alpha = 1.;
            for v in &self.hex.vertices {
                let p = v.position;
                let mut pos = vec3(-p.z, -p.x, p.y) * 32.;
                if state == 5 && pos.z > 0. {
                    pos.z *= 3.
                }
                let (d, a) = Self::deform(g, i, pos + origin, g.map_shader_time);
                alpha = a;
                // The source fragment multiplies float fade by its side alpha.
                // UV is unused by tile color; carry fade without a u8 roundtrip.
                let mut vertex = Vertex::new2(vp.transform_point3(d), vec2(a, 0.), WHITE);
                vertex.normal = p.extend(state as f32);
                tiles.vertices.push(vertex)
            }
            tiles
                .indices
                .extend(self.hex.indices.iter().map(|j| first + j));
            if g.map.obstacles[i] >= 0 && alpha > 0. {
                let ob = g.map.obstacles[i] as f32;
                let n = trees.vertices.len() as u16;
                for (p, uv) in [
                    (vec2(-1., 1.), vec2(0., 0.)),
                    (vec2(1., 1.), vec2(0.25, 0.)),
                    (vec2(1., -1.), vec2(0.25, 0.25)),
                    (vec2(-1., -1.), vec2(0., 0.25)),
                ] {
                    let pos = origin
                        + vec3(
                            p.x * 64.,
                            p.y * 64. * (PI / 6.).cos() + g.map.grid.cell_height() as f32 / 2.,
                            p.y * 32. + 48.,
                        );
                    let (pos, a) = Self::deform(g, i, pos, g.obstacle_shader_time);
                    let mut vertex = Vertex::new2(
                        vp.transform_point3(pos), uv + vec2(ob / 4., 0.), WHITE,
                    );
                    // Atlas alpha cutoff stays .1, after float fade multiplication.
                    vertex.normal = vec4(0., 0., -3., a);
                    trees.vertices.push(vertex);
                }
                trees.indices.extend([n, n + 1, n + 2, n, n + 2, n + 3]);
            }
        }
        self.tile.set_uniform("Time", g.map_shader_time as f32);
        gl_use_material(&self.tile);
        draw_batched(&tiles);
        gl_use_material(&self.clip);
        draw_batched(&trees);
        let mut parachutes = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: Some(assets.scene("snowball", "parachute.png").clone()),
        };
        for c in &g.parachutes.canopies {
            Self::quad(
                &mut parachutes,
                vp,
                Self::canopy_matrix(c) * Mat4::from_rotation_x(PI / 6.),
                vec2(64., 64.),
                Rect::new(0., 0., 1., 1.),
                WHITE,
            );
        }
        // Parachute's MeshBasicMaterial also has no alpha test. Keep the
        // filtered translucent fringe, including its original depth writes.
        for vertex in &mut parachutes.vertices {
            vertex.normal.z = -2.;
            vertex.normal.w = 1.;
        }
        for (di, d) in g.drops.iter().enumerate() {
            let Some(base) = Self::actor_base(
                g,
                Id::Drop(di),
                d.parent,
                vec3(d.pos.x as f32, d.pos.y as f32, (d.entity_z + d.z) as f32),
            ) else {
                continue;
            };
            let model =
                base * Mat4::from_quat(Quat::from_euler(
                    EulerRot::XYZ,
                    d.tilt as f32,
                    d.rotation as f32,
                    0.,
                )) * Mat4::from_scale(vec3(
                    d.scale[0] as f32,
                    d.scale[1] as f32,
                    d.scale[2] as f32,
                ));
            let mut gift = Mesh {
                vertices: self.present.vertices.clone(),
                indices: self.present.indices.clone(),
                texture: Some(self.gifts[d.color].clone()),
            };
            let opacity = d.opacity as f32;
            for v in &mut gift.vertices {
                v.position = vp.transform_point3(model.transform_point3(v.position));
                v.color = WHITE.into();
                // Drop uses MeshBasicMaterial with the default alphaTest=0.
                // Only tree atlas fragments discard below .1. Keep opacity
                // in a float attribute instead of quantizing it to color's u8.
                v.normal.z = -2.;
                v.normal.w = opacity;
            }
            draw_batched(&gift);
        }
        for (i, p) in g.players.iter().enumerate() {
            if !p.lod.root_visible {
                continue;
            }
            let Some(base) = Self::actor_base(
                g,
                Id::Player(i),
                p.parent,
                vec3(p.pos.x as f32, p.pos.y as f32, p.z as f32),
            ) else {
                continue;
            };
            let screen = vp.transform_point3(base.transform_point3(Vec3::ZERO));
            if screen.x.abs() > 1.3 || screen.y.abs() > 1.5 {
                continue;
            }
            let texture = self.elf_texture(p.color, p.skin, p.hair, p.pony);
            let mut mesh = self
                .model
                .as_ref()
                .unwrap()
                .mesh_pose_scaled(&p.animator.pose, 5.);
            mesh.texture = Some(texture);
            let model = base
                * Mat4::from_translation(vec3(0., -10., 19.))
                * Mat4::from_quat(Quat::from_euler(
                    EulerRot::XYZ,
                    PI / 2.25,
                    p.face as f32,
                    0.,
                ));
            self.lighting(g, model, p.material_opacity as f32);
            gl_use_material(&self.lit);
            draw_batched(&mesh);
            gl_use_material(&self.clip);
            if g.client_ring.owner == Some(i) && g.client_ring.visible {
                self.draw_marker(vp, model * Mat4::from_rotation_x(-PI / 2.), false);
                gl_use_material(&self.clip);
            }
        }
        draw_batched(&parachutes);
        let mut shots = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: Some(assets.scene("snowball", "snowball.png").clone()),
        };
        for s in &g.shots {
            if !s.render_eligible() {
                continue;
            }
            let size = if s.big { 40. } else { 12. };
            let from = shots.vertices.len();
            Self::quad(
                &mut shots,
                vp,
                Mat4::from_translation(vec3(s.pos.x as f32, s.pos.y as f32, s.z as f32)),
                Vec2::splat(size),
                Rect::new(0., 0., 1., 1.),
                WHITE,
            );
            for v in &mut shots.vertices[from..] {
                v.normal = vec4(s.random as f32, s.shader_time as f32, 0., 0.);
            }
        }
        gl_use_material(&self.projectile);
        draw_batched(&shots);
        gl_use_material(&self.clip);
        self.effects(assets, g, vp);
        if let Some(marker) = g.client_arrival.marker {
            self.draw_marker(
                vp,
                Mat4::from_translation(vec3(
                    marker.position[0] as f32,
                    marker.position[1] as f32,
                    marker.position[2] as f32,
                )) * Mat4::from_rotation_x(marker.rotation as f32),
                true,
            );
            gl_use_material(&self.clip);
        }
        if let Some(marker) = g.destination_marker {
            self.draw_symbol(
                &self.destination_mesh,
                vp,
                destination_matrix(marker, g.map.grid.size as f32),
                true,
            );
            gl_use_material(&self.clip);
        }

        gl_use_default_material();
        self.powerups(assets, g);
        Ok(())
    }
    fn draw_marker(&self, vp: Mat4, at: Mat4, arrival: bool) {
        self.draw_symbol(&self.marker_mesh, vp, at, arrival);
    }
    fn draw_symbol(&self, geometry: &Mesh, vp: Mat4, at: Mat4, no_depth: bool) {
        let mut mesh = Mesh {
            vertices: geometry.vertices.clone(),
            indices: geometry.indices.clone(),
            texture: None,
        };
        for v in &mut mesh.vertices {
            v.position = vp.transform_point3(at.transform_point3(v.position));
            v.normal = vec4(0., 0., -1., 0.);
        }
        gl_use_material(if no_depth { &self.particle } else { &self.clip });
        draw_batched(&mesh);
    }
    fn effects(&self, assets: &Assets, g: &Snowball, vp: Mat4) {
        let mut mesh = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: Some(assets.scene("snowball", "snowball.png").clone()),
        };
        // RawShaderMaterial gl_PointSize is in framebuffer pixels: THREE's
        // PointsMaterial-only pixel-ratio scaling does not apply here.
        let (pixel_width, pixel_height) = santa_tracker::surface::framebuffer_size();
        // EffectSystem draws the entire trail layer before the impact layer.
        for p in [true, false]
            .into_iter()
            .flat_map(|trail| g.particles.iter().filter(move |p| p.trail == trail))
        {
            let born = p.born as f32;
            let delta = g.effects.time as f32 - born;
            let elapsed = delta / if p.trail { 700. } else { 300. };
            if elapsed >= 1. || if p.trail { born <= 0. } else { born < 1. } {
                continue;
            }
            let (offset, size) = if p.trail {
                (
                    vec2(0., -(75. - (3.14159 / 2. + 3.14159 / 2. * elapsed).sin() * 75.)),
                    p.size as f32 * (10. - elapsed * 7.5),
                )
            } else {
                let random = p.random as f32;
                let spread = PI / -3. * random + PI / 6.;
                let d = vec2(p.direction.x as f32, p.direction.y as f32);
                let rotated = vec2(
                    spread.cos() * d.x + spread.sin() * d.y,
                    -spread.sin() * d.x + spread.cos() * d.y,
                );
                let mut offset = rotated * ((random * 7.158).fract() * 85. * elapsed);
                offset.y -= (3. * elapsed).powi(2);
                (
                    offset,
                    (p.size as f32 / 2. * random + p.size as f32 / 2. + p.size as f32 / 8.) * (1. - elapsed),
                )
            };
            let center = vp.transform_point3(vec3(
                p.pos.x as f32 + offset.x,
                p.pos.y as f32 + offset.y,
                p.z as f32,
            ));
            let n = mesh.vertices.len() as u16;
            for (delta, uv) in [
                (vec2(-1., 1.), vec2(0., 0.)),
                (vec2(1., 1.), vec2(1., 0.)),
                (vec2(1., -1.), vec2(1., 1.)),
                (vec2(-1., -1.), vec2(0., 1.)),
            ] {
                let mut v = Vertex::new2(
                    center
                        + vec3(
                            delta.x * size / pixel_width as f32,
                            delta.y * size / pixel_height as f32,
                            0.,
                        ),
                    uv,
                    WHITE,
                );
                if p.trail {
                    v.color = [
                        (p.color[0] * 255.).round() as u8,
                        (p.color[1] * 255.).round() as u8,
                        (p.color[2] * 255.).round() as u8,
                        255,
                    ];
                }
                v.normal = vec4(
                    p.born as f32,
                    p.size as f32,
                    p.random as f32,
                    if p.trail { 1. } else { 0. },
                );
                mesh.vertices.push(v);
            }
            mesh.indices.extend([n, n + 1, n + 2, n, n + 2, n + 3]);
        }
        self.particle.set_uniform("Time", g.effects.time as f32);
        gl_use_material(&self.particle);
        draw_batched(&mesh);
        let mut ice = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: None,
        };
        for &slot in &g.removal.layer {
            let body = &g.removal.bodies[slot];
            let radius = body.radius.unwrap_or(32.) as f32;
            let model = Self::ice_matrix(body);
            let n = ice.vertices.len() as u16;
            for v in &self.hex.vertices {
                let pos = v.position;
                let mut out = Vertex::new2(
                    vp.transform_point3(
                        model.transform_point3(vec3(-pos.z, -pos.x, pos.y) * radius),
                    ),
                    v.uv,
                    WHITE,
                );
                out.normal = (pos * radius).extend(body.opacity as f32);
                ice.vertices.push(out)
            }
            ice.indices.extend(self.hex.indices.iter().map(|i| i + n));
        }
        self.iceberg.set_uniform("Time", g.time as f32);
        gl_use_material(&self.iceberg);
        draw_batched(&ice);
        gl_use_material(&self.clip);
    }
    fn lighting(&self, g: &Snowball, model: Mat4, opacity: f32) {
        let eye = vec3(g.camera.x as f32, g.camera.y as f32, -3200.);
        let view = Mat4::look_at_rh(eye, eye + Vec3::Z, -Vec3::Y);
        let model = Mat4::from_rotation_x(4. * PI / 5.) * model;
        let projection = Mat4::orthographic_rh_gl(
            -screen_width() / 2.,
            screen_width() / 2.,
            -screen_height() / 2.,
            screen_height() / 2.,
            1.,
            100000.,
        );
        for (k, m) in [
            ("modelMatrix", model),
            ("modelViewMatrix", view * model),
            ("projectionMatrix", projection),
            ("viewMatrix", view),
            ("NormalMatrix", (view * model).inverse().transpose()),
            ("UvTransform", Mat4::IDENTITY),
        ] {
            self.lit.set_uniform(k, m)
        }
        self.lit.set_uniform("cameraPosition", eye);
        self.lit.set_uniform("isOrthographic", 1i32);
        self.lit.set_uniform("diffuse", Vec3::splat(0.64));
        self.lit.set_uniform("emissive", Vec3::ZERO);
        self.lit.set_uniform(
            "ambientLightColor",
            vec3(187. / 255., 170. / 255., 170. / 255.) * PI,
        );
        for (k, n) in [
            ("opacity", opacity),
            ("toneMappingExposure", 1.),
            ("roughness", 1.),
            ("metalness", 0.),
            ("spotLights[0].distance", 0.),
            ("spotLights[0].decay", 1.),
            ("spotLights[0].coneCos", 0.5),
            ("spotLights[0].penumbraCos", 0.5),
        ] {
            self.lit.set_uniform(k, n)
        }
        self.lit.set_uniform("spotLights[0].position", Vec3::Z);
        self.lit.set_uniform("spotLights[0].direction", Vec3::Z);
        self.lit.set_uniform("spotLights[0].color", Vec3::ZERO);
    }
    fn powerups(&mut self, assets: &Assets, g: &Snowball) {
        use santa_tracker::ui::Ease;
        let Some(p) = g.players.first() else { return };
        let responsive = if screen_width() <= 768. { 0.7 } else { 1. };
        let now = g.wall / 1000.;
        santa_tracker::surface::scissor(Some((
            0,
            (screen_height() - 265. * responsive) as i32,
            (212. * responsive) as i32,
            (265. * responsive) as i32,
        )));
        for slot in 0..2 {
            let quantity = if slot == 0 {
                p.powerups.primary
            } else {
                p.powerups.secondary
            }
            .unwrap_or(0);
            let active = (slot == 0) != p.powerups.swapped;
            let target = if active {
                [51.2, if quantity > 0 { 0. } else { 266.189 }, 1.]
            } else {
                [147.2, if quantity > 0 { 66.5473 } else { 266.189 }, 0.5]
            };
            self.powerup_slots[slot].set(now, target, 0.3, Ease::Default);
            let [x, y, s] = self.powerup_slots[slot].value(now);
            let size = vec2(128., 209. / 201. * 128.) * (s as f32 * responsive);
            let origin = vec2(
                x as f32 * responsive,
                screen_height() - 265. * responsive + y as f32 * responsive,
            );
            draw_texture_ex(
                assets.scene("snowball", "item-frame.png"),
                origin.x,
                origin.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(size),
                    ..Default::default()
                },
            );
            for i in 0..3 {
                let (scale, dx, dy) = if quantity >= 3 {
                    match i {
                        0 => (0.5, -0.2, -0.2),
                        1 => (0.6, 0.27, -0.06),
                        _ => (0.8, 0., 0.14),
                    }
                } else if quantity == 2 && i < 2 {
                    (
                        0.6,
                        if i == 0 { -0.33 } else { 0.33 },
                        if i == 0 { -0.06 } else { 0.06 },
                    )
                } else {
                    (1., 0., 0.)
                };
                self.powerup_icons[slot][i].set(
                    now,
                    [scale, dx, dy, if i < quantity as usize { 1. } else { 0. }],
                    0.3,
                    Ease::Default,
                );
                let [scale, dx, dy, alpha] = self.powerup_icons[slot][i].value(now);
                let at = origin
                    + (vec2(28., 28.)
                        + Vec2::splat(36. * (1. - scale as f32))
                        + vec2(dx as f32, dy as f32) * 72.)
                        * (s as f32 * responsive);
                draw_texture_ex(
                    assets.scene("snowball", "powerup-1.png"),
                    at.x,
                    at.y,
                    Color::new(1., 1., 1., alpha as f32),
                    DrawTextureParams {
                        dest_size: Some(Vec2::splat(72. * scale as f32 * s as f32 * responsive)),
                        ..Default::default()
                    },
                );
            }
        }
        santa_tracker::surface::scissor(None);
    }
    fn splash(&self, assets: &Assets, g: &Snowball) {
        clear_background(Color::from_hex(0x7ca6d6));
        let w = screen_width();
        let h = screen_height();
        let t = (g.wall / 3000.) % 1.;
        let p = if t < 0.5 { t * 2. } else { 2. - t * 2. };
        let ease = santa_tracker::ui::Ease::InOut.at(p) as f32;
        let y = h / 2. - 60. - 240. - 12. * ease;
        let small = w <= 600.;
        let size = if small {
            vec2(257., 45.25)
        } else {
            vec2(514., 90.5)
        };
        draw_texture_ex(
            assets.scene("snowball", "sleigh.png"),
            (w - size.x) / 2.,
            if small { y + 97.375 } else { y + 14.75 },
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
        if g.phase == Phase::Playing {
            return;
        }
        if g.phase == Phase::Intro {
            let f = ((g.wall - g.intro_at) / 5000.) as f32;
            let height = if small { 128. } else { 256. };
            let cloud = assets.scene("snowball", "loading-cloud.svg");
            let size = vec2(height * 117.8 / 70.7, height);
            draw_texture_ex(
                cloud,
                (w - size.x) / 2. + w * (1. - f * 2.),
                h / 2. - 128. - height * 0.9,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(size),
                    ..Default::default()
                },
            );
            if f >= 0.5 {
                let size = if small { 100. } else { 160. };
                let y = h * (-0.2 + (f - 0.5) * 2. * 1.2) + h / 2. - size / 2.;
                draw_texture_ex(
                    assets.scene("snowball", "elves-parachuting.svg"),
                    w / 2. - size / 2.,
                    y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(size, size)),
                        source: Some(Rect::new(
                            0.,
                            0.,
                            assets.scene("snowball", "elves-parachuting.svg").width() / 2.,
                            assets.scene("snowball", "elves-parachuting.svg").height() / 2.,
                        )),
                        ..Default::default()
                    },
                );
            }
            return;
        }
        let scale = if small { 0.75 } else { 1. };
        let c = vec2(w / 2., h / 2.);
        self.shapes.circle(c, 140. * scale, WHITE, 0.);
        self.shapes.circle(
            vec2(c.x - 1.4 * scale, c.y - 2.8 * scale),
            123.2 * scale,
            Color::from_hex(0x448b4d),
            0.,
        );
        let y = c.y + if self.pressed { -2.8 } else { -21. } * scale;
        self.shapes.circle(
            vec2(c.x - 1.4 * scale, y),
            123.2 * scale,
            Color::from_hex(0x56ad57),
            0.,
        );
        let size = 98.56 * scale;
        let tex = assets.scene("snowball", "parachute-button.png");
        draw_texture_ex(
            tex,
            c.x - 1.4 * scale - size / 2.,
            y - (size * tex.height() / tex.width()) / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size * tex.height() / tex.width())),
                ..Default::default()
            },
        );
    }
}

fn draw_batched(mesh: &Mesh) {
    for indices in mesh.indices.chunks(3900) {
        let mut remap = HashMap::new();
        let mut vertices = vec![];
        let mut local = vec![];
        for &i in indices {
            let n = *remap.entry(i).or_insert_with(|| {
                let n = vertices.len() as u16;
                vertices.push(mesh.vertices[i as usize]);
                n
            });
            local.push(n);
        }
        draw_mesh(&Mesh {
            vertices,
            indices: local,
            texture: mesh.texture.clone(),
        });
    }
}

#[cfg(test)]
mod arrival_tests {
    use super::*;
    #[test]
    fn canopy_and_carried_matrices_match_actual_three_hierarchy() {
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-arrival-reference.json")).unwrap();
        let mut checked = 0;
        for case in data["cases"].as_array().unwrap() {
            for frame in case["frames"].as_array().unwrap() {
                for phase in ["before", "after"] {
                    for row in frame[phase]["canopies"].as_array().unwrap() {
                        let id = row["id"].as_str().unwrap();
                        let c = Canopy {
                            entity: if id.starts_with('p') {
                                Id::Player(0)
                            } else {
                                Id::Drop(0)
                            },
                            pos: V::new(
                                row["pos"][0].as_f64().unwrap(),
                                row["pos"][1].as_f64().unwrap(),
                            ),
                            z: row["pos"][2].as_f64().unwrap(),
                            rotation: row["rotation"].as_f64().unwrap(),
                        };
                        let carried = SnowballView::carried_matrix(&c);
                        let sample = if id.starts_with('p') {
                            carried
                                * Mat4::from_translation(vec3(0., -10., 19.))
                                * Mat4::from_quat(Quat::from_euler(
                                    EulerRot::XYZ,
                                    PI / 2.25,
                                    0.8,
                                    0.,
                                ))
                        } else {
                            carried
                                * Mat4::from_translation(vec3(0., 0., 32.))
                                * Mat4::from_quat(Quat::from_euler(
                                    EulerRot::XYZ,
                                    PI / 2.5,
                                    0.2,
                                    0.,
                                ))
                        };
                        for (name, matrix) in [
                            ("carried", carried),
                            (
                                "graphic",
                                SnowballView::canopy_matrix(&c) * Mat4::from_rotation_x(PI / 6.),
                            ),
                            ("sample", sample),
                        ] {
                            for (i, a) in matrix.to_cols_array().into_iter().enumerate() {
                                let b = row[name][i].as_f64().unwrap();
                                assert!(
                                    (a as f64 - b).abs() < 0.0003,
                                    "{} {phase} {id} {name}[{i}]: {a} vs {b}",
                                    case["name"]
                                );
                            }
                            checked += 1;
                        }
                    }
                }
            }
        }
        assert!(checked > 100);
    }
}

#[cfg(test)]
mod removal_tests {
    use super::*;
    #[test]
    fn iceberg_and_frozen_present_matrices_match_original_three() {
        let data: Value = serde_json::from_str(include_str!(
            "../data/snowball-removal-render-reference.json"
        ))
        .unwrap();
        let mut game = Snowball::new("removal-matrices");
        for row in data["frames"].as_array().unwrap() {
            let body = Ice {
                pos: V::new(
                    row["pos"][0].as_f64().unwrap(),
                    row["pos"][1].as_f64().unwrap(),
                ),
                z: row["pos"][2].as_f64().unwrap(),
                rotation: row["rotation"].as_f64().unwrap(),
                opacity: row["opacity"].as_f64().unwrap(),
                start_tick: row["start"].as_f64().unwrap(),
                radius: Some(32.),
            };
            game.removal.bodies = vec![body];
            game.removal.layer = vec![0];
            let base = SnowballView::actor_base(
                &game,
                Id::Drop(0),
                Parent::Ice(0),
                vec3(
                    row["entity"][0].as_f64().unwrap() as f32,
                    row["entity"][1].as_f64().unwrap() as f32,
                    (row["entity"][2].as_f64().unwrap() + row["modelZ"].as_f64().unwrap()) as f32,
                ),
            )
            .unwrap();
            let model = Mat4::from_rotation_x(4. * PI / 5.)
                * base
                * Mat4::from_quat(Quat::from_euler(
                    EulerRot::XYZ,
                    row["modelX"].as_f64().unwrap() as f32,
                    row["modelY"].as_f64().unwrap() as f32,
                    0.,
                ))
                * Mat4::from_scale(vec3(18., 16., 13.));
            for (name, matrix) in [
                ("matrix", SnowballView::ice_matrix(&body)),
                ("modelMatrix", model),
            ] {
                for (i, a) in matrix.to_cols_array().into_iter().enumerate() {
                    let b = row[name][i].as_f64().unwrap();
                    assert!(
                        (a as f64 - b).abs() < 0.0003,
                        "{} {name}[{i}]: {a} vs {b}",
                        row["tick"]
                    );
                }
            }
        }
    }
}

fn world_vp(camera: V, w: f32, h: f32) -> Mat4 {
    let p = Mat4::orthographic_rh_gl(-w / 2., w / 2., -h / 2., h / 2., 1., 100000.);
    let eye = vec3(camera.x as f32, camera.y as f32, -3200.);
    p * Mat4::look_at_rh(eye, eye + Vec3::Z, -Vec3::Y) * Mat4::from_rotation_x(4. * PI / 5.)
}

fn destination_matrix(marker: V, cell_size: f32) -> Mat4 {
    Mat4::from_translation(vec3(marker.x as f32, marker.y as f32, cell_size * 0.5))
        * Mat4::from_rotation_z(PI / 4.)
}

#[cfg(test)]
mod destination_tests {
    use super::*;
    #[test]
    fn destination_shape_and_projection_match_actual_three() {
        let data: Value =
            serde_json::from_str(include_str!("../data/snowball-command-reference.json")).unwrap();
        let mesh = geometry(include_str!(
            "../data/snowball-destination-marker-geometry.json"
        ));
        assert_eq!(mesh.vertices.len(), 12);
        for row in data["projections"].as_array().unwrap() {
            let v = |a: &Value| V::new(a[0].as_f64().unwrap(), a[1].as_f64().unwrap());
            let camera = v(&row["camera"]);
            let target = v(&row["target"]);
            let vp = world_vp(
                camera,
                row["width"].as_f64().unwrap() as f32,
                row["height"].as_f64().unwrap() as f32,
            );
            let model = destination_matrix(
                V::new(target.x, target.y - 20.),
                row["cellSize"].as_f64().unwrap() as f32,
            );
            for (vertex, expected) in mesh.vertices.iter().zip(row["clip"].as_array().unwrap()) {
                let clip = vp.transform_point3(model.transform_point3(vertex.position));
                for (i, c) in clip.to_array().iter().enumerate() {
                    assert!(
                        (*c - expected[i].as_f64().unwrap() as f32).abs() < 1e-5,
                        "{clip:?} != {expected}"
                    );
                }
            }
        }
    }
}
