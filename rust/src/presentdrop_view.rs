//! Present Drop HTML/SCSS geometry and animations, drawn in native world units.
use crate::assets::Assets;
use macroquad::prelude::*;
use santa_tracker::presentdrop::{Entity, PresentDrop};
pub struct PresentDropView {
    font: Font,
    touch: bool,
}
impl PresentDropView {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            font: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/OpenSans-Bold.ttf"))
                .map_err(|e| e.to_string())?,
            touch: false,
        })
    }
    pub fn scale() -> f32 {
        let w = screen_width();
        let h = screen_height();
        let s = if w < 890. { w / 890. } else { 1. };
        if h < 660. {
            (h / 640.).min(s)
        } else {
            s
        }
    }
    pub fn stage() -> Vec2 {
        let s = Self::scale();
        vec2(
            screen_width() / 2. - 444.5 * s,
            screen_height() / 2. - 5.5 - 324.5 * s,
        )
    }
    pub fn input(&mut self, g: &mut PresentDrop) {
        // Source key handlers update targets on key changes, including pause.
        if [KeyCode::Left, KeyCode::Right, KeyCode::Space, KeyCode::Down]
            .into_iter()
            .any(|k| is_key_pressed(k) || is_key_released(k))
        {
            g.keyboard(is_key_down(KeyCode::Left), is_key_down(KeyCode::Right));
        }
        if g.playing
            && !g.paused
            && (is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Down))
        {
            g.drop_present();
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && g.playing
            && !g.paused
            && !crate::chrome::over_controls(Vec2::from(mouse_position()), 3)
        {
            self.touch = true;
        }
        if self.touch {
            if is_mouse_button_down(MouseButton::Left) && g.playing && !g.paused {
                g.set_x(((mouse_position().0 - Self::stage().x) / Self::scale()) as f64);
            } else if is_mouse_button_released(MouseButton::Left) {
                self.touch = false;
                g.touch_ended();
            }
        }
    }
    pub fn draw(&self, a: &Assets, g: &PresentDrop) {
        let s = Self::scale();
        let v = vec2(
            screen_width() / 2. - 900. * s,
            screen_height() / 2. - 730. + 724.5 * (1. - s),
        );
        let st = Self::stage();
        let rect = |o: Vec2, x: f32, y: f32, w: f32, h: f32, color: u32| {
            draw_rectangle(
                o.x + x * s,
                o.y + y * s,
                w * s,
                h * s,
                Color::from_hex(color),
            )
        };
        let sprite = |o: Vec2,
                      name: &str,
                      x: f32,
                      y: f32,
                      w: f32,
                      h: f32,
                      sx: f32,
                      sy: f32,
                      rotation: f32,
                      origin: Vec2| {
            draw_texture_ex(
                a.scene("presentdrop", name),
                o.x + x * s,
                o.y + y * s,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h) * s),
                    source: Some(Rect::new(sx, sy, w, h)),
                    rotation,
                    pivot: Some(o + (vec2(x, y) + origin) * s),
                    ..Default::default()
                },
            );
        };
        let gradient = |y: f32, h: f32, top: u32, bottom: u32| {
            let vertices = vec![
                Vertex::new(0., y, 0., 0., 0., Color::from_hex(top)),
                Vertex::new(screen_width(), y, 0., 0., 0., Color::from_hex(top)),
                Vertex::new(screen_width(), y + h, 0., 0., 0., Color::from_hex(bottom)),
                Vertex::new(0., y + h, 0., 0., 0., Color::from_hex(bottom)),
            ];
            draw_mesh(&Mesh {
                vertices,
                indices: vec![0, 1, 2, 0, 2, 3],
                texture: None,
            });
        };
        clear_background(Color::from_hex(0xa0d7ef));
        gradient(v.y - 1425. * s, 2440. * s, 0x8bbedb, 0xa0d7ef);
        gradient(v.y + 1015. * s, 2000. * s, 0xededed, 0xffffff);
        // Background is an unpositioned empty div; its absolute children use .view.
        let mountain = a.scene("presentdrop", "mountains.png");
        let period = mountain.width();
        let mut x = -1800.;
        while x < 3600. {
            sprite(
                v,
                "mountains.png",
                x,
                819.,
                period,
                196.,
                0.,
                0.,
                0.,
                Vec2::ZERO,
            );
            x += period;
        }
        sprite(
            v,
            "environment.png",
            65.,
            900.,
            210.,
            150.,
            300.,
            150.,
            0.,
            Vec2::ZERO,
        );
        sprite(
            v,
            "environment.png",
            1515.,
            900.,
            210.,
            150.,
            300.,
            0.,
            0.,
            Vec2::ZERO,
        );
        sprite(
            v,
            "environment.png",
            1596.,
            310.,
            114.,
            114.,
            150.,
            0.,
            0.,
            Vec2::ZERO,
        );
        if screen_width() >= 1025. {
            for (x, y, k) in [
                (1574., 500., 1.),
                (1354., 150., 1.),
                (954., 100., 0.5),
                (100., 140., 0.5),
                (500., 200., 0.7),
                (100., 600., 0.7),
                (30., 400., 0.85),
            ] {
                draw_texture_ex(
                    a.scene("presentdrop", "environment.png"),
                    v.x + (x + (1. - k) * 73.) * s,
                    v.y + (y + (1. - k) * 44.) * s,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(146., 88.) * k * s),
                        source: Some(Rect::new(0., 0., 146., 88.)),
                        ..Default::default()
                    },
                );
            }
        }
        let house = v + vec2(250., 360.) * s;
        rect(house, 0., 0., 1291., 700., 0xcccccc);
        rect(house, -6., -20., 1304., 20., 0x808284);
        rect(house, 0., 0., 1291., 12., 0xb4b6b9);
        for y in [495., 370., 250., 130.] {
            for x in [63., 1210.] {
                rect(house, x, y, 18., 24., 0x009ee1);
                rect(house, x - 3., y + 24., 24., 6., 0xa7a9ab);
            }
        }
        rect(house, 46., 600., 55., 100., 0x6d6e70);
        rect(house, 84., 642., 5., 17., 0xa7a9ab);
        rect(house, 61., 615., 25., 25., 0xffffff);
        rect(house, 1181., 598., 40., 102., 0x6d6e70);
        rect(house, 1225., 598., 40., 102., 0x6d6e70);
        sprite(
            house,
            "environment.png",
            1167.,
            645.,
            9.,
            17.,
            300.,
            300.,
            0.,
            Vec2::ZERO,
        );
        sprite(
            house,
            "environment.png",
            130.,
            618.,
            55.,
            92.,
            75.,
            150.,
            0.,
            Vec2::ZERO,
        );
        sprite(
            house,
            "environment.png",
            1121.,
            618.,
            55.,
            92.,
            0.,
            150.,
            0.,
            Vec2::ZERO,
        );
        // The source stage clips all inner content, including scrolling backgrounds.
        unsafe {
            get_internal_gl().quad_gl.scissor(Some((
                st.x.round() as i32,
                st.y.round() as i32,
                (889. * s).round() as i32,
                (660. * s).round() as i32,
            )));
        }
        rect(st, 0., 0., 889., 660., 0x002c65);
        rect(st, 0., 0., 889., 10., 0x818181);
        for x in [90., 290., 490., 690.] {
            sprite(
                st,
                "environment.png",
                x,
                5.,
                104.,
                34.,
                0.,
                300.,
                0.,
                Vec2::ZERO,
            );
        }
        rect(st, 0., 111., 889., 2., 0xffffff);
        // Rotated rope-end blocks sit behind the elves and sled.
        for (x, angle) in [(-30., -35f32), (819., 35.)] {
            let pivot = st + vec2(x + 50., 140.) * s;
            let m = Mat4::from_translation(pivot.extend(0.))
                * Mat4::from_rotation_z(angle.to_radians())
                * Mat4::from_translation(-pivot.extend(0.));
            unsafe {
                get_internal_gl().quad_gl.push_model_matrix(m);
            }
            rect(st, x, 90., 100., 100., 0x002c65);
            rect(st, x, 140., 100., 2., 0xffffff);
            unsafe {
                get_internal_gl().quad_gl.pop_model_matrix();
            }
        }
        let back = g.pull_back == Some(true);
        sprite(
            st,
            "environment.png",
            if back { 33. } else { 22. },
            62.,
            66.,
            105.,
            if back { 216. } else { 150. },
            300.,
            0.,
            Vec2::ZERO,
        );
        let p = st + vec2(g.x as f32 - 150., 52.) * s;
        rect(p, 30., 58., 220., 4., 0x002c65);
        let age = (g.time - g.drop_at) as f32;
        let frame = if age < 0.5 {
            [0., 62., 124., 186.][((age / 0.125).floor() as usize).min(3)]
        } else {
            186.
        };
        sprite(
            p,
            "player.png",
            107.,
            -17.,
            62.,
            103.,
            frame,
            0.,
            0.,
            Vec2::ZERO,
        );
        sprite(
            p,
            "player.png",
            0.,
            0.,
            288.,
            127.,
            0.,
            110.,
            0.,
            Vec2::ZERO,
        );
        sprite(
            st,
            "environment.png",
            if back { 801. } else { 790. },
            62.,
            66.,
            105.,
            if back { 216. } else { 150. },
            150.,
            0.,
            Vec2::ZERO,
        );
        // Background at right:-bgRight; image repeats every 4450 source pixels.
        let bg = a.scene("presentdrop", "level-bg.png");
        let left = 889. - 4450. + g.bg_right as f32;
        let mut x = left;
        while x < 889. {
            sprite(
                st,
                "level-bg.png",
                x,
                165.,
                bg.width(),
                490.,
                0.,
                0.,
                0.,
                Vec2::ZERO,
            );
            x += bg.width();
        }
        let rails = a.scene("presentdrop", "player-rails.png");
        let mut x = 0.;
        while x < 889. {
            sprite(
                st,
                "player-rails.png",
                x,
                193.,
                rails.width(),
                25.,
                0.,
                30.,
                0.,
                Vec2::ZERO,
            );
            x += rails.width();
        }
        for (x, theme) in [
            (889. - 932. + g.fg_right as f32, g.fg_last),
            (889. - 1864. + g.fg_right as f32, g.fg_next),
        ] {
            let sx = [0., 2796., 1864., 932.][theme as usize];
            sprite(
                st,
                "level-fg.png",
                x,
                189.,
                932.,
                361.,
                sx,
                0.,
                0.,
                Vec2::ZERO,
            );
        }
        let mut x = 0.;
        while x < 889. {
            sprite(
                st,
                "player-rails.png",
                x,
                164.,
                rails.width(),
                26.,
                0.,
                0.,
                0.,
                Vec2::ZERO,
            );
            x += rails.width();
        }
        for e in &g.entities {
            if let Entity::Present(i) = e {
                let p = &g.presents[*i];
                rect(st, p.x as f32, p.y as f32, 20., 20., 0xdb4536);
                rect(st, p.x as f32, p.y as f32 + 9., 20., 2., 0xfedf00);
                rect(st, p.x as f32 + 9., p.y as f32, 2., 20., 0xfedf00);
            }
        }
        for e in &g.entities {
            if let Entity::Chimney(i) = e {
                let c = &g.chimneys[*i];
                let height = if c.large { 150. } else { 123. };
                let o = st + vec2(c.x as f32, 660. - 25. - height) * s;
                let flag = c.flag.value(g.wall_time)[0] as f32;
                rect(o, 66., 15. + flag, 6., 58., 0xe29e26);
                rect(o, 44., 25. + flag, 50., 34., 0xcf181f);
                let text = c.flag_score.to_string();
                let size = (20. * s).max(1.) as u16;
                let w = measure_text(&text, Some(&self.font), size, 1.).width;
                draw_text_ex(
                    &text,
                    o.x + 69. * s - w / 2.,
                    o.y + (49. + flag) * s,
                    TextParams {
                        font: Some(&self.font),
                        font_size: size,
                        color: WHITE,
                        ..Default::default()
                    },
                );
                let phase = (g.time as f32).fract();
                let wave = if phase < 0.5 {
                    phase * 2.
                } else {
                    (1. - phase) * 2.
                };
                sprite(
                    o,
                    "chimneys.png",
                    84.,
                    if c.large { 83. } else { 53. },
                    75.,
                    30.,
                    100.,
                    150.,
                    (27. * wave).to_radians(),
                    vec2(2., 19.),
                );
                sprite(
                    o,
                    "chimneys.png",
                    135.,
                    if c.large { 80. } else { 50. },
                    27.,
                    16.,
                    if c.large { 200. } else { 0. },
                    150.,
                    (-60. * wave).to_radians(),
                    vec2(25., 15.),
                );
                sprite(
                    o,
                    "chimneys.png",
                    0.,
                    0.,
                    180.,
                    height,
                    if c.large { 200. } else { 0. },
                    0.,
                    0.,
                    Vec2::ZERO,
                );
                let angle = (180. * (1. - (g.time as f32 / 0.3).fract())).to_radians();
                for x in [20., 130.] {
                    sprite(
                        o,
                        "chimneys.png",
                        x,
                        height - 9.,
                        30.,
                        30.,
                        300.,
                        150.,
                        angle,
                        vec2(15., 15.),
                    );
                }
            }
        }
        rect(st, 0., 650., 889., 10., 0x818181);
        draw_rectangle(st.x, st.y, 890. * s, 41. * s, Color::new(0., 0., 0., 0.1));
        for x in [0., 873.] {
            draw_rectangle(
                st.x + x * s,
                st.y + 41. * s,
                17. * s,
                619. * s,
                Color::new(0., 0., 0., 0.1),
            );
        }
        unsafe {
            get_internal_gl().quad_gl.scissor(None);
        }
    }
}
