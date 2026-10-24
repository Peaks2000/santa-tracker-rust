//! Original Penguin Dash images, sprite frames, world layers and camera.
use crate::assets::Assets;
use macroquad::prelude::*;
use santa_tracker::{
    penguindash::{image_size, player_frame, PenguinDash},
    random::Random,
    ui::Ease,
};
use std::sync::mpsc::{self, Receiver, Sender};
struct Water {
    texture: Texture2D,
    request: Sender<usize>,
    ready: Receiver<(usize, Image)>,
    requested: usize,
    shown: usize,
}
impl Water {
    fn new(a: &Assets) -> Self {
        // Own the streaming texture: never overwrite the cached reset frame.
        let data = std::fs::read(a.root.join("scenes/penguindash/img/native-water.png"))
            .expect("Penguin water reset frame");
        let image = Image::from_file_with_format(&data, None).expect("Penguin water image");
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Linear);
        let (tx, rx) = mpsc::channel();
        let (out, ready) = mpsc::sync_channel(1);
        let root = a.root.join("scenes/penguindash/video");
        std::thread::spawn(move || {
            while let Ok(mut n) = rx.recv() {
                while let Ok(next) = rx.try_recv() {
                    n = next
                }
                let path = root.join(format!("{:03}.png", n + 1));
                let Ok(data) = std::fs::read(path) else { break };
                let Ok(im) = Image::from_file_with_format(&data, None) else {
                    break;
                };
                if out.send((n, im)).is_err() {
                    break;
                }
            }
        });
        Self {
            texture,
            request: tx,
            ready,
            requested: 0,
            shown: 0,
        }
    }
    fn update(&mut self, time: f64) {
        let frame = (time * 12.).floor() as usize % 194;
        if self.requested != frame {
            let _ = self.request.send(frame);
            self.requested = frame
        }
        while let Ok((n, im)) = self.ready.try_recv() {
            self.texture.update(&im);
            self.shown = n;
        }
    }
}
#[derive(Clone)]
struct Wave {
    x: f64,
    y: f64,
    angle: f64,
    kind: usize,
    since: f64,
    first: bool,
}
pub struct PenguinView {
    water: Option<Water>,
    waves: Vec<Wave>,
    random: Random,
    font: Font,
    initialized: bool,
}
impl PenguinView {
    pub fn new(seed: &str) -> Self {
        let mut random = Random::seeded(seed);
        let mut waves = vec![];
        for _ in 0..4 {
            let kind = (random.next() * 2.).ceil() as usize;
            waves.push(Wave {
                x: random.next() * 3000.,
                y: random.next() * 3000.,
                angle: 0.,
                kind,
                since: 0.,
                first: true,
            })
        }
        Self {
            water: None,
            waves,
            random,
            initialized: false,
            font: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Lobster-Regular.ttf"))
                .unwrap(),
        }
    }
    pub fn input() -> santa_tracker::penguindash::Input {
        santa_tracker::penguindash::Input {
            left: is_key_down(KeyCode::Left),
            right: is_key_down(KeyCode::Right),
            up: is_key_down(KeyCode::Up),
            down: is_key_down(KeyCode::Down),
        }
    }
    pub fn draw(&mut self, a: &Assets, g: &PenguinDash) {
        let w = screen_width() as f64;
        let h = screen_height() as f64;
        let scale = if w < 980. { (w + 490.) / 1470. } else { 1. };
        let vw = w / scale;
        let vh = h / scale;
        let camera = (
            (g.x - (vw / 2.).round())
                .clamp(0., (4000. - vw).max(0.))
                .floor(),
            (g.y - (vh / 2.).round())
                .clamp(0., (3000. - vh).max(0.))
                .floor(),
        );
        clear_background(Color::from_hex(0x3ec4f0));
        if self.water.is_none() {
            self.water = Some(Water::new(a));
        }
        let water = self.water.as_mut().unwrap();
        water.update(g.time);
        draw_texture_ex(
            &water.texture,
            (-camera.0 * scale) as f32,
            (-camera.1 * scale) as f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2((4400. * scale) as f32, (3378. * scale) as f32)),
                ..Default::default()
            },
        );
        if !self.initialized {
            self.random = g.random.clone();
            self.initialized = true;
        }
        loop {
            let next = self
                .waves
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let frames = if w.kind == 1 { 41. } else { 55. };
                    (i, w.since + frames / if w.first { 12. } else { 10. })
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();
            if next.1 > g.time {
                break;
            }
            let wave = &mut self.waves[next.0];
            wave.x = (self.random.next() * 2000. + 500.).floor();
            wave.y = (self.random.next() * 2000. + 500.).floor();
            wave.angle += (self.random.next() * 2.).floor() * 180.;
            wave.since = next.1;
            wave.first = false;
        }
        for wave in &self.waves {
            let fps = if wave.first { 12. } else { 10. };
            let frame = ((g.time - wave.since) * fps).floor() as usize;
            sprite(
                a,
                &format!("sprite-wave{}", wave.kind),
                wave.x,
                wave.y,
                500.,
                200.,
                0.,
                0.,
                wave.angle,
                false,
                false,
                frame,
                WHITE,
                camera,
                scale,
            );
        }
        if g.dead {
            draw_penguin(a, g, camera, scale);
        }
        for group in [8, 7, 5, 4, 3, 2, 0] {
            self.group(a, g, group, camera, scale)
        }
        if !g.dead {
            draw_penguin(a, g, camera, scale);
        }
        for group in [1, 6] {
            self.group(a, g, group, camera, scale)
        }
        let age = g.wall - g.transition_at;
        if (0. ..1.75).contains(&age) {
            let border = if age < 1.25 {
                w * ease((age / 0.5).min(1.))
            } else {
                w * (1. - ease(((age - 1.25) / 0.5).min(1.)))
            };
            draw_ring(w, h, border);
            let s = if age < 0.25 {
                0.
            } else {
                cubic_out(((age - 0.25) / 1.).min(1.))
            };
            let alpha = if age < 1.25 {
                1.
            } else {
                (1. - (age - 1.25) / 0.2).max(0.)
            };
            let label = g.level.to_string();
            let metrics = measure_text(&label, Some(&self.font), 128, 1.);
            draw_text_ex(
                &label,
                (w / 2. - metrics.width as f64 * s / 2.) as f32,
                (h / 2. - 75. + 128. * 0.82 * s + 60. * (1. - s)) as f32,
                TextParams {
                    font: Some(&self.font),
                    font_size: 128,
                    font_scale: s as f32,
                    color: Color::new(1., 1., 1., alpha as f32),
                    ..Default::default()
                },
            );
        }
    }
    fn group(&self, a: &Assets, g: &PenguinDash, group: usize, camera: (f64, f64), scale: f64) {
        for e in g.levels[g.visible_level - 1]
            .iter()
            .filter(|e| e.group == group && !e.killed)
        {
            let (x, y) = e.position(g.tween_time);
            let frame = if group == 8 {
                character_frame(&e.key, g.time)
            } else {
                0
            };
            let centered = group == 2 || group == 6;
            sprite(
                a,
                &e.key,
                x,
                y,
                e.width,
                e.height,
                if centered { 0.5 } else { 0. },
                if centered { 0.5 } else { 0. },
                e.rotation,
                e.flip_x,
                e.flip_y,
                frame,
                WHITE,
                camera,
                scale,
            );
        }
    }
}
fn ease(t: f64) -> f64 {
    Ease::Default.at(t)
}
fn cubic_out(t: f64) -> f64 {
    bezier(t, 0.255, 1.005, 0.380, 0.950)
}
fn draw_ring(w: f64, h: f64, b: f64) {
    if b <= 0. {
        return;
    }
    let r = w;
    let inner = (r - b).max(0.);
    let center = vec2((w / 2.) as f32, (h / 2.) as f32);
    let color = Color::from_hex(0xea4437);
    for i in 0..160 {
        let a = i as f32 * std::f32::consts::TAU / 160.;
        let z = (i + 1) as f32 * std::f32::consts::TAU / 160.;
        let v = |q: f32, r: f64| center + vec2(q.cos(), q.sin()) * r as f32;
        draw_triangle(v(a, r), v(z, r), v(a, inner), color);
        draw_triangle(v(z, r), v(z, inner), v(a, inner), color);
    }
}
#[allow(clippy::too_many_arguments)]
fn sprite(
    a: &Assets,
    key: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    ax: f64,
    ay: f64,
    angle: f64,
    fx: bool,
    fy: bool,
    frame: usize,
    color: Color,
    c: (f64, f64),
    s: f64,
) {
    let tex = a.scene("penguindash", &format!("{key}.png"));
    let (fw, fh) = image_size(key);
    let pos = vec2(((x - c.0) * s) as f32, ((y - c.1) * s) as f32);
    draw_texture_ex(
        tex,
        pos.x - (w * ax * s) as f32,
        pos.y - (h * ay * s) as f32,
        color,
        DrawTextureParams {
            dest_size: Some(vec2((w * s) as f32, (h * s) as f32)),
            source: Some(Rect::new(
                0.,
                (fh * frame as f64) as f32,
                fw as f32,
                fh as f32,
            )),
            rotation: angle.to_radians() as f32,
            pivot: Some(pos),
            flip_x: fx,
            flip_y: fy,
            ..Default::default()
        },
    );
}
fn draw_penguin(a: &Assets, g: &PenguinDash, c: (f64, f64), s: f64) {
    let frame = player_frame(g.state, g.time - g.state_since);
    sprite(
        a,
        "sprite-penguin",
        g.x,
        g.y,
        153.,
        108.,
        0.4,
        0.5,
        g.angle,
        false,
        false,
        frame,
        WHITE,
        c,
        s,
    );
    let rot = g.angle.to_radians();
    let child = |x: f64, y: f64| {
        (
            g.x + x * rot.cos() - y * rot.sin(),
            g.y + x * rot.sin() + y * rot.cos(),
        )
    };
    let (x, y) = child(g.shadow.0, g.shadow.1);
    // Shadow is a sprite child (rendered after the penguin), not a separate world layer.
    let tex = a.scene("penguindash", "penguin-shadow.png");
    let pos = vec2(((x - c.0) * s) as f32, ((y - c.1) * s) as f32);
    draw_texture_ex(
        tex,
        pos.x - (153. * 0.4 * s) as f32,
        pos.y - (108. * 0.5 * s) as f32,
        Color::new(1., 1., 1., g.shadow_alpha),
        DrawTextureParams {
            dest_size: Some(vec2((153. * s) as f32, (108. * s) as f32)),
            rotation: rot as f32,
            pivot: Some(pos),
            ..Default::default()
        },
    );
    let (x, y) = child(-30., 0.);
    sprite(
        a,
        "sprite-dust",
        x,
        y,
        150.,
        108.,
        0.4,
        0.5,
        g.angle,
        false,
        false,
        ((g.time * 12.).floor() as usize) % 15,
        Color::new(
            219. / 255.,
            216. / 255.,
            206. / 255.,
            g.dust_alpha.clamp(0., 1.),
        ),
        c,
        s,
    );
}
fn character_frame(key: &str, time: f64) -> usize {
    let t = (time * 10.).floor() as usize;
    match key {
        "sprite-duck" => [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
        ][t % 23],
        "sprite-walrus" => {
            const A: &[usize] = &[
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 16, 16, 15, 14, 13, 13,
                13, 14, 15, 16, 16, 16, 15, 14, 13, 13, 13, 14, 15, 16, 16, 16, 15, 14, 13, 13, 13,
                14, 15, 16, 16, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
            ];
            A[t % A.len()]
        }
        "sprite-green_elf" => t % 17,
        "sprite-orange_elf" => t % 16,
        _ => 0,
    }
}

fn bezier(x: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    let cubic = |t: f64, a: f64, b: f64| {
        3. * (1. - t).powi(2) * t * a + 3. * (1. - t) * t * t * b + t * t * t
    };
    let (mut lo, mut hi) = (0., 1.);
    for _ in 0..48 {
        let t = (lo + hi) / 2.;
        if cubic(t, a, c) < x {
            lo = t
        } else {
            hi = t
        }
    }
    cubic((lo + hi) / 2., b, d)
}
