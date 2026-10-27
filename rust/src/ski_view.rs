//! Native rendering of the source Elf Ski glTF, sprite points and MeshLine trail.
use crate::assets::Assets;
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Comparison, Equation},
    prelude::*,
};
use santa_tracker::ski::Ski;
use serde_json::Value;

const CLIP_VERTEX: &str = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;attribute vec4 color0;varying vec2 uv;varying vec4 tint;
void main(){gl_Position=vec4(position,1.0);uv=texcoord;tint=color0/255.0;}"#;
const CLIP_FRAGMENT: &str = r#"#version 100
precision highp float;varying vec2 uv;varying vec4 tint;uniform sampler2D Texture;
void main(){vec4 c=texture2D(Texture,uv)*tint;if(c.a<0.1)discard;gl_FragColor=c;}"#;
pub struct SkiView {
    standard: Material,
    phong: Material,
    clip: Material,
    elf: Option<Mesh>,
    present: Mesh,
    snowball: Mesh,
    colors: Vec<Texture2D>,
}
fn pipeline(blend: bool) -> PipelineParams {
    PipelineParams {
        depth_test: Comparison::LessOrEqual,
        depth_write: true,
        color_blend: blend.then_some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        alpha_blend: blend.then_some(BlendState::new(
            Equation::Add,
            BlendFactor::One,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    }
}
pub(crate) fn lit(vertex: &str, fragment: &str, phong: bool) -> Result<Material, String> {
    let mut uniforms = vec![];
    for name in [
        "modelMatrix",
        "modelViewMatrix",
        "projectionMatrix",
        "viewMatrix",
        "NormalMatrix",
        "UvTransform",
    ] {
        uniforms.push(UniformDesc::new(name, UniformType::Mat4));
    }
    for name in [
        "cameraPosition",
        "diffuse",
        "emissive",
        "ambientLightColor",
        "spotLights[0].position",
        "spotLights[0].direction",
        "spotLights[0].color",
    ] {
        uniforms.push(UniformDesc::new(name, UniformType::Float3));
    }
    for name in [
        "opacity",
        "toneMappingExposure",
        "spotLights[0].distance",
        "spotLights[0].decay",
        "spotLights[0].coneCos",
        "spotLights[0].penumbraCos",
    ] {
        uniforms.push(UniformDesc::new(name, UniformType::Float1));
    }
    uniforms.push(UniformDesc::new("isOrthographic", UniformType::Int1));
    if phong {
        uniforms.push(UniformDesc::new("specular", UniformType::Float3));
        uniforms.push(UniformDesc::new("shininess", UniformType::Float1));
    } else {
        uniforms.push(UniformDesc::new("roughness", UniformType::Float1));
        uniforms.push(UniformDesc::new("metalness", UniformType::Float1));
    }
    load_material(
        ShaderSource::Glsl { vertex, fragment },
        MaterialParams {
            uniforms,
            pipeline_params: pipeline(phong),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}
impl SkiView {
    pub fn new() -> Result<Self, String> {
        let standard = lit(
            include_str!("../assets/shaders/ski-standard.vert"),
            include_str!("../assets/shaders/ski-standard.frag"),
            false,
        )?;
        let phong = lit(
            include_str!("../assets/shaders/ski-phong.vert"),
            include_str!("../assets/shaders/ski-phong.frag"),
            true,
        )?;
        let clip = load_material(
            ShaderSource::Glsl {
                vertex: CLIP_VERTEX,
                fragment: CLIP_FRAGMENT,
            },
            MaterialParams {
                pipeline_params: pipeline(true),
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())?;
        let g: Value = serde_json::from_str(include_str!("../data/ski-present-geometry.json"))
            .map_err(|e| e.to_string())?;
        let mut present = Mesh {
            vertices: vec![],
            indices: g["indices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u16)
                .collect(),
            texture: None,
        };
        for i in 0..24 {
            let f = |key: &str, j: usize| g[key][j].as_f64().unwrap() as f32;
            let mut v = Vertex::new(
                f("position", i * 3),
                f("position", i * 3 + 1),
                f("position", i * 3 + 2),
                f("uv", i * 2),
                1. - f("uv", i * 2 + 1),
                WHITE,
            );
            v.normal = vec4(
                f("normal", i * 3),
                f("normal", i * 3 + 1),
                f("normal", i * 3 + 2),
                0.,
            );
            present.vertices.push(v);
        }
        let colors = [
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
                        im.set_pixel(x, y, Color::from_hex(b));
                    }
                }
            }
            let t = Texture2D::from_image(&im);
            crate::assets::configure_texture("scenes/elfski/generated-present.png", &t);
            t
        })
        .collect();
        let mut snowball = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: None,
        };
        // SphereGeometry(5,8,8), including the source pole UV offsets and winding.
        for y in 0..=8 {
            let v = y as f32 / 8.;
            for x in 0..=8 {
                let u = x as f32 / 8.;
                let p = vec3(
                    -5. * (u * std::f32::consts::TAU).cos() * (v * std::f32::consts::PI).sin(),
                    5. * (v * std::f32::consts::PI).cos(),
                    5. * (u * std::f32::consts::TAU).sin() * (v * std::f32::consts::PI).sin(),
                );
                snowball.vertices.push(Vertex::new2(p, vec2(u, v), WHITE));
            }
        }
        for y in 0..8 {
            for x in 0..8 {
                let (a, b, c, d) = (
                    (y * 9 + x + 1) as u16,
                    (y * 9 + x) as u16,
                    ((y + 1) * 9 + x) as u16,
                    ((y + 1) * 9 + x + 1) as u16,
                );
                if y != 0 {
                    snowball.indices.extend([a, b, d]);
                }
                if y != 7 {
                    snowball.indices.extend([b, c, d]);
                }
            }
        }
        Ok(Self {
            standard,
            phong,
            clip,
            elf: None,
            present,
            snowball,
            colors,
        })
    }
    pub fn input(block_mouse: bool) -> Option<f64> {
        if !block_mouse
            && is_mouse_button_down(MouseButton::Left)
            && !crate::chrome::over_controls(Vec2::from(mouse_position()), 3)
        {
            return Some(
                ((mouse_position().0 - screen_width() / 2.) as f64)
                    .atan2((mouse_position().1 - screen_height() / 3. + 40. / 3.) as f64),
            );
        }
        let l = u8::from(is_key_down(KeyCode::Left));
        let r = u8::from(is_key_down(KeyCode::Right));
        let d = u8::from(is_key_down(KeyCode::Down));
        (l + r + d > 0)
            .then(|| (r as f64 - l as f64) * std::f64::consts::FRAC_PI_2 / (l + r + d) as f64)
    }
    fn load_elf(&mut self, assets: &Assets) -> Result<(), String> {
        if self.elf.is_some() {
            return Ok(());
        }
        let root = assets.root.join("scenes/elfski/models");
        let doc: Value = serde_json::from_slice(
            &std::fs::read(root.join("elf-ski.gltf")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let data = std::fs::read(root.join("elf-ski.bin")).map_err(|e| e.to_string())?;
        let primitive = &doc["meshes"][0]["primitives"][0];
        let accessor = |index: usize, n: usize| -> Result<Vec<f32>, String> {
            let a = &doc["accessors"][index];
            if a["componentType"].as_u64() != Some(5126) {
                return Err("Elf glTF expected f32 accessor".into());
            }
            let v = &doc["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
            let start = v["byteOffset"].as_u64().unwrap_or(0) as usize
                + a["byteOffset"].as_u64().unwrap_or(0) as usize;
            let count = a["count"].as_u64().unwrap() as usize;
            let stride = v["byteStride"].as_u64().unwrap_or((n * 4) as u64) as usize;
            let mut out = vec![];
            for i in 0..count {
                for j in 0..n {
                    let offset = start + i * stride + j * 4;
                    let bytes = data.get(offset..offset + 4).ok_or("Elf glTF bounds")?;
                    out.push(f32::from_le_bytes(bytes.try_into().unwrap()));
                }
            }
            Ok(out)
        };
        let positions = accessor(
            primitive["attributes"]["POSITION"].as_u64().unwrap() as usize,
            3,
        )?;
        let normals = accessor(
            primitive["attributes"]["NORMAL"].as_u64().unwrap() as usize,
            3,
        )?;
        let uv = accessor(
            primitive["attributes"]["TEXCOORD_0"].as_u64().unwrap() as usize,
            2,
        )?;
        let a = &doc["accessors"][primitive["indices"].as_u64().unwrap() as usize];
        if a["componentType"].as_u64() != Some(5123) {
            return Err("Elf glTF expected u16 indices".into());
        }
        let v = &doc["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
        let offset = v["byteOffset"].as_u64().unwrap_or(0) as usize
            + a["byteOffset"].as_u64().unwrap_or(0) as usize;
        let mut indices = vec![];
        for i in 0..a["count"].as_u64().unwrap() as usize {
            let p = data
                .get(offset + i * 2..offset + i * 2 + 2)
                .ok_or("Elf indices bounds")?;
            indices.push(u16::from_le_bytes(p.try_into().unwrap()));
        }
        let vertices = positions
            .chunks_exact(3)
            .enumerate()
            .map(|(i, p)| {
                let mut v = Vertex::new(p[0], p[1], p[2], uv[i * 2], uv[i * 2 + 1], WHITE);
                v.normal = vec4(normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2], 0.);
                v
            })
            .collect();
        self.elf = Some(Mesh {
            vertices,
            indices,
            texture: Some(assets.texture("scenes/elfski/models/elf-tidy.png").clone()),
        });
        Ok(())
    }
    fn lighting(
        material: &Material,
        game: &Ski,
        view: Mat4,
        projection: Mat4,
        model: Mat4,
        phong: bool,
    ) {
        for (k, m) in [
            ("modelMatrix", model),
            ("modelViewMatrix", view * model),
            ("projectionMatrix", projection),
            ("viewMatrix", view),
            ("NormalMatrix", (view * model).inverse().transpose()),
            ("UvTransform", Mat4::IDENTITY),
        ] {
            material.set_uniform(k, m);
        }
        let focus = vec3(game.camera_y as f32, 0., -game.camera_x as f32);
        material.set_uniform("cameraPosition", focus + vec3(800., 800., 0.));
        material.set_uniform("isOrthographic", 1i32);
        material.set_uniform("diffuse", Vec3::splat(if phong { 1. } else { 0.64 }));
        material.set_uniform("emissive", Vec3::ZERO);
        material.set_uniform("ambientLightColor", Vec3::splat(2. / 3.));
        for (name, value) in [
            ("opacity", 1.),
            ("toneMappingExposure", 1.),
            ("spotLights[0].distance", 0.),
            ("spotLights[0].decay", 1.),
            ("spotLights[0].coneCos", 0.5),
            ("spotLights[0].penumbraCos", 0.5),
        ] {
            material.set_uniform(name, value as f32);
        }
        // The spotlight stops following at the first crash, just like _internalTick.
        let a = game.facing_angle as f32;
        let player = vec3(
            (game.camera_y - game.height / 4. - (a.cos() as f64) * 50.) as f32,
            24.,
            -(game.camera_x - (a.sin() as f64) * 10.) as f32,
        );
        let spot = player + vec3(a.cos() * 10., 72., -a.sin() * 10.);
        // THREE SpotLight has its own default target at the world origin; the
        // source assigns _spottarget but never sets spot.target to the skier.
        material.set_uniform("spotLights[0].position", view.transform_point3(spot));
        material.set_uniform(
            "spotLights[0].direction",
            view.transform_vector3(spot.normalize()).normalize(),
        );
        material.set_uniform("spotLights[0].color", Vec3::splat(2.));
        if phong {
            material.set_uniform("specular", Vec3::splat(0x11 as f32 / 255.));
            material.set_uniform("shininess", 30f32);
        } else {
            material.set_uniform("roughness", 1f32);
            material.set_uniform("metalness", 0f32);
        }
    }
    pub fn draw(&mut self, assets: &Assets, game: &mut Ski) -> Result<(), String> {
        self.load_elf(assets)?;
        game.render_update(
            (screen_width() + 80.) as f64,
            (screen_height() + 80.) as f64,
        );
        clear_background(Color::from_hex(0xf5f2e2));
        let focus = vec3(game.camera_y as f32, 0., -game.camera_x as f32);
        let view = Mat4::look_at_rh(focus + vec3(800., 800., 0.), focus, Vec3::Y);
        let original_projection = Mat4::orthographic_rh_gl(
            -game.width as f32 / 2.,
            game.width as f32 / 2.,
            -game.height as f32 / 2.,
            game.height as f32 / 2.,
            1.,
            2000.,
        );
        let crop = vec2(
            game.width as f32 / screen_width(),
            game.height as f32 / screen_height(),
        );
        let projection = Mat4::from_scale(crop.extend(1.)) * original_projection;
        let vp = projection * view;
        gl_use_material(&self.clip);
        for (offset, line_width, color) in [
            (vec3(-10., 0., 0.), 0.015, Color::from_hex(0xcccccc)),
            (vec3(0., 6., 0.), 0.01, WHITE),
        ] {
            let points: Vec<Vec3> = game
                .trail
                .iter()
                .map(|(x, y)| {
                    (original_projection * view)
                        .transform_point3(vec3(*y as f32, 16., -*x as f32) + offset)
                })
                .collect();
            let mut mesh = Mesh {
                vertices: vec![],
                indices: vec![],
                texture: None,
            };
            for i in 0..points.len() {
                let current = points[i];
                let prev = points[i.saturating_sub(1)];
                let next = points[(i + 1).min(points.len() - 1)];
                let dir = if next == current {
                    (current - prev).truncate().normalize_or_zero()
                } else if prev == current {
                    (next - current).truncate().normalize_or_zero()
                } else {
                    ((current - prev).truncate().normalize_or_zero()
                        + (next - current).truncate().normalize_or_zero())
                    .normalize_or_zero()
                };
                let point = i as f32 / (points.len() - 1) as f32;
                let mut width = (point * std::f32::consts::FRAC_PI_2).sin();
                if point * std::f32::consts::PI > 2. {
                    width = width.min((point * std::f32::consts::PI + 4.).cos());
                }
                let normal = vec2(-dir.y, dir.x) * (0.9 * line_width * width);
                for side in [-1., 1.] {
                    let p = (current.truncate() + normal * side) * crop;
                    mesh.vertices
                        .push(Vertex::new2(p.extend(current.z), Vec2::ZERO, color));
                }
                if i > 0 {
                    let n = (i * 2) as u16;
                    mesh.indices.extend([n - 2, n - 1, n, n, n - 1, n + 1]);
                }
            }
            draw_mesh(&mesh);
        }
        let q = game.rotation();
        let rotation = Quat::from_xyzw(q[0] as f32, q[1] as f32, q[2] as f32, q[3] as f32);
        let model = Mat4::from_scale_rotation_translation(
            Vec3::splat(5.),
            rotation,
            vec3(game.y as f32, 24., -game.x as f32),
        );
        Self::lighting(&self.standard, game, view, projection, model, false);
        gl_use_material(&self.standard);
        draw_mesh(self.elf.as_ref().unwrap());
        if let Some(hit) = game.hit_tree {
            let snow_model = model
                * Mat4::from_translation(vec3(0., -0.5, 0.))
                * Mat4::from_scale(Vec3::splat((hit as f32 / 2.).max(0.001)));
            let mut sphere = Mesh {
                vertices: self.snowball.vertices.clone(),
                indices: self.snowball.indices.clone(),
                texture: None,
            };
            for v in &mut sphere.vertices {
                v.position = vp.transform_point3(snow_model.transform_point3(v.position));
            }
            gl_use_material(&self.clip);
            draw_mesh(&sphere);
        }
        // Source Points is one transparent draw call, with individual points at a
        // fixed depth. Billboards preserve that depth instead of standing in 3D.
        let mut trees = Mesh {
            vertices: vec![],
            indices: vec![],
            texture: Some(assets.scene("elfski", "tiles.png").clone()),
        };
        for tree in game.trees.iter().flatten() {
            let p = vp.transform_point3(vec3(tree.y as f32, 72., -tree.x as f32));
            if p.x.abs() > 1.2 || p.y.abs() > 1.3 || p.z.abs() > 1. {
                continue;
            }
            let uv = vec2((tree.sprite % 4) as f32 / 4., (tree.sprite / 4) as f32 / 4.);
            let i = trees.vertices.len() as u16;
            for (delta, t) in [
                (vec2(-1., 1.), vec2(0., 0.)),
                (vec2(1., 1.), vec2(0.25, 0.)),
                (vec2(1., -1.), vec2(0.25, 0.25)),
                (vec2(-1., -1.), vec2(0., 0.25)),
            ] {
                trees.vertices.push(Vertex::new2(
                    vec3(
                        p.x + delta.x * 128. / screen_width(),
                        p.y + delta.y * 128. / screen_height(),
                        p.z,
                    ),
                    uv + t,
                    WHITE,
                ));
            }
            trees.indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
        }
        gl_use_material(&self.clip);
        draw_mesh(&trees);
        let mut presents: Vec<_> = game.presents.iter().filter(|p| p.attached).collect();
        presents.sort_by(|a, b| {
            let z = |p: &santa_tracker::ski::Present| {
                view.transform_point3(vec3(p.y as f32, 0., -p.x as f32)).z
            };
            z(a).total_cmp(&z(b))
        });
        for p in presents {
            let model = Mat4::from_translation(vec3(p.y as f32, 0., -p.x as f32))
                * Mat4::from_scale(Vec3::splat(p.scale as f32))
                * Mat4::from_translation(vec3(0., p.lift as f32, 0.))
                * Mat4::from_rotation_y(p.rotation as f32)
                * Mat4::from_scale(vec3(p.size[0] as f32, p.size[1] as f32, p.size[2] as f32));
            Self::lighting(&self.phong, game, view, projection, model, true);
            self.present.texture = Some(self.colors[p.color].clone());
            gl_use_material(&self.phong);
            draw_mesh(&self.present);
        }
        gl_use_default_material();
        Ok(())
    }
}
