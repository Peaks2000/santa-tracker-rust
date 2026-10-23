//! Native Runner HTML/SCSS layout, sprite animation and scenery.
use crate::assets::Assets;
use macroquad::prelude::*;
use santa_tracker::{
    runner::{Entity, Kind, Layout, Runner, State},
    ui::Ease,
};
pub struct View {
    scale: f32,
    x: f32,
    y: f32,
    title: Font,
    score_font: Font,
    jump_down: bool,
    touch_start: Option<f32>,
    speed_warning: Texture2D,
}
impl View {
    pub fn new() -> Self {
        let tree = resvg::usvg::Tree::from_data(
            include_bytes!("../data/runner-speed-warning.svg"),
            &Default::default(),
        )
        .expect("Runner warning vector");
        let mut pixels = resvg::tiny_skia::Pixmap::new(200, 200).unwrap();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(4., 4.),
            &mut pixels.as_mut(),
        );
        let mut rgba = pixels.data().to_vec();
        for p in rgba.chunks_exact_mut(4) {
            if p[3] > 0 {
                for c in 0..3 {
                    p[c] = (p[c] as u32 * 255 / p[3] as u32).min(255) as u8;
                }
            }
        }
        santa_tracker::image_alpha::bleed_edges(&mut rgba, 200, 200);
        let speed_warning = Texture2D::from_rgba8(200, 200, &rgba);
        speed_warning.set_filter(FilterMode::Linear);
        Self {
            speed_warning,
            scale: 1.,
            x: 0.,
            y: 0.,
            jump_down: false,
            touch_start: None,
            title: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Lobster-Regular.ttf"))
                .expect("Runner Lobster"),
            score_font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/OpenSans-ExtraBold.ttf"
            ))
            .expect("Runner score font"),
        }
    }
    pub fn reset_input(&mut self) {
        self.jump_down = false;
        self.touch_start = None;
    }
    pub fn input(&mut self, g: &mut Runner) {
        if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Space)) && !self.jump_down {
            g.jump();
            self.jump_down = true;
        }
        if is_key_released(KeyCode::Up) || is_key_released(KeyCode::Space) {
            self.jump_down = false;
        }
        if is_key_pressed(KeyCode::Down) {
            g.down();
        }
        if is_key_released(KeyCode::Down) {
            g.end_slide();
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && !crate::chrome::over_controls(Vec2::from(mouse_position()), 3)
        {
            self.touch_start = Some(mouse_position().1);
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(start) = self.touch_start.take() {
                let diff = start - mouse_position().1;
                if diff >= 0. {
                    g.jump();
                } else if diff < -25. {
                    g.down_for(2.);
                }
            }
        }
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        draw_rectangle(
            self.x + x * self.scale,
            self.y + y * self.scale,
            w * self.scale,
            h * self.scale,
            c,
        );
    }
    fn texture(
        &self,
        t: &Texture2D,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        source: Option<Rect>,
        rotation: f32,
        color: Color,
    ) {
        draw_texture_ex(
            t,
            self.x + x * self.scale,
            self.y + y * self.scale,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(w, h) * self.scale),
                source,
                rotation,
                ..Default::default()
            },
        );
    }
    fn image(&self, a: &Assets, n: &str, x: f32, y: f32, w: f32, h: f32) {
        self.texture(a.runner(n), x, y, w, h, None, 0., WHITE);
    }
    fn tile(&self, a: &Assets, n: &str, x: f32, y: f32, w: f32, h: f32, tile: f32, offset: f32) {
        let mut px = x - offset.rem_euclid(tile);
        while px < x + w {
            self.image(a, n, px, y, tile, h);
            px += tile;
        }
    }
    fn clip(&self, r: Option<Rect>) {
        unsafe {
            get_internal_gl().quad_gl.scissor(r.map(|r| {
                let x = self.x + r.x * self.scale;
                let y = self.y + r.y * self.scale;
                let l = x.max(0.).floor();
                let t = y.max(0.).floor();
                let right = (x + r.w * self.scale).min(screen_width()).ceil();
                let bottom = (y + r.h * self.scale).min(screen_height()).ceil();
                (
                    l as i32,
                    t as i32,
                    (right - l).max(0.) as i32,
                    (bottom - t).max(0.) as i32,
                )
            }));
        }
    }
    fn text(&self, text: &str, x: f32, y: f32, size: f32, alpha: f32, title: bool) {
        draw_text_ex(
            text,
            self.x + x * self.scale,
            self.y + y * self.scale,
            TextParams {
                font: Some(if title { &self.title } else { &self.score_font }),
                font_size: (size * self.scale).max(1.) as u16,
                color: Color::new(1., 1., 1., alpha),
                ..Default::default()
            },
        );
    }
    pub fn runner(&mut self, a: &Assets, g: &Runner) {
        let layout = Layout::new(screen_width() as f64, screen_height() as f64);
        self.scale = layout.scale as f32;
        self.x = screen_width() / 2. - 900. * self.scale;
        self.y = layout.top as f32;
        let s = self.scale;
        let sx = 195.;
        let origin = sx - 352.5;
        let ground = 810.;
        let time = g.animation_time as f32;
        let speed = [1., 1.3, 1.6, 1.9][g.level];
        clear_background(Color::from_hex(0x878384));
        // Background .wall isn't transformed; only its CSS height and the window
        // tile size change with media queries. Its pattern is centered on viewport.
        let wall_height = layout.wall_height as f32;
        unsafe {
            get_internal_gl().quad_gl.scissor(Some((
                0,
                self.y as i32,
                screen_width() as i32,
                wall_height.ceil() as i32,
            )));
        }
        let tw = 368. * s;
        let mut px = screen_width() / 2. - tw / 2.;
        while px > 0. {
            px -= tw;
        }
        while px < screen_width() {
            draw_texture_ex(
                a.runner("bg-windows-repeating.png"),
                px,
                self.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(tw, 1080. * s)),
                    ..Default::default()
                },
            );
            px += tw;
        }
        let shadow = a.runner("bg-wall-shadow.svg");
        let fit = (screen_width() / shadow.width()).max(wall_height / shadow.height());
        let size = vec2(shadow.width(), shadow.height()) * fit;
        draw_texture_ex(
            shadow,
            (screen_width() - size.x) / 2.,
            self.y + (wall_height - size.y) / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
        self.clip(None);
        if self.y > 0. {
            draw_rectangle(0., 0., screen_width(), self.y, Color::from_hex(0x845a62));
            let mut x = (screen_width() - 598.) / 2.;
            while x > 0. {
                x -= 598.;
            }
            while x < screen_width() {
                draw_texture_ex(
                    a.runner("building-brick.svg"),
                    x,
                    self.y - 152.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(598., 152.)),
                        ..Default::default()
                    },
                );
                x += 598.;
            }
            draw_rectangle(
                0.,
                self.y - 4.,
                screen_width(),
                4.,
                Color::from_hex(0x5f4c54),
            );
        }
        // Stage machinery comes before the clipped layers in the original DOM.
        for (x, y, period, direction) in [
            (sx - 88., 488., 3., -1.),
            (sx + 1410. - 38., 131., 3., 1.),
            (sx + 1410. - 2., 477., 3., 1.),
        ] {
            self.texture(
                a.runner("paper-stage-gear-big.svg"),
                x,
                y,
                86.,
                86.,
                None,
                time / period * speed * std::f32::consts::TAU * direction,
                WHITE,
            );
        }
        self.texture(
            a.runner("paper-stage-gear-small.svg"),
            sx + 1375.,
            83.,
            52.,
            52.,
            None,
            -time / 2. * speed * std::f32::consts::TAU,
            WHITE,
        );
        self.image(a, "paper-stage-left.svg", sx - 102., 85., 135., 725.);
        self.image(
            a,
            "paper-stage-right.svg",
            sx + 1410. - 33.,
            85.,
            135.,
            725.,
        );
        for (x, y, w, h) in [
            (origin - 4., -4., 2123., 4.),
            (origin - 4., 810., 2123., 4.),
            (origin - 4., 0., 4., 810.),
            (origin + 2115., 0., 4., 810.),
        ] {
            self.rect(x, y, w, h, Color::new(0., 0., 0., 0.25));
        }
        // The canvas only paints the central mountain strip; trees extend wider.
        self.clip(Some(Rect::new(sx, 140., 1410., 670.)));
        let mx = g.mountain_x as f32;
        self.image(
            a,
            "bg-tile-layer1.png",
            origin + mx,
            140.,
            1490.,
            671. * 810. / 670.,
        );
        if mx < 352.5 + 1410. - 1490. {
            self.image(
                a,
                "bg-tile-layer1.png",
                origin + mx + 1490.,
                140.,
                1490.,
                671. * 810. / 670.,
            );
        }
        self.clip(Some(Rect::new(origin, 0., 2115., 810.)));
        self.image(
            a,
            "bg-tile-layer2.png",
            origin + g.trees_x as f32,
            140. + 428. * 810. / 670.,
            2980.,
            242. * 810. / 670.,
        );
        if g.trees_x < 2115. - 2980. {
            self.image(
                a,
                "bg-tile-layer2.png",
                origin + g.trees_x as f32 + 2980.,
                140. + 428. * 810. / 670.,
                2980.,
                242. * 810. / 670.,
            );
        }
        // Small gears underneath the moving scenery have their own clipping box.
        let fg = origin + 2115. - 500. - 150.;
        self.clip(Some(Rect::new(fg, 710., 150., 100.)));
        self.image(a, "foreground-gear-base.svg", fg, 810. - 22.7, 115., 22.7);
        self.texture(
            a.runner("foreground-gear-big.svg"),
            fg + 45.,
            744.,
            86.,
            86.,
            None,
            time / 4. * speed * std::f32::consts::TAU,
            WHITE,
        );
        self.texture(
            a.runner("foreground-gear-small.svg"),
            fg + 17.,
            748.,
            52.,
            52.,
            None,
            -time / 3. * speed * std::f32::consts::TAU,
            WHITE,
        );
        self.clip(Some(Rect::new(origin, 0., 2115., 810.)));
        // The transformed entities layer is one stacking context below the player.
        for kind in [
            Kind::Platform,
            Kind::Obstacle,
            Kind::Time,
            Kind::Magnet,
            Kind::Finish,
            Kind::Present,
        ] {
            for e in g.entities.iter().filter(|e| e.kind == kind) {
                self.entity(a, g, e, origin, ground);
            }
        }
        self.tile(
            a,
            "bg-tile-track.svg",
            origin,
            803.,
            2115.,
            10.,
            25.,
            if screen_width() >= 800. || screen_height() >= 690. {
                time / 0.072 * 25. * speed - 1045.
            } else {
                -1045.
            },
        );
        let (hx, hy, hw, hh) = g.player_hitbox();
        if g.cloud {
            let t = Ease::Default.at((g.wall_time - g.cloud_since) / 0.5) as f32;
            let x = origin + hx as f32 + hw as f32 / 2. - 60.;
            let y = ground + hy as f32 - hh as f32 / 2. - 60.;
            self.texture(
                a.runner("hit-cloud.svg"),
                x - 60. * t,
                y - 60. * t,
                120. * (1. + t),
                120. * (1. + t),
                None,
                0.,
                Color::new(1., 1., 1., 1. - t),
            );
        }
        let magnet = g.magnet_visual.value(g.wall_time)[0] as f32;
        if magnet > 0. {
            let x = origin + hx as f32 - 75.;
            let y = ground + hy as f32 - hh as f32 / 2.;
            self.texture(
                a.runner("orb-magnet-back.svg"),
                x,
                y,
                60.,
                60.,
                None,
                time / 2. * std::f32::consts::TAU,
                Color::new(1., 1., 1., magnet),
            );
            self.texture(
                a.runner("orb-magnet-front.svg"),
                x,
                y,
                60.,
                60.,
                None,
                0.,
                Color::new(1., 1., 1., magnet),
            );
        }
        let state_time = (g.animation_time - g.player.state_since) as f32;
        let (image, frame) = match g.player.state {
            State::Running => {
                let phase = (state_time / (0.75 / speed)).fract();
                (
                    "hero.svg",
                    if phase >= 0.88 {
                        0
                    } else {
                        ((phase / 0.11).floor() as u32 + 1).min(8)
                    },
                )
            }
            State::Sliding => {
                let p = (state_time / 0.5).fract();
                (
                    "hero-slide.svg",
                    if p >= 0.75 {
                        0
                    } else {
                        (p * 4.).floor() as u32 + 1
                    },
                )
            }
            State::Jumping | State::JumpStart => ("hero-jump-01.svg", 0),
            State::Collision => ("hero-crash.svg", 0),
        };
        let alpha = if g.player.state == State::Collision {
            0.5 + 0.5 * (state_time / 0.2).fract()
        } else {
            1.
        };
        self.texture(
            a.runner(image),
            origin + g.player.x as f32,
            ground - 170. + g.player.y as f32,
            275.,
            170.,
            Some(Rect::new(frame as f32 * 275., 0., 275., 170.)),
            (g.player.angle as f32).to_radians(),
            Color::new(1., 1., 1., alpha),
        );
        // Opaque edge strips and their inset shadows are above all layer children.
        self.rect(origin, 0., 40., 810., Color::from_hex(0x878384));
        self.rect(
            origin + 2115. - 40.,
            0.,
            40.,
            810.,
            Color::from_hex(0x878384),
        );
        self.rect(origin + 40., 0., 4., 810., Color::new(0., 0., 0., 0.25));
        self.rect(
            origin + 2115. - 44.,
            0.,
            4.,
            810.,
            Color::new(0., 0., 0., 0.25),
        );
        self.clip(None);
        self.image(a, "spotlight.svg", sx - 70., 0., 553., 849.);
        // CSS background-size:contain maps the 160x120 image into a 40x30 tile.
        let wide = (screen_width() / s) + 2820.;
        let left = 900. - wide / 2.;
        self.clip(Some(Rect::new(left, 0., wide, 30.)));
        self.tile(
            a,
            "ceiling-repeating.png",
            left,
            0.,
            wide,
            30.,
            40.,
            -(wide - 40.) / 2.,
        );
        self.clip(None);
        self.image(a, "treadmill-base.svg", sx - 100., ground, 522., 66.);
        for (y, dir) in [(811., 1.), (848., -1.)] {
            self.clip(Some(Rect::new(sx - 61., y, 431., 3.)));
            self.tile(
                a,
                "treadmill-belt.svg",
                sx - 61.,
                y,
                431.,
                3.,
                22.,
                time * 15.7 * 22. * dir,
            );
            self.clip(None);
        }
        for (i, delay) in [0.5, 0.9, 0.3, 0.7, 0.2, 0.].into_iter().enumerate() {
            self.texture(
                a.runner("treadmill-gears.svg"),
                sx - 16. + i as f32 * 62.,
                ground + 4.,
                33.,
                33.,
                None,
                -(time + delay) / 4.712 * 15.7 * std::f32::consts::TAU,
                WHITE,
            );
        }
        self.image(a, "treadmill-shadow.svg", sx - 100., ground, 522., 66.);
        self.image(a, "treadmill-corners.svg", sx - 100., ground, 522., 66.);
        self.image(
            a,
            "foreground-machines.svg",
            sx - 230.,
            ground - 110.,
            361.,
            207.,
        );
        for i in 0..2 {
            self.texture(
                a.runner("foreground-machines-cog.svg"),
                sx - 210. + i as f32 * 47.,
                ground - 98.,
                39.,
                39.,
                None,
                (time + i as f32 * 0.2) / 2. * std::f32::consts::TAU,
                WHITE,
            );
        }
        self.speedometer(g, sx - 230. + 154., 705.);
        self.image(
            a,
            "foreground-operator.svg",
            sx - 230. + 156.,
            708.,
            85.,
            195.,
        );
        self.image(
            a,
            "foreground-cooling-off.svg",
            sx + 1410. - 206.,
            670.,
            321.,
            241.,
        );
    }
    fn speedometer(&self, g: &Runner, x: f32, y: f32) {
        self.clip(Some(Rect::new(x, y, 91., 58.)));
        let time = g.animation_time;
        let changing = g.transition > 0. && g.transition < 1.8;
        if !changing {
            let phase = (time / if g.level < 2 { 3. } else { 2.5 }).fract();
            let second = if g.level == 0 || g.level == 3 {
                let states = [
                    (0., '3'),
                    (0.25, '4'),
                    (0.27, '3'),
                    (0.28, '4'),
                    (0.30, '3'),
                    (0.74, '4'),
                    (0.93, '3'),
                    (0.95, '4'),
                    (0.97, '3'),
                    (0.98, '4'),
                ];
                if g.level == 3 {
                    states
                        .iter()
                        .find(|(t, _)| *t > phase)
                        .unwrap_or(&states[0])
                        .1
                } else {
                    states.iter().rev().find(|(t, _)| phase >= *t).unwrap().1
                }
            } else {
                let states = [
                    (0., '7'),
                    (0.03, '8'),
                    (0.27, '7'),
                    (0.28, '8'),
                    (0.30, '7'),
                    (0.74, '8'),
                    (0.78, '7'),
                    (0.95, '8'),
                    (0.97, '7'),
                    (0.98, '8'),
                ];
                if g.level == 1 {
                    states
                        .iter()
                        .find(|(t, _)| *t > phase)
                        .unwrap_or(&states[0])
                        .1
                } else {
                    states.iter().rev().find(|(t, _)| phase >= *t).unwrap().1
                }
            };
            let first = ['2', '3', '5', '6'][g.level];
            self.text(&first.to_string(), x + 22., y + 42., 40., 0.6, true);
            self.text(&first.to_string(), x + 24., y + 42., 40., 0.8, true);
            self.text(&second.to_string(), x + 48., y + 42., 40., 0.6, true);
            self.text(&second.to_string(), x + 50., y + 42., 40., 0.8, true);
        }
        if g.transition > 0. {
            let age = 2. - g.transition;
            let colors = [0x22d0bd, 0x03d32b, 0x3cc800, 0x99d400, 0xe8c000, 0xeb6201];
            for (i, c) in colors.into_iter().enumerate() {
                let t = (age - i as f64 * 0.13) / 1.9;
                if t < 0. {
                    continue;
                }
                let v = cubic_stripe(t) as f32;
                self.rect(
                    x + i as f32 * 91. / 6.,
                    y + 81.2 - 250. * v,
                    91. / 6.,
                    150.,
                    Color::from_hex(c),
                );
            }
            if changing && ((age - 0.2) / 0.334).fract() < 0.8 {
                self.texture(
                    &self.speed_warning,
                    x + 20.5,
                    y + 4.,
                    50.,
                    50.,
                    None,
                    0.,
                    WHITE,
                );
                let size = (35. * self.scale).max(1.) as u16;
                let width = measure_text("!", Some(&self.title), size, 1.).width;
                for dx in [0., 0.3] {
                    draw_text_ex(
                        "!",
                        self.x + (x + 45.5 + dx) * self.scale - width / 2.,
                        self.y + (y + 46.) * self.scale,
                        TextParams {
                            font: Some(&self.title),
                            font_size: size,
                            color: Color::from_hex(0x0aa6a3),
                            ..Default::default()
                        },
                    );
                }
            }
        }
        self.clip(None);
    }
    fn entity(&self, a: &Assets, g: &Runner, e: &Entity, origin: f32, ground: f32) {
        let x = origin + (e.x - g.distance) as f32;
        if x < origin - 500. || x > origin + 2300. {
            return;
        }
        let age = (g.time - e.collected_at) as f32;
        let visual_age = (g.wall_time - e.collected_wall_at) as f32;
        let t = if e.collected {
            Ease::Default.at((visual_age / 0.5) as f64) as f32
        } else {
            0.
        };
        let color = Color::new(1., 1., 1., 1. - t);
        let grow = 1. + t;
        let (name, w, h, frame) = match e.kind {
            Kind::Finish => {
                let phase = ((g.animation_time - e.born) / 3.).fract();
                (
                    "finish.svg".to_owned(),
                    300.,
                    380.,
                    if phase < 0.8 || (0.85..0.9).contains(&phase) {
                        1
                    } else {
                        0
                    },
                )
            }
            Kind::Platform => {
                let tall = e.style.contains("tall");
                (
                    format!(
                        "plat-{}-{}.svg",
                        if tall { "tall" } else { "short" },
                        e.style.rsplit("--").next().unwrap()
                    ),
                    442.,
                    e.height as f32,
                    0,
                )
            }
            Kind::Obstacle => {
                let (file, w, h) = if let Some(n) = e.style.strip_prefix("house--") {
                    (
                        format!("hit-house-{:02}.svg", n.parse::<u32>().unwrap()),
                        450.,
                        270.,
                    )
                } else if let Some(n) = e.style.strip_prefix("tree-short--") {
                    (format!("hit-tree-short-{n}.svg"), 190., 268.)
                } else if let Some(n) = e.style.strip_prefix("tree-tall--") {
                    (
                        format!("hit-tree-tall-{n}.svg"),
                        if n == "1" { 280. } else { 236. },
                        466.,
                    )
                } else {
                    match e.style.as_str() {
                        "antenna" => ("hit-roof-antenna.svg".into(), 138., 112.),
                        "chimney--1" => ("hit-roof-chimney-1.svg".into(), 122., 112.),
                        "chimney--2" => ("hit-roof-chimney-2.svg".into(), 106., 112.),
                        "watertower" => ("hit-roof-water.svg".into(), 115., 112.),
                        _ => ("trashcan.svg".into(), 206., 128.),
                    }
                };
                (
                    file,
                    w,
                    h,
                    if e.style == "trashcan" && e.hit { 1 } else { 0 },
                )
            }
            Kind::Present => {
                let name = match e.style.as_str() {
                    "present--1" => "present1.svg",
                    "present--2" => "present2.svg",
                    "carrot" => "treat-carrot.svg",
                    _ => "treat-candy.svg",
                };
                let p = ((g.animation_time - e.born) / 5.).fract();
                let frame = if e.style.starts_with("present") {
                    if p < 0.94 {
                        0
                    } else if p < 0.96 {
                        1
                    } else if p < 0.98 {
                        2
                    } else {
                        3
                    }
                } else {
                    0
                };
                (name.into(), e.width as f32, e.height as f32, frame)
            }
            Kind::Time => ("orb-time-front.svg".into(), 60., 60., 0),
            Kind::Magnet => ("orb-magnet-front.svg".into(), 60., 60., 0),
        };
        let y = if matches!(e.kind, Kind::Present | Kind::Time | Kind::Magnet) {
            ground + e.y as f32
        } else {
            ground + e.y as f32 - h + if e.kind == Kind::Obstacle { 2. } else { 0. }
        };
        if e.kind == Kind::Platform || e.style.starts_with("tree-") {
            let platform = e.kind == Kind::Platform;
            let roller = if platform { 350. } else { 150. };
            let left = (w - roller - 50.) / 2.;
            let wire_left = (w - roller - 50. + 81.) / 2.;
            let top = y + h + 30. - 140. - 670. - 1.;
            let bottom = y + h - if platform { 40. } else { 140. };
            for dx in [wire_left, wire_left + if platform { 265. } else { 65. }] {
                self.rect(
                    x + dx,
                    top,
                    5.,
                    (bottom - top).max(0.),
                    Color::new(64. / 255., 108. / 255., 130. / 255., 0.7),
                );
            }
            self.image(
                a,
                if platform {
                    "ceiling-roller-long.svg"
                } else {
                    "ceiling-roller-short.svg"
                },
                x + left,
                top - 8.,
                roller,
                15.,
            );
        }
        if matches!(e.kind, Kind::Time | Kind::Magnet) {
            self.texture(
                a.runner(if e.kind == Kind::Time {
                    "orb-time-back.svg"
                } else {
                    "orb-magnet-back.svg"
                }),
                x - (grow - 1.) * w / 2.,
                y - (grow - 1.) * h / 2.,
                w * grow,
                h * grow,
                None,
                ((g.animation_time - e.born) / 2.) as f32 * std::f32::consts::TAU,
                color,
            );
        }
        if !e.collected || visual_age < 0.5 {
            self.texture(
                a.runner(&name),
                x - (grow - 1.) * w / 2.,
                y - (grow - 1.) * h / 2.,
                w * grow,
                h * grow,
                Some(Rect::new(frame as f32 * w, 0., w, h)),
                0.,
                color,
            );
        }
        if e.kind == Kind::Time && e.collected && age < 2. {
            let opacity = 1. - Ease::Default.at(((age - 0.5) / 1.5) as f64) as f32;
            self.text(
                "+00:10",
                x - 40.,
                ground + e.y as f32 + 40.,
                40.,
                opacity,
                false,
            );
        }
    }
}
fn cubic_stripe(t: f64) -> f64 {
    let x = t.clamp(0., 1.);
    let (mut lo, mut hi) = (0., 1.);
    for _ in 0..40 {
        let u = (lo + hi) / 2.;
        let px = 3. * (1. - u) * u * u + u * u * u;
        if px < x {
            lo = u;
        } else {
            hi = u;
        }
    }
    let u = (lo + hi) / 2.;
    3. * (1. - u).powi(2) * u * 0.77 + 3. * (1. - u) * u * u * 0.21 + u.powi(3)
}
