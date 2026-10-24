//! Original Glider SVG/sprite layers and CSS keyframes, rendered natively.
use crate::assets::Assets;
use macroquad::prelude::*;
use santa_tracker::glider::{Glider, Kind};
use santa_tracker::ui::Ease;
pub struct GliderView {
    font: Font,
    buildings: Vec<(f32, f32)>,
    last_size: (f32, f32),
    keys: [bool; 5],
}
fn bg(t: &Texture2D, r: Rect, angle: f32, alpha: f32) {
    let scale = (r.w / t.width()).min(r.h / t.height());
    draw_texture_ex(
        t,
        r.x,
        r.y,
        Color::new(1., 1., 1., alpha),
        DrawTextureParams {
            dest_size: Some(vec2(t.width(), t.height()) * scale),
            rotation: angle,
            pivot: Some(r.center()),
            ..Default::default()
        },
    );
}
fn sprite(a: &Assets, r: Rect, x: f32, y: f32, w: f32, h: f32, angle: f32, alpha: f32) {
    let t = a.scene("glider", "glider-sprite.svg");
    draw_texture_ex(
        t,
        r.x,
        r.y,
        Color::new(1., 1., 1., alpha),
        DrawTextureParams {
            dest_size: Some(r.size()),
            source: Some(Rect::new(
                x * t.width() / 400.,
                y * t.height() / 600.,
                w * t.width() / 400.,
                h * t.height() / 600.,
            )),
            rotation: angle,
            pivot: Some(r.center()),
            ..Default::default()
        },
    );
}
pub fn normal_row(t: f64) -> f32 {
    let p = (t / 0.8).rem_euclid(1.);
    if p < 0.25 {
        1.
    } else if p < 0.5 {
        2.
    } else if p < 0.75 {
        1.
    } else {
        0.
    }
}
pub fn obstacle_row(t: f64) -> f32 {
    let p = (t / 0.6).rem_euclid(1.);
    if p < 0.17 {
        1.
    } else if p < 0.34 {
        2.
    } else if p < 0.5 {
        3.
    } else if p < 0.67 {
        2.
    } else {
        0.
    }
}
impl GliderView {
    pub fn new() -> Result<Self, String> {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/glider-layout-reference.json")).unwrap();
        Ok(Self {
            font: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/OpenSans-ExtraBold.ttf"
            ))
            .map_err(|e| e.to_string())?,
            buildings: data["buildings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| {
                    (
                        b["width"].as_f64().unwrap() as f32,
                        b["height"].as_f64().unwrap() as f32,
                    )
                })
                .collect(),
            last_size: (0., 0.),
            keys: [false; 5],
        })
    }
    pub fn resize(&mut self, g: &mut Glider) {
        let size = (screen_width(), screen_height());
        let (w, h, _) = Glider::dimensions(size.0 as f64, size.1 as f64);
        if self.last_size != size {
            if self.last_size != (0., 0.) {
                g.player.anim_base = g.time + 0.1;
                g.player.anim_at = g.wall + 0.1;
                for id in g.active.clone() {
                    if g.entities[id].kind == Kind::Obstacle {
                        g.entities[id].anim_at = g.time + 0.1;
                    }
                }
            }
            self.last_size = size;
        }
        g.width = w;
        g.height = h;
    }
    pub fn reset_input(&mut self) {
        self.keys = [false; 5];
    }
    pub fn input(&mut self, _block_mouse: bool) -> (f64, f64, bool) {
        // Source ignores both keydown and keyup while paused, retaining held keys.
        for (i, key) in [
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Space,
        ]
        .into_iter()
        .enumerate()
        {
            if is_key_pressed(key) {
                self.keys[i] = true;
            }
            if is_key_released(key) {
                self.keys[i] = false;
            }
        }
        (
            (i32::from(self.keys[1]) - i32::from(self.keys[0])) as f64 * 0.6,
            (i32::from(self.keys[3]) - i32::from(self.keys[2])) as f64 * 0.6,
            self.keys[4],
        )
    }
    pub fn draw(&self, a: &Assets, g: &Glider) {
        let (w, h) = (screen_width(), screen_height());
        let (_, _, em) = Glider::dimensions(w as f64, h as f64);
        let e = em as f32;
        let top = Color::from_hex(0x2369ba);
        let bottom = Color::from_hex(0x7d5ebc);
        draw_mesh(&Mesh {
            vertices: vec![
                Vertex::new(0., 0., 0., 0., 0., top),
                Vertex::new(w, 0., 0., 1., 0., top),
                Vertex::new(w, h, 0., 1., 1., bottom),
                Vertex::new(0., h, 0., 0., 1., bottom),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            texture: None,
        });
        for (name, width, height, duration) in [
            ("stars-tile.svg", w * 2., h, 1000.),
            ("bg-mountains.svg", w * 10., 50. * e, 500.),
            ("bg-tile-trees.svg", w * 2., 21. * e, 50.),
        ] {
            let t = a.scene("glider", name);
            let sc = (width / t.width()).min(height / t.height());
            let (tw, th) = (t.width() * sc, t.height() * sc);
            let shift = -(width * 0.5) * (g.bg_time as f32 / duration).fract();
            let x = shift.rem_euclid(tw) - tw;
            let mut xx = x;
            while xx < w {
                draw_texture_ex(
                    t,
                    xx,
                    h - height,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(tw, th)),
                        ..Default::default()
                    },
                );
                xx += tw;
            }
        }
        for kind in [
            Kind::Cloud,
            Kind::Collision,
            Kind::Item,
            Kind::Obstacle,
            Kind::Present,
            Kind::Building,
        ] {
            for (id, obj) in g.entities.iter().enumerate() {
                if !g.active.contains(&id) {
                    continue;
                }
                if obj.kind != kind || obj.dead {
                    continue;
                }
                let x = obj.x as f32 * e + if obj.kind == Kind::Present { 0. } else { 8. };
                let y = obj.y as f32 * e + if obj.kind == Kind::Collision { 8. } else { 0. };
                let age = (g.time - obj.anim_at).max(0.);
                match kind {
                    Kind::Cloud => {
                        let sc = obj.size as f32;
                        bg(
                            a.scene("glider", "cloud.svg"),
                            Rect::new(
                                x + 9. * e * (1. - sc),
                                y + 5.4 * e * (1. - sc),
                                18. * e * sc,
                                10.8 * e * sc,
                            ),
                            0.,
                            1.,
                        );
                    }
                    Kind::Collision => {
                        let p = ((g.wall - obj.born) / 0.5).clamp(0., 1.) as f32;
                        let (sc, alpha) = if p < 0.33 {
                            (1. + 2. * p / 0.33, 1.)
                        } else if p < 0.66 {
                            (3. - 1.5 * (p - 0.33) / 0.33, 1. - 0.5 * (p - 0.33) / 0.33)
                        } else {
                            (1.5, 0.5 * (1. - (p - 0.66) / 0.34))
                        };
                        let center = vec2(x + 4.5 * e, y + 5. * e);
                        sprite(
                            a,
                            Rect::new(
                                center.x - 4.5 * e * sc,
                                center.y - 5. * e * sc,
                                9. * e * sc,
                                10. * e * sc,
                            ),
                            210.,
                            400.,
                            90.,
                            100.,
                            0.,
                            alpha,
                        );
                    }
                    Kind::Item => {
                        if obj.hit {
                            let t = g.wall - obj.hit_at;
                            let alpha = if t <= 0.5 {
                                1.
                            } else {
                                1. - Ease::Default.at((t - 0.5) / 1.5) as f32
                            };
                            let s = if obj.style == 0 { "200" } else { "+00:05" };
                            let width = measure_text(s, Some(&self.font), 40, 1.).width;
                            draw_text_ex(
                                s,
                                x - width / 2.,
                                y - 25. + 40. * 0.8,
                                TextParams {
                                    font: Some(&self.font),
                                    font_size: 40,
                                    color: Color::new(1., 1., 1., alpha),
                                    ..Default::default()
                                },
                            );
                        } else if obj.style == 0 {
                            bg(
                                a.scene("glider", "treat-candy.svg"),
                                Rect::new(x, y, 6. * e, 6. * e),
                                -(age as f32 / 4.).fract() * std::f32::consts::TAU,
                                1.,
                            );
                        } else {
                            bg(
                                a.scene("glider", "orb-time-back.svg"),
                                Rect::new(x, y, 6. * e, 6. * e),
                                -(age as f32 / 2.).fract() * std::f32::consts::TAU,
                                1.,
                            );
                            bg(
                                a.scene("glider", "orb-time-front.svg"),
                                Rect::new(x, y, 6. * e, 6. * e),
                                0.,
                                1.,
                            );
                        }
                    }
                    Kind::Obstacle => {
                        let y = y + obj.variance as f32 * e;
                        match obj.style {
                            0 | 3 => {
                                let row = if g.time < obj.anim_at {
                                    0.
                                } else {
                                    obstacle_row(age)
                                };
                                let alpha = if obj.popping {
                                    1. - Ease::Default.at((g.time - obj.hit_anim) / 0.6) as f32
                                } else {
                                    1.
                                };
                                sprite(
                                    a,
                                    Rect::new(x, y, 9. * e, 10. * e),
                                    if obj.style == 0 { 310. } else { 210. },
                                    if obj.popping { 400. } else { row * 100. },
                                    90.,
                                    100.,
                                    0.,
                                    alpha,
                                );
                            }
                            1 | 2 => bg(
                                a.scene(
                                    "glider",
                                    if obj.style == 1 {
                                        "hit-beam.svg"
                                    } else {
                                        "hit-box.svg"
                                    },
                                ),
                                Rect::new(
                                    x,
                                    y,
                                    11.2 * e,
                                    if obj.style == 1 { 63. * e } else { 75.5 * e },
                                ),
                                0.,
                                1.,
                            ),
                            _ => {
                                let row = if (age / 0.3).fract() < 0.5 {
                                    550.
                                } else {
                                    500.
                                };
                                sprite(
                                    a,
                                    Rect::new(x, y, 9. * e, 4. * e),
                                    210.,
                                    row,
                                    90.,
                                    40.,
                                    0.,
                                    1.,
                                );
                            }
                        }
                    }
                    Kind::Present => {
                        if obj.style > 0 {
                            bg(
                                a.scene("glider", &format!("present-{:02}.svg", obj.style)),
                                Rect::new(x, y, 2.5 * e, 2.5 * e),
                                0.,
                                1.,
                            );
                        }
                    }
                    Kind::Building => {
                        let (bw, bh) = self.buildings[obj.style];
                        let top = h - bh * e;
                        let name = if obj.style < 24 {
                            format!("hit-house-{:02}.svg", obj.style + 1)
                        } else {
                            format!("hit-tree-{}.svg", obj.style - 23)
                        };
                        bg(
                            a.scene("glider", &name),
                            Rect::new(x, top, bw * e, bh * e),
                            0.,
                            1.,
                        );
                        for (i, hit) in obj.chimney_hits.iter().enumerate() {
                            if let Some(at) = hit {
                                let c = &g.buildings[obj.style].chimneys[i];
                                let (cx, cy) = if obj.style == 2 {
                                    (c.xpos, 0.1)
                                } else if obj.style == 21 {
                                    (9.2, c.ypos)
                                } else {
                                    (c.xpos, c.ypos)
                                };
                                let left = match c.points {
                                    100 => -1.5,
                                    200 => -1.9,
                                    _ => -2.5,
                                };
                                bg(
                                    a.scene("glider", &format!("points-{}.svg", c.points)),
                                    Rect::new(
                                        x + (cx as f32 + left) * e,
                                        top + (cy as f32 - 8.) * e,
                                        8. * e,
                                        7. * e,
                                    ),
                                    0.,
                                    1. - Ease::Default.at((g.wall - at) / 2.) as f32,
                                );
                            }
                        }
                    }
                }
            }
        }
        let p = &g.player;
        let phase_time = (g.css_time - g.player_epoch).max(0.);
        let mut row = if p.crash {
            5.
        } else if p.dropping {
            3.
        } else {
            0.
        };
        let mut alpha = 1.;
        match g.player_mode {
            1 => row = normal_row(phase_time),
            2 => row = if phase_time < 0.1 { 3. } else { 4. },
            3 => {
                let phase = (phase_time / 0.5).rem_euclid(1.);
                alpha = if phase < 0.5 {
                    1. - 1.6 * phase as f32
                } else {
                    0.2 + 1.6 * (phase as f32 - 0.5)
                };
            }
            _ => {}
        }
        sprite(
            a,
            Rect::new(p.x as f32 * e, p.y as f32 * e, 20. * e, 10. * e),
            0.,
            row * 100.,
            200.,
            100.,
            (p.vy * 25.).to_radians() as f32,
            alpha,
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_sprite_step_start_boundaries() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/glider-layout-reference.json")).unwrap();
        for f in data["frames"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["cls"] == "animated")
        {
            let t = f["time"].as_f64().unwrap();
            let y = f["position"]
                .as_str()
                .unwrap()
                .split(' ')
                .nth(1)
                .unwrap()
                .trim_end_matches("px")
                .parse::<f32>()
                .unwrap();
            assert_eq!(-normal_row(t) * 120., y, "phase {t}");
        }
    }
    #[test]
    fn source_obstacle_step_start_boundaries() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/glider-layout-reference.json")).unwrap();
        for row in data["obstacleFrames"].as_array().unwrap() {
            let t = row["time"].as_f64().unwrap();
            let y = row["position"]
                .as_str()
                .unwrap()
                .split(' ')
                .nth(1)
                .unwrap()
                .trim_end_matches("px")
                .parse::<f32>()
                .unwrap();
            let expected = if row["type"] == "frisbee" {
                if (t / 0.3).fract() < 0.5 {
                    -660.
                } else {
                    -600.
                }
            } else {
                -obstacle_row(t) * 120.
            };
            assert_eq!(expected, y, "{} phase {t}", row["type"]);
        }
    }
}
