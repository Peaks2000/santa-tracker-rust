//! Original layered instrument art and CSS keyframe data, evaluated natively.
use crate::{assets::Assets, ui_render::Shapes};
use macroquad::prelude::*;
use santa_tracker::{
    jamband::{Jamband, DRAWER, NAMES},
    ui::{Ease, Transition},
};
use serde::Deserialize;
#[derive(Clone, Deserialize)]
struct Frame {
    offset: f32,
    easing: String,
    transform: String,
    opacity: Option<f32>,
}
#[derive(Clone, Deserialize)]
struct Anim {
    #[serde(default)]
    name: String,
    duration: f32,
    delay: f32,
    paused: bool,
    frames: Vec<Frame>,
}
#[derive(Clone, Deserialize)]
struct Node {
    cls: String,
    xy: [f32; 2],
    wh: [f32; 2],
    origin: Vec<f32>,
    transform: String,
    bg: String,
    asset: String,
    size: String,
    pos: String,
    repeat: String,
    opacity: f32,
    radius: f32,
    clip: bool,
    anims: Vec<Anim>,
    children: Vec<Node>,
}
fn nums(s: &str) -> Vec<f32> {
    s.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e'))
        .filter_map(|x| x.parse().ok())
        .collect()
}
fn rgba(s: &str, alpha: f32) -> Color {
    let n = nums(s);
    if n.len() < 3 {
        return BLANK;
    }
    Color::new(
        n[0] / 255.,
        n[1] / 255.,
        n[2] / 255.,
        n.get(3).copied().unwrap_or(1.) * alpha,
    )
}
#[derive(Clone, Copy)]
struct Tx {
    x: f32,
    y: f32,
    r: f32,
    sx: f32,
    sy: f32,
    skew: f32,
}
fn parse(s: &str, wh: [f32; 2]) -> Tx {
    let mut t = Tx {
        x: 0.,
        y: 0.,
        r: 0.,
        sx: 1.,
        sy: 1.,
        skew: 0.,
    };
    for part in s.split(')').filter(|s| s.contains('(')) {
        let (name, arg) = part.trim().split_once('(').unwrap();
        let a = nums(arg);
        if a.is_empty() {
            continue;
        }
        let v = |i: usize, axis: usize| {
            let raw = arg.split(',').nth(i).unwrap_or("").trim();
            let n = a.get(i).copied().unwrap_or(0.);
            if raw.contains('%') {
                n * wh[axis] / 100.
            } else {
                n
            }
        };
        match name {
            "matrix" => {
                if a.len() >= 6 {
                    t.x = a[4];
                    t.y = a[5];
                    t.r = a[1].atan2(a[0]);
                    t.sx = a[0].hypot(a[1]);
                    t.sy = (a[0] * a[3] - a[1] * a[2]) / t.sx.max(1e-6);
                    t.skew = ((a[0] * a[2] + a[1] * a[3]) / (t.sx * t.sy).max(1e-6)).atan();
                }
            }
            "translate" | "translate3d" => {
                t.x = v(0, 0);
                t.y = v(1, 1)
            }
            "translateX" => t.x = v(0, 0),
            "translateY" => t.y = v(0, 1),
            "rotate" | "rotateZ" => t.r = a[0].to_radians(),
            "scale" => {
                t.sx = a[0];
                t.sy = a.get(1).copied().unwrap_or(a[0]);
            }
            "scaleX" => t.sx = a[0],
            "scaleY" => t.sy = a[0],
            "skew" | "skewX" => t.skew = a[0].to_radians(),
            _ => {}
        }
    }
    t
}
fn matrix(t: Tx) -> Mat3 {
    Mat3::from_translation(vec2(t.x, t.y))
        * Mat3::from_angle(t.r)
        * Mat3::from_cols(
            vec3(1., 0., 0.),
            vec3(t.skew.tan(), 1., 0.),
            vec3(0., 0., 1.),
        )
        * Mat3::from_scale(vec2(t.sx, t.sy))
}
fn interpolate(a: Tx, b: Tx, t: f32) -> Tx {
    let f = |x, y| x + (y - x) * t;
    Tx {
        x: f(a.x, b.x),
        y: f(a.y, b.y),
        r: f(a.r, b.r),
        sx: f(a.sx, b.sx),
        sy: f(a.sy, b.sy),
        skew: f(a.skew, b.skew),
    }
}
fn easing(s: &str, t: f32) -> f32 {
    match s {
        "linear" => Ease::Linear,
        "ease-in" => Ease::In,
        "ease-out" => Ease::Out,
        "ease-in-out" => Ease::InOut,
        _ => Ease::Default,
    }
    .at(t as f64) as f32
}
fn animation(n: &Node, t: f32, waiting: Option<f32>) -> (Mat3, f32) {
    let mut tx = parse(&n.transform, n.wh);
    let mut alpha = n.opacity;
    for a in &n.anims {
        let t = if a.paused {
            0.
        } else if a.name.contains("waiting") {
            waiting.unwrap_or(t)
        } else {
            t
        };
        if t < a.delay {
            continue;
        }
        let phase = ((t - a.delay) / a.duration).rem_euclid(1.);
        let i = a
            .frames
            .iter()
            .rposition(|f| f.offset <= phase)
            .unwrap_or(0)
            .min(a.frames.len() - 2);
        let (f, g) = (&a.frames[i], &a.frames[i + 1]);
        let p = easing(
            &f.easing,
            ((phase - f.offset) / (g.offset - f.offset)).clamp(0., 1.),
        );
        if f.transform != "none" || g.transform != "none" {
            tx = interpolate(parse(&f.transform, n.wh), parse(&g.transform, n.wh), p);
        }
        if let (Some(x), Some(y)) = (f.opacity, g.opacity) {
            alpha = x + (y - x) * p;
        }
    }
    let o = vec2(n.origin[0], n.origin[1]);
    (
        Mat3::from_translation(o) * matrix(tx) * Mat3::from_translation(-o),
        alpha,
    )
}
fn quad(t: Option<&Texture2D>, r: Rect, m: Mat3, c: Color) {
    let uv = [vec2(0., 0.), vec2(1., 0.), vec2(1., 1.), vec2(0., 1.)];
    let vertices = uv
        .into_iter()
        .map(|u| {
            let p = m.transform_point2(r.point() + u * r.size());
            Vertex::new(p.x, p.y, 0., u.x, u.y, c)
        })
        .collect();
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: t.cloned(),
    });
}
pub struct JambandView {
    data: std::collections::HashMap<String, Node>,
    shapes: Shapes,
    premult: Material,
    drag: Option<(usize, Vec2, Rect)>,
    scroll: Transition<1>,
    scroll_target: f32,
    drawer: Transition<1>,
    hidden: bool,
    collapse: [Transition<1>; 10],
    press: [Transition<1>; 4],
    pub share_open: bool,
    start: f64,
    waiting_clock: std::cell::Cell<Option<f32>>,
    floor_clock: std::cell::Cell<f32>,
    light_fade: Transition<4>,
    last_time: f64,
}
impl JambandView {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            data: {
                let mut v: serde_json::Value =
                    serde_json::from_str(include_str!("../data/jamband-art.json")).unwrap();
                let mut m = std::collections::HashMap::new();
                for (k, n) in v["instruments"].as_object().unwrap() {
                    m.insert(k.clone(), serde_json::from_value(n.clone()).unwrap());
                }
                v.as_object_mut().unwrap().remove("instruments");
                for (k, n) in v.as_object().unwrap() {
                    m.insert(k.clone(), serde_json::from_value(n.clone()).unwrap());
                }
                m
            },
            shapes: Shapes::new()?,
            premult: crate::ui_render::premultiplied_material()?,
            drag: None,
            scroll: Transition::new([0.]),
            scroll_target: 0.,
            drawer: Transition::new([0.]),
            hidden: false,
            collapse: std::array::from_fn(|_| Transition::new([210.])),
            press: std::array::from_fn(|_| Transition::new([0.])),
            share_open: false,
            start: 0.,
            waiting_clock: std::cell::Cell::new(None),
            floor_clock: std::cell::Cell::new(0.),
            light_fade: Transition::new([0.; 4]),
            last_time: 0.,
        })
    }
    pub fn reset(&mut self) {
        self.drag = None;
        self.scroll = Transition::new([0.]);
        self.scroll_target = 0.;
        self.hidden = false;
        self.drawer = Transition::new([0.]);
        self.start = get_time();
        self.share_open = false;
        self.collapse = std::array::from_fn(|_| Transition::new([210.]));
        self.floor_clock.set(0.);
        self.light_fade = Transition::new([0.; 4]);
        self.last_time = 0.;
    }
    pub fn stages(w: f32, h: f32) -> [Rect; 6] {
        let s = (w / 1800.).min(1.);
        let c = w / 2.;
        let y = h / 2.;
        [
            Rect::new(c + 356. * s, y - 260. * s, 216. * s, 270. * s),
            Rect::new(c + 129. * s, y - 330. * s, 216. * s, 270. * s),
            Rect::new(c - 91. * s, y - 410. * s, 216. * s, 270. * s),
            Rect::new(c - 315. * s, y - 330. * s, 216. * s, 270. * s),
            Rect::new(c - 542. * s, y - 260. * s, 216. * s, 270. * s),
            Rect::new(c - 215. * s, y - 115. * s, 347. * s, 270. * s),
        ]
    }
    fn node(
        &self,
        a: &Assets,
        n: &Node,
        parent: Mat3,
        t: f32,
        alpha: f32,
        viewport: Vec2,
        environment: bool,
        clip: Option<Rect>,
    ) {
        let mut n = n.clone();
        if environment {
            let (w, h) = (viewport.x, viewport.y);
            let cls = n.cls.as_str();
            if cls == "scene" || cls == "Background" {
                n.wh = [w, h];
                n.xy = [0., 0.];
            } else if cls == "Background-mountains" {
                n.wh = [w, h / 2.];
            } else if cls == "Background-trees" {
                n.wh = [w, h];
                n.xy = [0., 83. - h / 2.];
            } else if cls == "Background-snow" {
                n.wh = [w, 60.];
                n.xy = [0., h / 2.];
            } else if cls == "Background-stageBackdrop" {
                n.xy = [w / 2., h / 2.];
            } else if cls == "Background-stage--top" {
                n.wh[0] = w;
                n.xy = [0., h / 2. + 60.];
            } else if cls == "Background-stage--front" {
                n.wh[0] = w;
                n.xy = [0., h / 2. + 223.];
            } else if cls == "Background-stage" {
                n.wh = [w, h];
                n.xy = [0., -h / 2. - 223.];
            } else if cls == "Lights" {
                n.wh[1] = h;
                n.xy = [w / 2., 0.];
            } else if cls.starts_with("Light-floor") || cls.starts_with("Light-ceiling") {
                if !cls.contains("::") {
                    n.xy[1] += h / 2. - 500.;
                }
            } else if cls.starts_with("Background-crowd") {
                n.xy[1] = h / 2. + 140.;
            }
        }
        let floor = n.cls.starts_with("Light-floor");
        let (tx, mut op) = animation(
            &n,
            if floor { self.floor_clock.get() } else { t },
            self.waiting_clock.get(),
        );
        let lights = self.light_fade.value(self.last_time);
        if n.cls.starts_with("Light-floor") && n.cls.ends_with("::before") {
            op = lights[0] as f32;
        }
        if n.cls.starts_with("Light-floor") && n.cls.ends_with("::after") {
            op = lights[1] as f32;
        }
        if n.cls.starts_with("Light-ceiling") && n.cls.ends_with("::before") {
            op = lights[2] as f32;
        }
        if n.cls.starts_with("Light-ceiling") && n.cls.ends_with("::after") {
            op *= lights[3] as f32;
        }
        let m = parent * Mat3::from_translation(Vec2::from(n.xy)) * tx;
        let alpha = alpha * op;
        let mut current_clip = clip;
        let r = Rect::new(0., 0., n.wh[0], n.wh[1]);
        if n.clip {
            let p = m.transform_point2(Vec2::ZERO);
            let q = m.transform_point2(vec2(r.w, r.h));
            let bounds = Rect::new(
                p.x.min(q.x),
                p.y.min(q.y),
                (q.x - p.x).abs(),
                (q.y - p.y).abs(),
            );
            current_clip = Some(if let Some(c) = clip {
                c.intersect(bounds).unwrap_or(Rect::new(0., 0., 0., 0.))
            } else {
                bounds
            });
        }
        if let Some(c) = current_clip {
            unsafe {
                get_internal_gl()
                    .quad_gl
                    .scissor(Some((c.x as i32, c.y as i32, c.w as i32, c.h as i32)));
            }
        }
        let bg = rgba(&n.bg, alpha);
        if bg.a > 0. {
            if n.radius > 0. && n.wh[0] == n.wh[1] {
                let p = m.transform_point2(r.center());
                let scale = m.x_axis.truncate().length();
                self.shapes.circle(p, r.w / 2. * scale, bg, 0.);
            } else {
                quad(None, r, m, bg)
            }
        }
        if !n.asset.is_empty() {
            let key = format!("scenes/jamband/{}", n.asset);
            if a.has(&key) {
                let tex = a.texture(&key);
                let natural = vec2(tex.width(), tex.height());
                let size = if n.size == "contain" {
                    natural * (r.w / natural.x).min(r.h / natural.y)
                } else {
                    let v = nums(&n.size);
                    if v.len() >= 2 {
                        vec2(v[0], v[1])
                    } else {
                        natural
                    }
                };
                let pos = nums(&n.pos);
                let p = vec2(
                    (r.w - size.x) * pos.first().copied().unwrap_or(0.) / 100.,
                    (r.h - size.y) * pos.get(1).copied().unwrap_or(0.) / 100.,
                );
                let rr = Rect::new(p.x, p.y, size.x, size.y);
                let color = Color::new(1., 1., 1., alpha);
                quad(Some(tex), rr, m, color);
                if n.repeat == "repeat-x" {
                    for offset in [-size.x, size.x] {
                        quad(
                            Some(tex),
                            Rect::new(rr.x + offset, rr.y, rr.w, rr.h),
                            m,
                            color,
                        )
                    }
                }
            }
        }
        for child in &n.children {
            self.node(a, child, m, t, alpha, viewport, environment, current_clip)
        }
        unsafe {
            get_internal_gl()
                .quad_gl
                .scissor(clip.map(|c| (c.x as i32, c.y as i32, c.w as i32, c.h as i32)));
        }
    }
    pub fn draw(&mut self, a: &Assets, g: &mut Jamband, input: bool) {
        let (w, h) = (screen_width(), screen_height());
        let s = (w / 1800.).min(1.);
        let gui = (s * 1.5).min(1.);
        let m = Vec2::from(mouse_position());
        let now = g.time;
        let time = g.time as f32;
        let delta = (g.time - self.last_time).max(0.);
        self.last_time = g.time;
        if g.count > 3 {
            self.floor_clock.set(self.floor_clock.get() + delta as f32);
        }
        self.light_fade.set(
            g.time,
            [
                if g.count > 2 && g.count <= 4 { 1. } else { 0. },
                if g.count > 4 { 1. } else { 0. },
                if g.count > 0 && g.count <= 5 { 1. } else { 0. },
                if g.count > 5 { 1. } else { 0. },
            ],
            0.5,
            Ease::Default,
        );
        self.waiting_clock.set(None);
        let level = if g.count >= 6 {
            6
        } else if g.count >= 5 {
            5
        } else if g.count >= 4 {
            4
        } else if g.count >= 3 {
            3
        } else if g.count >= 1 {
            1
        } else {
            0
        };
        clear_background(WHITE);
        let env = &self.data[&format!("environment{level}")];
        self.node(
            a,
            &env.children[0],
            Mat3::from_scale(vec2(s, s)),
            time,
            1.,
            vec2(w / s, h / s),
            true,
            None,
        );
        let stages = Self::stages(w, h);
        for (stage, r) in stages.iter().enumerate() {
            if g.stages[stage].is_none()
                && (g.stages.iter().all(Option::is_none) || self.drag.is_some())
            {
                let t = a.scene("jamband", "stage-arrow.svg");
                let bounce = if time % 1. < 0.4 {
                    -30. * (1. - easing("ease", time % 1. / 0.4))
                } else {
                    -30. * easing("ease", (time % 1. - 0.4) / 0.6)
                };
                draw_texture_ex(
                    t,
                    r.x + (r.w - 98.1 * s) / 2.,
                    r.y + (r.h - 203.7 * s) * 0.9 + bounce * s + 30.05 * s,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(98.1, 143.6) * s),
                        ..Default::default()
                    },
                );
            }
            if let Some(i) = g.stages[stage] {
                if self.drag.as_ref().is_none_or(|d| d.0 != i) {
                    let state = if let Some(p) = g.playing[i] {
                        format!("playing-{p}")
                    } else {
                        "waiting".into()
                    };
                    let mut stage_node = self.data[&format!("{}-{state}", NAMES[i])].clone();
                    stage_node.clip = !matches!(i, 0 | 1 | 3 | 5 | 6);
                    let n = &stage_node;
                    let margin = [35., 6., 56., 36., 30., 23., 55., 36., 50., -7.][i];
                    let x = r.x + (margin + if stage == 5 { 70. } else { 0. }) * s;
                    self.waiting_clock.set(Some((now - g.waiting_at[i]) as f32));
                    let clock = if g.playing[i].is_some() {
                        now - g.started_at[i]
                    } else {
                        now - g.waiting_at[i]
                    };
                    self.node(
                        a,
                        n,
                        Mat3::from_translation(vec2(x, r.y)) * Mat3::from_scale(vec2(s, s)),
                        clock as f32,
                        1.,
                        vec2(w, h),
                        false,
                        None,
                    );
                    if input
                        && is_mouse_button_pressed(MouseButton::Left)
                        && Rect::new(x, r.y, n.wh[0] * s, n.wh[1] * s).contains(m)
                        && !crate::chrome::over_controls(m, 2)
                    {
                        self.drag = Some((i, m, Rect::new(x, r.y, n.wh[0] * s, n.wh[1] * s)));
                    }
                }
            }
        }
        self.waiting_clock.set(None);
        // .Lights creates a z-index:1 stacking layer above stage instruments;
        // stage LEDs share that level and occur later in source DOM order.
        self.node(
            a,
            &env.children[1],
            Mat3::from_scale(vec2(s, s)),
            time,
            1.,
            vec2(w / s, h / s),
            true,
            None,
        );
        for (stage, r) in stages.iter().enumerate() {
            let color = if stage == 5 {
                "red"
            } else if stage % 2 == 0 {
                "green"
            } else {
                "blue"
            };
            let t = a.scene("jamband", &format!("lights/stage-{color}.svg"));
            let sz = vec2(if stage == 5 { 346.9 } else { 170.6 }, 27.7) * s;
            draw_texture_ex(
                t,
                r.x + (r.w - sz.x) * 0.1,
                r.bottom() - sz.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(sz),
                    ..Default::default()
                },
            );
            let t = a.scene("jamband", &format!("lights/stage-{color}--on.svg"));
            let phase = (time - if color == "green" { 0.5 } else { 0. }).rem_euclid(1.);
            let alpha = if phase < 0.5 {
                1. - easing("ease", phase * 2.)
            } else {
                easing("ease", (phase - 0.5) * 2.)
            };
            draw_texture_ex(
                t,
                r.x + (r.w - sz.x) * 0.1,
                r.bottom() - sz.y,
                Color::new(1., 1., 1., alpha),
                DrawTextureParams {
                    dest_size: Some(sz),
                    ..Default::default()
                },
            );
        }
        self.waiting_clock.set(None);
        let dy = self.drawer.value(now)[0] as f32;
        let y = h - 191. * gui + dy;
        draw_rectangle(0., y, w, 191. * gui, Color::from_hex(0x374289));
        let tab = Rect::new(w - 97. * gui, y - 45. * gui, 73. * gui, 127. * gui);
        draw_texture_ex(
            a.scene("jamband", "drawer-tab.svg"),
            tab.x,
            tab.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(tab.size()),
                ..Default::default()
            },
        );
        let icon = if self.hidden { "open" } else { "close" };
        draw_texture_ex(
            a.scene("jamband", &format!("icons/{icon}.svg")),
            tab.x + 25. * gui,
            tab.y + 20. * gui,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(23., 23.) * gui),
                ..Default::default()
            },
        );
        let click = input && is_mouse_button_released(MouseButton::Left);
        if click && tab.contains(m) && self.drag.is_none() {
            self.hidden = !self.hidden;
            self.drawer.set(
                now,
                [if self.hidden { 191. * gui as f64 } else { 0. }],
                0.3,
                Ease::Default,
            );
        }
        let left = if w <= 1024. { 20. * gui } else { 200. * gui };
        let right = w - 50. * gui;
        let button_size = if w >= 768. { 55. } else { 42. };
        let wrapper = if w >= 768. { 45. } else { 34. };
        let controls = wrapper * 2. + (button_size + 8.) * 2.;
        let tray = Rect::new(
            left + wrapper,
            y,
            (right - left - controls).max(1.),
            270.4 * gui,
        );
        draw_rectangle(tray.x, tray.y, tray.w, tray.h, Color::from_hex(0x3e50a5));
        let scroll = self.scroll.value(now)[0] as f32;
        let mut x = tray.x - scroll;
        let mut hit = None;
        for i in DRAWER {
            let collapsed = g.location(i).is_some() && self.drag.as_ref().is_none_or(|d| d.0 != i);
            self.collapse[i].set(now, [if collapsed { 0. } else { 210. }], 0.3, Ease::Default);
            let cw = self.collapse[i].value(now)[0] as f32;
            if cw < 0.01 {
                continue;
            }
            let n = &self.data[&format!("{}-idle", NAMES[i])];
            let r = Rect::new(
                x + (210. - n.wh[0]) * gui / 2.,
                y,
                n.wh[0] * gui,
                n.wh[1] * gui,
            );
            if !collapsed && self.drag.as_ref().is_none_or(|d| d.0 != i) {
                unsafe {
                    get_internal_gl().quad_gl.scissor(Some((
                        tray.x as i32,
                        tray.y as i32,
                        tray.w as i32,
                        tray.h as i32,
                    )));
                }
                let mat = Mat3::from_translation(r.point() + r.size() * 0.1)
                    * Mat3::from_scale(vec2(gui * 0.8, gui * 0.8));
                self.node(a, n, mat, 0., 1., vec2(w, h), false, Some(tray));
                unsafe {
                    get_internal_gl().quad_gl.scissor(None);
                }
                if r.contains(m) && tray.contains(m) {
                    hit = Some((i, r));
                }
            }
            x += cw * gui;
        }
        let max = (x + scroll - tray.x - tray.w).max(0.);
        if input && tray.contains(m) {
            let wheel = mouse_wheel().1;
            if wheel != 0. {
                self.scroll_target = (self.scroll_target - wheel * 80.).clamp(0., max);
                self.scroll
                    .set(now, [self.scroll_target as f64], 0.3, Ease::Default);
            }
        }
        for (i, key) in [
            "arrow-key-left-dark.svg",
            "arrow-key-right-dark.svg",
            "shuffle.svg",
            "share.svg",
        ]
        .iter()
        .enumerate()
        {
            let bx = match i {
                0 => left + if w >= 768. { 14. } else { 7. },
                1 => tray.right() - if w >= 768. { 24. } else { 15. },
                2 => right - 2. * (button_size + 8.) + 4.,
                _ => right - button_size - 4.,
            };
            let r = Rect::new(
                bx,
                y + (191. * gui - button_size) / 2.,
                button_size,
                button_size,
            );
            let held = input && r.contains(m) && is_mouse_button_down(MouseButton::Left);
            self.press[i].set(
                now,
                [if held { 1. } else { 0. }],
                if held { 0.05 } else { 0.25 },
                Ease::InOut,
            );
            let p = self.press[i].value(now)[0] as f32;
            let c = r.center() + vec2(-3., -6.) * (1. - p);
            let base = Color::from_hex(if i < 2 { 0xefefef } else { 0xff3f00 });
            let shadow = Color::from_hex(if i < 2 { 0xc1dbd9 } else { 0xf52f00 });
            if i < 2 {
                self.shapes.circle(
                    r.center(),
                    button_size / 2. + if w >= 768. { 14. } else { 7. },
                    Color::from_hex(0x3e50a5),
                    0.,
                );
            }
            self.shapes.circle(
                c + vec2(8., 9.) * (1. - p),
                button_size / 2.,
                Color::new(0., 0., 0., 0.16),
                0.,
            );
            self.shapes
                .circle(c + vec2(3., 6.) * (1. - p), button_size / 2., shadow, 0.);
            self.shapes.circle(c, button_size / 2., base, 0.);
            let tex = a.texture(&format!("scenes/_shared/img/button/{key}"));
            let sz = if w >= 768. { 30. } else { 22. };
            gl_use_material(&self.premult);
            draw_texture_ex(
                tex,
                c.x - sz / 2.,
                c.y - sz / 2.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(sz, sz)),
                    ..Default::default()
                },
            );
            gl_use_default_material();
            if click && r.contains(m) && self.drag.is_none() {
                match i {
                    0 | 1 => {
                        self.scroll_target = (self.scroll_target
                            + if i == 0 { -210. * gui } else { 210. * gui })
                        .clamp(0., max);
                        self.scroll
                            .set(now, [self.scroll_target as f64], 0.3, Ease::InOut);
                    }
                    2 => g.randomize(),
                    3 => {
                        self.share_open = true;
                        let _ = std::fs::write("jamband.band", g.save());
                    }
                    _ => {}
                }
            }
        }
        if input && self.drag.is_none() && is_mouse_button_pressed(MouseButton::Left) {
            if let Some((i, r)) = hit {
                g.preview(i);
                self.drag = Some((i, m, r));
            }
        }
        if let Some((i, start, r)) = self.drag {
            self.waiting_clock.set(Some((now - g.waiting_at[i]) as f32));
            let state = if let Some(p) = g.playing[i] {
                format!("playing-{p}")
            } else if g.waiting[i] {
                "waiting".into()
            } else {
                "idle".into()
            };
            let n = &self.data[&format!("{}-{state}", NAMES[i])];
            self.node(
                a,
                n,
                Mat3::from_translation(r.point() + m - start)
                    * Mat3::from_scale(vec2(
                        if r.y == y { gui } else { s },
                        if r.y == y { gui } else { s },
                    )),
                time,
                1.,
                vec2(w, h),
                false,
                None,
            );
            if input && is_mouse_button_released(MouseButton::Left) {
                let target = stages
                    .iter()
                    .position(|r| r.x < m.x && m.x < r.right() && r.y < m.y && m.y < r.bottom());
                g.drop(i, target);
                self.drag = None;
            }
        }
        if self.share_open {
            let r = Rect::new(w / 2. - 220., h / 2. - 65., 440., 130.);
            self.shapes.rounded(r, 12., WHITE, 0.);
            draw_text(
                "Band saved to jamband.band",
                r.x + 20.,
                r.y + 45.,
                26.,
                DARKGRAY,
            );
            draw_text("Click to return", r.x + 20., r.y + 88., 22., DARKGRAY);
            if click {
                self.share_open = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn instrument_keyframes_match_original_browser_matrices() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/jamband-art.json")).unwrap();
        let samples: serde_json::Value =
            serde_json::from_str(include_str!("../data/jamband-animation-reference.json")).unwrap();
        for row in samples.as_array().unwrap() {
            let mut node: Node =
                serde_json::from_value(data["instruments"][row["key"].as_str().unwrap()].clone())
                    .unwrap();
            for i in row["path"].as_array().unwrap() {
                node = node.children[i.as_u64().unwrap() as usize].clone();
            }
            let t = row["time"].as_f64().unwrap() as f32;
            let (actual, alpha) = animation(&node, t, None);
            let o = vec2(node.origin[0], node.origin[1]);
            let expected = Mat3::from_translation(o)
                * matrix(parse(row["transform"].as_str().unwrap(), node.wh))
                * Mat3::from_translation(-o);
            for (a, b) in actual.to_cols_array().iter().zip(expected.to_cols_array()) {
                assert!(
                    (a - b).abs() < 0.02,
                    "{} {:?} t={t}: {a} vs {b}",
                    row["key"],
                    row["path"]
                );
            }
            assert!((alpha - row["opacity"].as_f64().unwrap() as f32).abs() < 0.001);
        }
    }
}
