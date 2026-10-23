use crate::assets::Assets;
use macroquad::prelude::*;
use santa_tracker::runner::{Entity, Kind, Runner, State};

pub struct View {
    scale: f32,
    x: f32,
    y: f32,
}
impl View {
    pub fn new() -> Self {
        let scale = (screen_width() / 1800.).min(screen_height() / 1000.);
        Self {
            scale,
            x: (screen_width() - 1800. * scale) / 2.,
            y: (screen_height() - 1000. * scale) / 2.,
        }
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        draw_rectangle(
            self.x + x * self.scale,
            self.y + y * self.scale,
            w * self.scale,
            h * self.scale,
            color,
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
                dest_size: Some(vec2(w * self.scale, h * self.scale)),
                source,
                rotation,
                ..Default::default()
            },
        );
    }
    fn image(&self, assets: &Assets, name: &str, x: f32, y: f32, w: f32, h: f32) {
        self.texture(assets.runner(name), x, y, w, h, None, 0., WHITE);
    }
    fn tile(
        &self,
        assets: &Assets,
        name: &str,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tile: f32,
        offset: f32,
    ) {
        let mut px = x - offset.rem_euclid(tile);
        while px < x + w {
            self.image(assets, name, px, y, tile, h);
            px += tile;
        }
    }
    fn text(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text(
            s,
            self.x + x * self.scale,
            self.y + y * self.scale,
            size * self.scale,
            color,
        );
    }
    pub fn runner(&self, a: &Assets, g: &Runner) {
        let sx = 195.;
        let ground = 810.;
        let time = g.time as f32;
        let speed = (g.speed / 350.) as f32;
        self.rect(0., 0., 1800., 1000., Color::from_rgba(135, 131, 132, 255));
        self.tile(
            a,
            "bg-windows-repeating.png",
            0.,
            0.,
            1800.,
            1080.,
            a.runner("bg-windows-repeating.png").width(),
            0.,
        );
        self.image(a, "bg-wall-shadow.svg", 0., 0., 1800., 810.);
        self.rect(0., 810., 1800., 190., Color::from_rgba(135, 131, 132, 255));
        self.rect(
            sx - 352.5,
            140.,
            2115.,
            670.,
            Color::from_rgba(26, 35, 126, 255),
        );
        self.tile(
            a,
            "bg-tile-layer1.png",
            sx - 352.5,
            140.,
            2115.,
            671.,
            1490.,
            time * 15.,
        );
        self.tile(
            a,
            "bg-tile-layer2.png",
            sx - 352.5,
            ground - 242.,
            2115.,
            242.,
            2980.,
            time * 30.,
        );
        for e in g
            .entities
            .iter()
            .filter(|e| matches!(e.kind, Kind::Platform | Kind::Obstacle | Kind::Finish))
        {
            self.entity(a, g, e, sx, ground);
        }
        let (image, frame) = match g.player.state {
            State::Running => {
                let phase = (time / (0.75 / speed)).fract();
                (
                    "hero.svg",
                    if phase >= 0.88 {
                        0
                    } else {
                        (phase / 0.11).ceil() as u32
                    },
                )
            }
            State::Sliding => ("hero-slide.svg", ((time / 0.5).fract() * 4.).floor() as u32),
            State::Jumping | State::JumpStart => ("hero-jump-01.svg", 0),
            State::Collision => ("hero-crash.svg", 0),
        };
        let alpha = if g.player.state == State::Collision {
            0.5 + 0.5 * (time / 0.2).fract()
        } else {
            1.
        };
        self.texture(
            a.runner(image),
            sx + g.player.x as f32,
            ground - 170. + g.player.y as f32,
            275.,
            170.,
            Some(Rect::new(frame as f32 * 275., 0., 275., 170.)),
            (g.player.angle as f32).to_radians(),
            Color::new(1., 1., 1., alpha),
        );
        for e in g
            .entities
            .iter()
            .filter(|e| matches!(e.kind, Kind::Present | Kind::Time | Kind::Magnet))
        {
            self.entity(a, g, e, sx, ground);
        }
        if g.cloud {
            self.image(
                a,
                "hit-cloud.svg",
                sx + g.player.x as f32 + 95.,
                ground + g.player.y as f32 - 150.,
                120.,
                120.,
            );
        }
        if g.magnet {
            self.image(
                a,
                "orb-magnet-front.svg",
                sx + g.player.x as f32 - 50.,
                ground + g.player.y as f32 - 120.,
                60.,
                60.,
            );
        }
        self.tile(
            a,
            "bg-tile-track.svg",
            sx - 352.5,
            ground - 7.,
            2115.,
            10.,
            25.,
            time / 0.072 * 25. * speed,
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
        for (x, y, period, dir) in [
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
                time / period * speed * std::f32::consts::TAU * dir,
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
        self.image(a, "spotlight.svg", sx - 70., 0., 553., 849.);
        self.tile(
            a,
            "ceiling-repeating.png",
            0.,
            0.,
            1800.,
            30.,
            a.runner("ceiling-repeating.png").width(),
            0.,
        );
        self.image(a, "treadmill-base.svg", sx - 100., ground, 522., 66.);
        for i in 0..6 {
            self.texture(
                a.runner("treadmill-gears.svg"),
                sx - 16. + i as f32 * 62.,
                ground + 4.,
                33.,
                33.,
                None,
                -(time + i as f32 * 0.3) / 4.712 * 15.7 * speed * std::f32::consts::TAU,
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
        self.image(
            a,
            "foreground-operator.svg",
            sx - 74.,
            ground - 102.,
            85.,
            195.,
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
        self.text(&format!("{}", g.level), sx - 91., ground - 65., 40., WHITE);
        if g.paused || !g.playing {
            self.rect(0., 55., 1800., 945., Color::new(0., 0., 0., 0.55));
            self.text(
                if g.paused { "Paused" } else { "Game Over" },
                680.,
                450.,
                80.,
                WHITE,
            );
            self.text("Press R to play again", 670., 530., 36., WHITE);
        }
    }
    fn entity(&self, a: &Assets, g: &Runner, e: &Entity, sx: f32, ground: f32) {
        let x = sx + (e.x - g.distance) as f32;
        if x < -500. || x > 2300. {
            return;
        }
        let mut color = WHITE;
        let mut grow = 1.;
        if e.collected {
            let t = ((g.time - e.collected_at) / 0.5).clamp(0., 1.) as f32;
            color.a = 1. - t;
            grow = 1. + t;
            if t >= 1. {
                return;
            }
        }
        let (name, w, h, frame) = match e.kind {
            Kind::Finish => {
                let phase = (g.time / 3.).fract();
                (
                    "finish.svg".into(),
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
                let idx = e.style.rsplit("--").next().unwrap();
                (
                    format!("plat-{}-{idx}.svg", if tall { "tall" } else { "short" }),
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
                (file, w, h, 0)
            }
            Kind::Present => {
                let file = match e.style.as_str() {
                    "present--1" => "present1.svg",
                    "present--2" => "present2.svg",
                    "carrot" => "treat-carrot.svg",
                    _ => "treat-candy.svg",
                };
                let phase = (g.time / 5.).fract();
                let frame = if e.style.starts_with("present") {
                    if phase < 0.94 {
                        0
                    } else if phase < 0.96 {
                        1
                    } else if phase < 0.98 {
                        2
                    } else {
                        3
                    }
                } else {
                    0
                };
                (file.into(), e.width as f32, e.height as f32, frame)
            }
            Kind::Time => ("orb-time-front.svg".into(), 60., 60., 0),
            Kind::Magnet => ("orb-magnet-front.svg".into(), 60., 60., 0),
        };
        let y = if e.kind == Kind::Present || e.kind == Kind::Time || e.kind == Kind::Magnet {
            ground + e.y as f32
        } else {
            ground + e.y as f32 - h
        };
        if e.kind == Kind::Time || e.kind == Kind::Magnet {
            self.texture(
                a.runner(if e.kind == Kind::Time {
                    "orb-time-back.svg"
                } else {
                    "orb-magnet-back.svg"
                }),
                x,
                y,
                w,
                h,
                None,
                g.time as f32 / 2. * std::f32::consts::TAU,
                color,
            );
        }
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
        if e.kind == Kind::Platform {
            self.image(a, "ceiling-roller-long.svg", x + 21., 31., 350., 15.);
            for dx in [61.5, 321.5] {
                self.rect(
                    x + dx,
                    46.,
                    5.,
                    (y - 46.).max(0.),
                    Color::new(0.25, 0.42, 0.51, 0.7),
                );
            }
        }
    }
}
