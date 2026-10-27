//! Original Snowball Storm art, native hex meshes, skinned elves and splash.
use crate::{assets::Assets, snowball_model::ElfModel};
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Comparison, Equation},
    prelude::*,
};
use santa_tracker::snowball::{Phase, Snowball, V};
use serde_json::Value;
use std::{collections::HashMap, f32::consts::PI};
const VERT: &str = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;attribute vec4 color0;varying vec2 uv;varying vec4 tint;
void main(){gl_Position=vec4(position,1.0);uv=texcoord;tint=color0/255.0;}"#;
const FRAG: &str = r#"#version 100
precision highp float;uniform sampler2D Texture;varying vec2 uv;varying vec4 tint;
void main(){vec4 c=texture2D(Texture,uv)*tint;if(c.a<0.1)discard;gl_FragColor=c;}"#;
pub struct SnowballView {
    model: Option<ElfModel>,
    shapes: crate::ui_render::Shapes,
    clip: Material,
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
    let uv = v["uv"].as_array().unwrap();
    Mesh {
        vertices: p
            .chunks_exact(3)
            .enumerate()
            .map(|(i, p)| {
                Vertex::new(
                    p[0].as_f64().unwrap() as f32,
                    p[1].as_f64().unwrap() as f32,
                    p[2].as_f64().unwrap() as f32,
                    uv[i * 2].as_f64().unwrap() as f32,
                    1. - uv[i * 2 + 1].as_f64().unwrap() as f32,
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
        let make = |fragment: &str, additive: bool, depth: bool| {
            load_material(
                ShaderSource::Glsl {
                    vertex: vertex_data,
                    fragment,
                },
                MaterialParams {
                    uniforms: vec![UniformDesc::new("Time", UniformType::Float1)],
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
void main(){vec4 color=data.w==5.0?vec4(.94,.97,1.,1.):vec4(1.);if(data.y<.5)color=vec4(.8,.87,.86,1.)+data.y/7.;if(data.w==2.){float t=.15+abs(sin(Time/300.))*.15;color.rgb*=vec3(1.-.45*t,1.-.75*t,1.);}gl_FragColor=vec4(color.rgb,color.a*tint.a);}"#,
            false,
            true,
        )?;
        let particle = make(
            r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;varying vec4 data;uniform sampler2D Texture;uniform float Time;
void main(){if(data.w>.5){float scale=data.y/6.;float fade=(Time-data.x)/(700.*scale);float show=.5-length(uv-vec2(.5));gl_FragColor=vec4(.6666667,.8,1.,max(0.,(show-fade)/(3.*scale)));}else{float elapsed=min((Time-data.x)/300.,1.);vec4 c=texture2D(Texture,uv);if(c.a>.5)c.a=elapsed;gl_FragColor=c;}}"#,
            false,
            false,
        )?;
        let iceberg = make(
            r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;varying vec4 data;uniform float Time;
void main(){float alpha=pow(length(abs(uv-vec2(.5))),3.);if(data.y<0.)alpha=min(0.,.5-data.y);gl_FragColor=vec4(.6,.9,1.,(alpha*.9+.1)*tint.a);}"#,
            true,
            true,
        )?;
        let projectile = make(
            r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;varying vec4 data;uniform sampler2D Texture;uniform float Time;
void main(){float a=Time/(1000.+5000.*data.x)*3.14159*2.;vec2 p=vec2(uv.x,1.-uv.y)-.5;vec2 q=mat2(cos(a),-sin(a),sin(a),cos(a))*p+.5;vec4 c=texture2D(Texture,vec2(q.x,1.-q.y));c-=vec4(.05,.015,0.,0.)*cos(Time/30.*data.x);gl_FragColor=c;}"#,
            false,
            true,
        )?;
        let vertex = include_str!("../assets/shaders/ski-standard.vert")
            .replace("#define USE_MAP", "#define USE_MAP\n#define USE_COLOR")
            .replace("attribute vec3 color;", "attribute vec4 color0;");
        let vertex = vertex.replace(
            "vColor.xyz = color.xyz;",
            "vColor.xyz = color0.rgb / 255.0;",
        );
        let frag = include_str!("../assets/shaders/ski-standard.frag")
            .replace("#define USE_MAP", "#define USE_MAP\n#define USE_COLOR");
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
        let mut im = Image::gen_image_color(512, 512, BLANK);
        let major =
            Color::from_hex([0x84488f, 0xd55040, 0x4db1e9, 0x55ab57, 0x56bbca, 0xbc3eb4][color]);
        let minor = Color::from_hex([0xe7c241, 0x936644, 0xb8906d, 0xdbbc99][skin]);
        let hair = Color::from_hex([0x2f3030, 0x7c4149, 0xab2923, 0xea9639][hair]);
        for y in 0..512 {
            for x in 0..512 {
                let c = if y < 153 {
                    if x >= 256 {
                        WHITE
                    } else if pony || x >= 128 {
                        hair
                    } else {
                        BLANK
                    }
                } else if y < 327 {
                    if ((x as i32 - 44).pow(2) + (y as i32 - 207).pow(2) < 441)
                        || ((x as i32 - 150).pow(2) + (y as i32 - 207).pow(2) < 441)
                    {
                        Color::from_hex(0x333333)
                    } else {
                        minor
                    }
                } else if y < 467 {
                    major
                } else {
                    Color::from_hex(0x333333)
                };
                im.set_pixel(
                    x,
                    y,
                    if (252..312).contains(&x) && y >= 460 {
                        Color::from_hex(0xf9d448)
                    } else {
                        c
                    },
                );
            }
        }
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
                g.throw(0, target);
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
        let w = screen_width();
        let h = screen_height();
        let p = Mat4::orthographic_rh_gl(-w / 2., w / 2., -h / 2., h / 2., 1., 100000.);
        let eye = vec3(g.camera.x as f32, g.camera.y as f32, -3200.);
        p * Mat4::look_at_rh(eye, eye + Vec3::Z, -Vec3::Y) * Mat4::from_rotation_x(4. * PI / 5.)
    }
    fn deform(g: &Snowball, index: usize, mut p: Vec3) -> (Vec3, f32) {
        let elapsed = (g.time - g.map.times[index]) as f32;
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
                self.tile.set_uniform("Time", g.time as f32);
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
                let (d, a) = Self::deform(g, i, pos + origin);
                alpha = a;
                let c = if p.y < 0.5 {
                    Color::new(0.8 + p.y / 7., 0.87 + p.y / 7., 0.86 + p.y / 7., a)
                } else if state == 5 {
                    Color::new(0.94, 0.97, 1., a)
                } else {
                    Color::new(1., 1., 1., a)
                };
                let mut vertex = Vertex::new2(vp.transform_point3(d), Vec2::ZERO, c);
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
                    let (pos, a) = Self::deform(g, i, pos);
                    trees.vertices.push(Vertex::new2(
                        vp.transform_point3(pos),
                        uv + vec2(ob / 4., 0.),
                        Color::new(1., 1., 1., a),
                    ));
                }
                trees.indices.extend([n, n + 1, n + 2, n, n + 2, n + 3]);
            }
        }
        self.tile.set_uniform("Time", g.time as f32);
        gl_use_material(&self.tile);
        draw_batched(&tiles);
        gl_use_material(&self.clip);
        draw_batched(&trees);
        let mut parachutes = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: Some(assets.scene("snowball", "parachute.png").clone()),
        };
        for d in &g.drops {
            if d.gone {
                continue;
            }
            let (pos, z, rotation) = if d.frozen {
                let tick = ((g.time - d.freeze_at) * 0.06).floor();
                (
                    d.ice,
                    (tick / 10.).cos() * 32. * (1. - (tick / 75.).min(1.).powi(3) * 0.9) + 16.,
                    0.,
                )
            } else if d.arrived {
                (d.pos, d.z, 0.)
            } else {
                let t = ((g.time * 0.06).floor() - d.dropped) / 300.;
                let z = 420. - t * 420.;
                let pos = g.map.grid.pos(d.tile);
                let angle = 0.1 * (z / (0.35 * 420.) * std::f64::consts::PI).sin();
                let m = Mat4::from_translation(vec3(pos.x as f32, pos.y as f32, (z + 80.) as f32))
                    * Mat4::from_rotation_y(angle as f32);
                Self::quad(
                    &mut parachutes,
                    vp,
                    m * Mat4::from_rotation_x(PI / 6.),
                    vec2(64., 64.),
                    Rect::new(0., 0., 1., 1.),
                    WHITE,
                );
                (pos, z + 28., angle)
            };
            let model = Mat4::from_translation(vec3(pos.x as f32, pos.y as f32, z as f32))
                * Mat4::from_rotation_y(rotation as f32)
                * Mat4::from_quat(Quat::from_euler(
                    EulerRot::XYZ,
                    if d.frozen { PI / 2. } else { PI / 2.5 },
                    d.rotation as f32,
                    0.,
                ))
                * Mat4::from_scale(vec3(
                    d.scale[0] as f32,
                    d.scale[1] as f32,
                    d.scale[2] as f32,
                ));
            let mut gift = Mesh {
                vertices: self.present.vertices.clone(),
                indices: self.present.indices.clone(),
                texture: Some(self.gifts[d.color].clone()),
            };
            let opacity = d
                .spin
                .map(|s| 1. - ((g.time - s) / 300.).min(1.))
                .unwrap_or(1.) as f32;
            for v in &mut gift.vertices {
                v.position = vp.transform_point3(model.transform_point3(v.position));
                v.color = Color::new(1., 1., 1., opacity).into()
            }
            draw_batched(&gift);
        }
        for (i, p) in g.players.iter().enumerate() {
            if p.gone {
                continue;
            }
            let mut pos = p.pos;
            let mut z = 0.;
            let mut para_rot = 0.;
            if !p.arrived {
                pos = g.map.grid.pos(p.tile);
                let amount = 420. - ((g.time * 0.06).floor() - p.dropped) / 300. * 420.;
                para_rot = 0.1 * (amount / (0.35 * 420.) * std::f64::consts::PI).sin();
                z = amount - 4.;
                let m =
                    Mat4::from_translation(vec3(pos.x as f32, pos.y as f32, (amount + 80.) as f32))
                        * Mat4::from_rotation_y(para_rot as f32);
                Self::quad(
                    &mut parachutes,
                    vp,
                    m * Mat4::from_rotation_x(PI / 6.),
                    vec2(64., 64.),
                    Rect::new(0., 0., 1., 1.),
                    WHITE,
                );
            }
            let mut extra = Mat4::IDENTITY;
            if p.frozen {
                let tick = (g.time - p.ice_start) * 0.06;
                pos = p.ice;
                z = (tick / 10.).cos() * 32. * (1. - (tick / 75.).min(1.).powi(3) * 0.9) - 16.;
                extra = Mat4::from_rotation_z((tick / 500.).sin() as f32 * PI)
            } else if !p.alive {
                z = ((g.time - p.death - 1500.) / 700.).clamp(0., 1.).powi(2) * 64. / 1.5;
            }
            let screen = vp.transform_point3(vec3(pos.x as f32, pos.y as f32, z as f32));
            if screen.x.abs() > 1.3 || screen.y.abs() > 1.5 {
                continue;
            }
            let texture = self.elf_texture(p.color, p.skin, p.hair, p.pony);
            let mut mesh = self.model.as_ref().unwrap().mesh(
                &p.anim,
                ((g.time - p.anim_start) / 1000.) as f32,
                p.throwing.map(|t| ((g.time - t) / 1000.) as f32),
            );
            mesh.texture = Some(texture);
            let model = Mat4::from_translation(vec3(pos.x as f32, pos.y as f32, z as f32))
                * Mat4::from_rotation_y(para_rot as f32)
                * extra
                * Mat4::from_translation(vec3(0., -10., 19.))
                * Mat4::from_quat(Quat::from_euler(
                    EulerRot::XYZ,
                    PI / 2.25,
                    p.face as f32,
                    0.,
                ))
                * Mat4::from_scale(Vec3::splat(5.));
            self.lighting(
                g,
                model,
                if p.alive || p.frozen {
                    1.
                } else {
                    (1. - (g.time - p.death - 1500.) / 700.).clamp(0., 1.) as f32
                },
            );
            gl_use_material(&self.lit);
            draw_batched(&mesh);
            gl_use_material(&self.clip);
            if i == 0 && p.alive {
                let mut ring = Mesh {
                    vertices: vec![],
                    indices: vec![],
                    texture: None,
                };
                let at = if p.arrived {
                    model * Mat4::from_scale(Vec3::splat(0.2)) * Mat4::from_rotation_x(-PI / 2.)
                } else {
                    Mat4::from_translation(vec3(pos.x as f32, pos.y as f32 - 10., 19.))
                        * Mat4::from_rotation_x(PI / 2.25)
                };
                for k in 0..=48 {
                    let a = k as f32 / 48. * 2. * PI;
                    for r in [15., 11.] {
                        ring.vertices.push(Vertex::new2(
                            vp.transform_point3(at.transform_point3(vec3(
                                a.cos() * r,
                                a.sin() * r,
                                0.,
                            ))),
                            Vec2::ZERO,
                            Color::new(0.133, 0.8, 0.533, 0.5),
                        ));
                    }
                    if k > 0 {
                        let n = k * 2;
                        ring.indices.extend([n - 2, n - 1, n, n, n - 1, n + 1]);
                    }
                }
                draw_batched(&ring)
            }
        }
        draw_batched(&parachutes);
        let mut shots = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: Some(assets.scene("snowball", "snowball.png").clone()),
        };
        for s in &g.shots {
            if s.born < 0. {
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
                v.normal = vec4(s.random as f32, 0., 0., 0.);
            }
        }
        self.projectile.set_uniform("Time", g.time as f32);
        gl_use_material(&self.projectile);
        draw_batched(&shots);
        gl_use_material(&self.clip);
        self.effects(assets, g, vp);
        if let Some(p) = g.players.first() {
            if let Some(target) = p.path.last() {
                let center = vp.transform_point3(vec3(target.x as f32, target.y as f32 - 20., 32.));
                let screen = vec2(
                    (center.x + 1.) * screen_width() / 2.,
                    (1. - center.y) * screen_height() / 2.,
                );
                gl_use_default_material();
                let c = Color::new(0.133, 0.8, 0.533, 0.5);
                for direction in [-1., 1.] {
                    draw_line(
                        screen.x - 3.535,
                        screen.y - 2.86 * direction,
                        screen.x + 3.535,
                        screen.y + 2.86 * direction,
                        3.333,
                        c,
                    );
                }
                gl_use_material(&self.clip);
            }
        }

        gl_use_default_material();
        self.powerups(assets, g);
        Ok(())
    }
    fn effects(&self, assets: &Assets, g: &Snowball, vp: Mat4) {
        let mut mesh = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: Some(assets.scene("snowball", "snowball.png").clone()),
        };
        for p in &g.particles {
            let elapsed = ((g.time - p.born) / if p.trail { 700. } else { 300. }) as f32;
            if elapsed >= 1. {
                continue;
            }
            let (offset, size) = if p.trail {
                (
                    vec2(0., -(75. - (PI / 2. + PI / 2. * elapsed).sin() * 75.)),
                    p.size as f32 * (10. - elapsed * 7.5),
                )
            } else {
                let spread = PI / -3. * p.random as f32 + PI / 6.;
                let d = vec2(p.direction.x as f32, p.direction.y as f32);
                let rotated = vec2(
                    spread.cos() * d.x + spread.sin() * d.y,
                    -spread.sin() * d.x + spread.cos() * d.y,
                );
                let mut offset = rotated * ((p.random * 7.158).fract() as f32 * 85. * elapsed);
                offset.y -= (3. * elapsed).powi(2);
                (
                    offset,
                    (p.size / 2. * p.random + p.size / 2. + p.size / 8.) as f32 * (1. - elapsed),
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
                            delta.x * size / screen_width(),
                            delta.y * size / screen_height(),
                            0.,
                        ),
                    uv,
                    WHITE,
                );
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
        self.particle.set_uniform("Time", g.time as f32);
        gl_use_material(&self.particle);
        draw_batched(&mesh);
        let mut ice = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: None,
        };
        let bodies = g
            .players
            .iter()
            .filter(|p| p.frozen && !p.gone)
            .map(|p| (p.ice, p.ice_start))
            .chain(
                g.drops
                    .iter()
                    .filter(|d| d.frozen && !d.gone)
                    .map(|d| (d.ice, d.freeze_at)),
            );
        for (position, started) in bodies {
            let tick = ((g.time - started) * 0.06).floor() as f32;
            let diminish = (tick / 75.).min(1.).powi(3);
            let model = Mat4::from_translation(vec3(
                position.x as f32,
                position.y as f32,
                (tick / 10.).cos() * 32. * (1. - diminish * 0.9),
            )) * Mat4::from_rotation_z((tick / 500.).sin() * PI);
            let n = ice.vertices.len() as u16;
            for v in &self.hex.vertices {
                let pos = v.position;
                let mut out = Vertex::new2(
                    vp.transform_point3(model.transform_point3(vec3(-pos.z, -pos.x, pos.y) * 32.)),
                    v.uv,
                    Color::new(1., 1., 1., diminish),
                );
                out.normal = pos.extend(0.);
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
        unsafe {
            get_internal_gl().quad_gl.scissor(Some((
                0,
                (screen_height() - 265. * responsive) as i32,
                (212. * responsive) as i32,
                (265. * responsive) as i32,
            )));
        }
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
        unsafe {
            get_internal_gl().quad_gl.scissor(None);
        }
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
