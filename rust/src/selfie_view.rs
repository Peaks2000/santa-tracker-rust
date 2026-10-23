use crate::assets::Assets;
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};
use santa_tracker::selfie::{Selfie, Tool};

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
varying vec2 uv;
varying vec4 color;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;color=color0/255.0;}
"#;
const PREMULT: &str = r#"#version 100
precision mediump float;
varying vec2 uv;varying vec4 color;uniform sampler2D Texture;
void main(){vec4 c=texture2D(Texture,uv)*color;gl_FragColor=vec4(c.rgb*c.a,c.a);}
"#;
const COPY: &str = r#"#version 100
precision mediump float;
varying vec2 uv;varying vec4 color;uniform sampler2D Texture;
void main(){gl_FragColor=texture2D(Texture,uv)*color;}
"#;
const SPRAYS: [&str; 8] = [
    "red", "orange", "yellow", "green", "cyan", "purple", "pink", "blue",
];
const DECORATIONS: [(&str, f32, f32); 4] = [
    ("snowman", 84., 138.),
    ("bauble", 75., 128.),
    ("bow", 127., 94.),
    ("holly", 120., 138.),
];

pub struct SelfieView {
    hair: Texture2D,
    target: Option<RenderTarget>,
    hair_material: Material,
    spray_material: Material,
    copy_material: Material,
    font: Font,
    pub export_open: bool,
    pub export_message: String,
    export_pending: bool,
    tool_used: f64,
}
fn rect_image(t: &Texture2D, r: Rect) {
    draw_texture_ex(
        t,
        r.x,
        r.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(r.size()),
            ..Default::default()
        },
    );
}
impl SelfieView {
    pub fn new() -> Result<Self, String> {
        let tree = resvg::usvg::Tree::from_data(
            include_bytes!("../data/selfie-hair.svg"),
            &Default::default(),
        )
        .map_err(|e| e.to_string())?;
        let mut pix = resvg::tiny_skia::Pixmap::new(212, 208).ok_or("Hair allocation failed")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(4., 4.),
            &mut pix.as_mut(),
        );
        let mut bytes = pix.data().to_vec();
        for p in bytes.chunks_exact_mut(4) {
            // Straight-alpha filtering must see white even in transparent
            // texels; black RGB there produces dark speckles between hairs.
            p[0] = 255;
            p[1] = 255;
            p[2] = 255;
        }
        let hair = Texture2D::from_rgba8(212, 208, &bytes);
        hair.set_filter(FilterMode::Linear);
        let hair_material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: COPY,
            },
            MaterialParams {
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
        let spray_material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: PREMULT,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::DestinationAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    alpha_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Zero,
                        BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|e| e.to_string())?;
        let copy_material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: COPY,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::One,
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
        Ok(Self {
            hair,
            target: None,
            hair_material,
            spray_material,
            copy_material,
            font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/GoogleSans-Regular.ttf"
            ))
            .map_err(|e| e.to_string())?,
            export_open: false,
            export_pending: false,
            export_message: String::new(),
            tool_used: -100.,
        })
    }
    pub fn scale() -> f32 {
        ((screen_width() + 1420.) / 3340.).min(screen_height() / 985.)
    }
    fn face_rect(game: &Selfie, mouse: Vec2) -> Rect {
        let s = Self::scale();
        let rel = vec2(
            2. * mouse.x / screen_width() - 1.,
            2. * mouse.y / screen_height() - 1.,
        );
        let _ = game;
        Rect::new(
            screen_width() / 2. - 279. * s + rel.x * 10. * s,
            screen_height() - 917. * s + rel.y * 5. * s,
            558. * s,
            497. * s,
        )
    }
    fn tools() -> Vec<(Tool, &'static str, f32, f32)> {
        let mut tools = vec![
            (Tool::Clipper, "clipper.svg", 47., 99.),
            (Tool::Grow, "hairgrow.svg", 85., 98.),
            (Tool::Clean, "hairclean.svg", 69., 105.),
            (Tool::Dryer, "hairdryer--left.svg", 84., 98.),
        ];
        for (i, name) in SPRAYS.iter().enumerate() {
            let file = match *name {
                "red" => "spray--red.svg",
                "orange" => "spray--orange.svg",
                "yellow" => "spray--yellow.svg",
                "green" => "spray--green.svg",
                "cyan" => "spray--cyan.svg",
                "purple" => "spray--purple.svg",
                "pink" => "spray--pink.svg",
                _ => "spray--blue.svg",
            };
            tools.push((Tool::Spray(i as u8), file, 27., 56.));
        }
        tools.extend([
            (Tool::Decoration(0), "decorations/snowman.svg", 33., 54.),
            (Tool::Decoration(1), "decorations/bauble.svg", 27., 46.),
            (Tool::Decoration(2), "decorations/bow.svg", 41., 31.),
            (Tool::Decoration(3), "decorations/holly.svg", 39., 45.),
        ]);
        tools
    }
    fn tool_rect(index: usize) -> Rect {
        if screen_width() < 800. || screen_height() < 690. {
            let x = screen_width() - 330.;
            let y = screen_height() - 106.;
            return Rect::new(
                x + (index % 8) as f32 * 40.,
                y + (index / 8) as f32 * 48.,
                40.,
                48.,
            );
        }
        let x = 20. + 232.3 - 208. - 15.;
        let top = screen_height() / 2. - 290.;
        let row_gap = (580. - (2. * 156. + 3. * 78. + 20.)) / 5.;
        if index < 4 {
            Rect::new(
                x + (index % 2) as f32 * 104.,
                top + (index / 2) as f32 * (156. + row_gap) + if index < 2 { 30. } else { 0. },
                104.,
                156.,
            )
        } else {
            let i = index - 4;
            Rect::new(
                x + (i % 4) as f32 * 52.,
                top + 2. * (156. + row_gap) - 15. + (i / 4) as f32 * (78. + row_gap),
                52.,
                78.,
            )
        }
    }
    pub fn input(&mut self, game: &mut Selfie, deterministic: bool) {
        let mouse = if deterministic {
            vec2(0., 0.)
        } else {
            Vec2::from(mouse_position())
        };
        let pressed = is_mouse_button_pressed(MouseButton::Left);
        let released = is_mouse_button_released(MouseButton::Left);
        if self.export_open {
            if is_key_pressed(KeyCode::Escape) || pressed {
                self.export_open = false;
            }
            return;
        }
        for (i, (tool, _, _, _)) in Self::tools().iter().enumerate() {
            if pressed && Self::tool_rect(i).contains(mouse) {
                game.select(*tool);
                game.mouse.down = false;
                game.update_tool_sounds(mouse.x as f64, false);
                self.tool_used = game.time;
                return;
            }
        }
        let reset = Rect::new(screen_width() - 82., 99., 55., 55.);
        let share = Rect::new(screen_width() - 82., 169., 55., 55.);
        if pressed && reset.contains(mouse) {
            game.reset();
            game.sounds.push("selfie_click");
            return;
        }
        if pressed && share.contains(mouse) {
            self.export_open = true;
            self.export_pending = true;
            game.tool = Tool::Hand;
            game.sounds.push("selfie_click");
            return;
        }
        if is_key_pressed(KeyCode::Key0) {
            game.select(Tool::Hand);
        }
        if is_key_pressed(KeyCode::Key1) {
            game.select(Tool::Clipper);
        }
        if is_key_pressed(KeyCode::Key2) {
            game.select(Tool::Grow);
        }
        if is_key_pressed(KeyCode::Key3) {
            game.select(Tool::Clean);
        }
        if is_key_pressed(KeyCode::Key4) {
            game.select(Tool::Dryer);
        }
        let face = Self::face_rect(game, mouse);
        let s = Self::scale();
        let cw = (1200. * s).floor();
        let ch = (1400. * s).floor();
        let beard = vec2(
            face.x + face.w / 2. - cw / 2.,
            face.y + face.h / 2. - ch / 2.,
        );
        let over_tool = Self::tools()
            .iter()
            .enumerate()
            .any(|(i, _)| Self::tool_rect(i).contains(mouse));
        let down = is_mouse_button_down(MouseButton::Left)
            && !crate::chrome::over_controls(mouse, 2)
            && !over_tool;
        game.pointer(
            (mouse.x - beard.x) as f64,
            (mouse.y - beard.y) as f64,
            down,
            s as f64,
            released,
        );
        game.update_tool_sounds(mouse.x as f64, down);
        if pressed && down {
            self.tool_used = game.time;
        }
    }
    fn beard(&mut self, assets: &Assets, game: &Selfie, rect: Rect, s: f32) {
        let width = (1200. * s).floor().max(1.) as u32;
        let height = (1400. * s).floor().max(1.) as u32;
        if self.target.as_ref().is_none_or(|r| {
            r.texture.width() != width as f32 || r.texture.height() != height as f32
        }) {
            let target = render_target(width, height);
            target.texture.set_filter(FilterMode::Linear);
            self.target = Some(target);
        }
        let target = self.target.as_ref().unwrap();
        let mut camera =
            Camera2D::from_display_rect(Rect::new(0., 0., width as f32, height as f32));
        camera.render_target = Some(target.clone());
        set_camera(&camera);
        clear_background(BLANK);
        let hair_w = (53. * s).floor();
        let hair_h = (52. * s).floor();
        gl_use_material(&self.hair_material);
        for p in game
            .points
            .iter()
            .rev()
            .filter(|p| p.draw && p.x.is_finite() && p.y.is_finite())
        {
            draw_texture_ex(
                &self.hair,
                p.x as f32 * s - hair_w / 2.,
                p.y as f32 * s - hair_h / 2.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(hair_w, hair_h)),
                    rotation: p.rotation as f32,
                    ..Default::default()
                },
            );
        }
        gl_use_material(&self.spray_material);
        for p in game.points.iter().rev().filter(|p| p.draw) {
            if let Some(spray) = p.spray {
                let key = format!("tools/sprays/{}.png", SPRAYS[spray as usize]);
                rect_image(
                    assets.scene("santaselfie", &key),
                    Rect::new(
                        (p.x as f32 - 64.) * s,
                        (p.y as f32 - 64.) * s,
                        128. * s,
                        128. * s,
                    ),
                );
            }
        }
        gl_use_material(&self.hair_material);
        for p in game.points.iter().rev().filter(|p| p.draw) {
            if let Some(decoration) = p.decoration {
                let (name, w, h) = DECORATIONS[decoration as usize];
                rect_image(
                    assets.scene("santaselfie", &format!("tools/decorations/{name}.svg")),
                    Rect::new(
                        (p.x as f32 - w / 2.) * s,
                        (p.y as f32 - h / 2.) * s,
                        w * s,
                        h * s,
                    ),
                );
            }
        }
        set_default_camera();
        gl_use_material(&self.copy_material);
        draw_texture_ex(
            &target.texture,
            rect.x + rect.w / 2. - width as f32 / 2.,
            rect.y + rect.h / 2. - height as f32 / 2.,
            WHITE,
            DrawTextureParams {
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
    pub fn draw(&mut self, assets: &Assets, game: &Selfie, deterministic: bool) {
        let s = Self::scale();
        let w = screen_width();
        let h = screen_height();
        let time = game.time as f32;
        clear_background(Color::from_hex(0x00adeb));
        let draw = |name: &str, r: Rect| rect_image(assets.scene("santaselfie", name), r);
        let mirror = Rect::new(w / 2. - 1092.4 * s / 2., h - 852. * s, 1092.4 * s, 732. * s);
        draw("background/mirror.svg", mirror);
        draw(
            "background/photos.svg",
            Rect::new(
                mirror.x + 1294. * s,
                mirror.y + 297. * s,
                306. * s,
                380. * s,
            ),
        );
        if w > 800. && h > 690. {
            let pole = Rect::new(mirror.x + 1000. * s, mirror.y + 23. * s, 180. * s, 380. * s);
            draw(
                "background/pole--background.svg",
                Rect::new(pole.x + 31. * s, pole.y + s, 118. * s, 378. * s),
            );
            let r = Rect::new(pole.x + 43. * s, pole.y + 62. * s, 94. * s, 257. * s);
            unsafe {
                get_internal_gl()
                    .quad_gl
                    .scissor(Some((r.x as i32, r.y as i32, r.w as i32, r.h as i32)));
            }
            draw(
                "background/pole--stripes.svg",
                Rect::new(
                    r.x,
                    r.y + (-0.39 + (time / 5.).fract() * 0.39) * 600. * s,
                    94. * s,
                    600. * s,
                ),
            );
            unsafe {
                get_internal_gl().quad_gl.scissor(None);
            }
            draw("background/pole--foreground.svg", pole);
        }
        draw(
            "background/chair.svg",
            Rect::new(w / 2. - 557. * s, h - 510. * s, 1114. * s, 862. * s),
        );
        let phase = (time / 3.).fract();
        let breathing = if phase < 0.3 {
            -7. * crate::village::css_ease(phase / 0.3)
        } else if phase < 0.72 {
            -7. * (1. - crate::village::css_ease((phase - 0.3) / 0.42))
        } else {
            0.
        };
        draw(
            "santa/body.svg",
            Rect::new(
                w / 2. - 526. * s,
                h + (1052. * 0.56 - 1068. + breathing) * s,
                1052. * s,
                1068. * s,
            ),
        );
        let mouse = if deterministic {
            Vec2::ZERO
        } else {
            Vec2::from(mouse_position())
        };
        let rel = vec2(2. * mouse.x / w - 1., 2. * mouse.y / h - 1.);
        let face = Self::face_rect(game, mouse);
        draw("santa/face.svg", face);
        let dryer = game.tool == Tool::Dryer && game.mouse.down;
        let alarmed = game.tool == Tool::Clipper;
        for side in 0..2 {
            let ex = if side == 0 { 164. } else { 359. };
            let closed = game.blink_remaining > 0.
                || dryer
                    && (rel.x.abs() < 0.3 || side == 0 && rel.x < -0.3 || side == 1 && rel.x > 0.3);
            draw(
                if closed {
                    "face/eye--blink.svg"
                } else {
                    "face/eye.svg"
                },
                Rect::new(
                    face.x + (ex + rel.x * 10.) * s,
                    face.y + (145. + rel.y * 5.) * s,
                    35. * s,
                    35. * s,
                ),
            );
            let brow_x = if side == 0 { 132. } else { 333. };
            let raise = if alarmed {
                let t = (time - game.selected_at as f32).clamp(0., 0.6);
                if t < 0.18 {
                    -20. * crate::village::css_ease(t / 0.18)
                } else if t < 0.42 {
                    -20.
                } else {
                    -20. * (1. - crate::village::css_ease((t - 0.42) / 0.18))
                }
            } else {
                0.
            };
            let r = Rect::new(
                face.x + brow_x * s,
                face.y + (95. + raise) * s,
                93. * s,
                22. * s,
            );
            crate::village::rounded(r, 8. * s, WHITE);
        }
        self.beard(assets, game, face, s);
        let squash = alarmed
            && game.mouse.down
            && (mouse.x - (face.x + 279. * s)).hypot(mouse.y - 86. - (face.y + 222. * s)) < 120.;
        draw(
            "face/nose.svg",
            Rect::new(
                face.x + 215.5 * s,
                face.y + (170. + if squash { 15.6 } else { 0. }) * s,
                127. * s,
                104. * s * if squash { 0.7 } else { 1. },
            ),
        );
        let (mouth, mx, mw, mh) = if alarmed {
            ("face/mouth--breath.svg", 250.5, 57., 57.)
        } else if dryer {
            (
                "face/mouth--breath.svg",
                if rel.x < -0.3 {
                    304.
                } else if rel.x > 0.3 {
                    234.
                } else {
                    259.
                },
                40.,
                40.,
            )
        } else {
            ("face/mouth--smile.svg", 234., 90., 45.)
        };
        draw(
            mouth,
            Rect::new(face.x + mx * s, face.y + 295. * s, mw * s, mh * s),
        );
        self.draw_tools(assets, game, mouse, s);
        for (y, key) in [(105., "refresh"), (175., "share")] {
            let mouse = if deterministic {
                Vec2::ZERO
            } else {
                Vec2::from(mouse_position())
            };
            let pressed = is_mouse_button_down(MouseButton::Left)
                && Rect::new(w - 82., y - 6., 55., 55.).contains(mouse);
            let offset = if pressed { vec2(3., 6.) } else { Vec2::ZERO };
            let center = vec2(w - 54.5, y + 21.5) + offset;
            if !pressed {
                draw_poly(
                    center.x + 8.,
                    center.y + 9.,
                    96,
                    27.5,
                    0.,
                    Color::new(0., 0., 0., 0.16),
                );
                draw_poly(
                    center.x + 3.,
                    center.y + 6.,
                    96,
                    27.5,
                    0.,
                    Color::from_hex(0x8f0093),
                );
            }
            draw_poly(center.x, center.y, 96, 27.5, 0., Color::from_hex(0xad00ad));
            rect_image(
                assets.texture(&format!("scenes/_shared/img/button/{key}.svg")),
                Rect::new(center.x - 15., center.y - 15., 30., 30.),
            );
        }
        if self.export_open {
            // Capture/export is intentionally user-triggered; the original
            // beard encoding is preserved alongside the exported image.
            if self.export_pending {
                let result = crate::capture::save_png(
                    &get_screen_data(),
                    std::path::Path::new("santa-selfie.png"),
                )
                .and_then(|_| {
                    std::fs::write("santa-selfie.beard", game.save()).map_err(|e| e.to_string())
                });
                self.export_message = match result {
                    Ok(()) => "Saved santa-selfie.png and santa-selfie.beard".into(),
                    Err(error) => format!("Could not save selfie: {error}"),
                };
                self.export_pending = false;
            }
            draw_rectangle(0., 0., w, h, Color::new(0., 0., 0., 0.5));
            draw_rectangle(w / 2. - 300., h / 2. - 80., 600., 160., WHITE);
            for (i, line) in [
                "Santa Selfie export",
                self.export_message.as_str(),
                "Click or press Escape to return",
            ]
            .iter()
            .enumerate()
            {
                let size = if i == 0 { 28 } else { 18 };
                let width = measure_text(line, Some(&self.font), size, 1.).width;
                draw_text_ex(
                    line,
                    w / 2. - width / 2.,
                    h / 2. - 30. + i as f32 * 38.,
                    TextParams {
                        font: Some(&self.font),
                        font_size: size,
                        color: DARKGRAY,
                        ..Default::default()
                    },
                );
            }
        }
    }
    fn draw_tools(&self, assets: &Assets, game: &Selfie, mouse: Vec2, s: f32) {
        let desktop = screen_width() >= 800. && screen_height() >= 690.;
        if desktop {
            rect_image(
                assets.scene("santaselfie", "tray_lg.svg"),
                Rect::new(20., screen_height() / 2. - 290., 232.3, 580.),
            );
            rect_image(
                assets.scene("santaselfie", "tray_ornament_lg.svg"),
                Rect::new(
                    17. + 232.3 * 0.05,
                    screen_height() / 2. - 290. + 2.,
                    232.3 * 0.9,
                    36.,
                ),
            );
        } else {
            draw_rectangle(
                screen_width() - 330.,
                screen_height() - 106.,
                330.,
                106.,
                Color::from_hex(0xb444bb),
            );
        }
        for (i, (tool, file, tw, th)) in Self::tools().iter().enumerate() {
            let r = Self::tool_rect(i);
            let shadow = match tool {
                Tool::Clipper => "clipper",
                Tool::Grow => "hairgrow",
                Tool::Clean => "hairclean",
                Tool::Dryer => "hairdryer",
                Tool::Spray(_) => "spray",
                Tool::Decoration(n) => DECORATIONS[*n as usize].0,
                _ => "clipper",
            };
            if desktop {
                rect_image(
                    assets.scene("santaselfie", &format!("tools/shadows/{shadow}.svg")),
                    Rect::new(
                        r.x + (r.w - tw + 1.) / 2.,
                        r.y + (r.h - th + 1.) / 2.,
                        tw - 1.,
                        th - 1.,
                    ),
                );
            }
            if game.tool != *tool {
                rect_image(
                    assets.scene("santaselfie", &format!("tools/{file}")),
                    Rect::new(r.x + (r.w - tw) / 2., r.y + (r.h - th) / 2., *tw, *th),
                );
            }
        }
        if game.tool == Tool::Hand {
            return;
        }
        let rel = 2. * mouse.x / screen_width() - 1.;
        let (file, width, height, ox, oy, flip) = match game.tool {
            Tool::Clipper => ("tools/clipper.svg".into(), 80., 172., 40., 0., false),
            Tool::Grow => ("tools/hairgrow.svg".into(), 140., 161., 110., 25., rel > 0.),
            Tool::Clean => (
                "tools/hairclean.svg".into(),
                115.,
                175.,
                120.,
                10.,
                rel > 0.,
            ),
            Tool::Dryer => {
                let center = rel.abs() <= 0.3;
                (
                    if center {
                        "tools/hairdryer--center.svg"
                    } else {
                        "tools/hairdryer--left.svg"
                    }
                    .into(),
                    if center { 143. } else { 221. },
                    if center { 260. } else { 257. },
                    100.,
                    0.,
                    rel > 0.3,
                )
            }
            Tool::Spray(i) => (
                format!("tools/spray--{}.svg", SPRAYS[i as usize]),
                93.,
                196.,
                47.,
                0.,
                false,
            ),
            Tool::Decoration(i) => {
                let (name, w, h) = DECORATIONS[i as usize];
                (
                    format!("tools/decorations/{name}.svg"),
                    w,
                    h,
                    if i == 2 { 50. } else { 40. },
                    if i < 2 { 50. } else { 45. },
                    false,
                )
            }
            Tool::Hand => return,
        };
        let offset = if flip { width - ox } else { ox };
        let x = mouse.x - offset * s;
        let y = mouse.y - oy * s;
        draw_texture_ex(
            assets.scene("santaselfie", &file),
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(width * s, height * s)),
                flip_x: flip,
                ..Default::default()
            },
        );
        if game.mouse.down {
            let (sprite, fw, fh, frames, duration, px, py) = match game.tool {
                Tool::Clipper if game.near_beard => {
                    ("sprites/clipper.png", 230., 150., 14., 0.46, -70., -80.)
                }
                Tool::Clean => ("sprites/hairclean.png", 230., 150., 14., 0.46, 20., -60.),
                Tool::Grow => ("sprites/hairgrow.png", 350., 350., 8., 0.23, -50., -140.),
                _ => return,
            };
            let age = (game.time - self.tool_used) as f32;
            if game.tool == Tool::Grow && age > duration {
                return;
            }
            let frame = ((age / duration).fract() * frames).floor();
            draw_texture_ex(
                assets.scene("santaselfie", sprite),
                x + px * s,
                y + py * s,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(fw * s, fh * s)),
                    source: Some(Rect::new(0., frame * fh, fw, fh)),
                    flip_x: flip,
                    ..Default::default()
                },
            );
        }
    }
}
