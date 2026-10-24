//! Native original SVG assembly, control strip and photo capture.
use crate::{
    assets::Assets,
    ui_render::{Button, Shapes},
};
use macroquad::prelude::*;
use santa_tracker::elfmaker::{ElfMaker, CATEGORIES, MODES, SAVED};
use std::collections::HashMap;
pub fn raster(svg: &str, w: u32, h: u32) -> Image {
    raster_inner(svg, w, h, false)
}
pub fn raster_premult(svg: &str, w: u32, h: u32) -> Image {
    raster_inner(svg, w, h, true)
}
fn raster_inner(svg: &str, w: u32, h: u32, premult: bool) -> Image {
    let tree = resvg::usvg::Tree::from_str(svg, &Default::default()).expect("Source elf SVG");
    let mut p = resvg::tiny_skia::Pixmap::new(w.max(1), h.max(1)).unwrap();
    let size = tree.size();
    let sc = (w as f32 / size.width()).min(h as f32 / size.height());
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_row(
            sc,
            0.,
            0.,
            sc,
            (w as f32 - size.width() * sc) / 2.,
            (h as f32 - size.height() * sc) / 2.,
        ),
        &mut p.as_mut(),
    );
    let mut bytes = p.data().to_vec();
    if !premult {
        for p in bytes.chunks_exact_mut(4) {
            if p[3] > 0 {
                for i in 0..3 {
                    p[i] = ((p[i] as u32 * 255) / p[3] as u32).min(255) as u8;
                }
            }
        }
        santa_tracker::image_alpha::bleed_edges(&mut bytes, w as usize, h as usize);
    }
    Image {
        bytes,
        width: w as u16,
        height: h as u16,
    }
}
fn upload(svg: &str, w: u32, h: u32) -> Texture2D {
    let im = raster(svg, w, h);
    let t = Texture2D::from_image(&im);
    t.set_filter(FilterMode::Linear);
    t
}
fn image(t: &Texture2D, r: Rect) {
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
fn cover(t: &Texture2D, r: Rect) {
    let sc = (r.w / t.width()).max(r.h / t.height());
    let source = Rect::new(
        (t.width() - r.w / sc) / 2.,
        (t.height() - r.h / sc) / 2.,
        r.w / sc,
        r.h / sc,
    );
    draw_texture_ex(
        t,
        r.x,
        r.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(r.size()),
            source: Some(source),
            ..Default::default()
        },
    );
}
pub struct ElfMakerView {
    texture: Option<Texture2D>,
    previews: HashMap<String, Texture2D>,
    pub scroll: f32,
    pub category_scroll: f32,
    pub footer_scroll: f32,
    shapes: Shapes,
    pub photo: Option<Texture2D>,
    photo_at: f64,
    premult: Material,
    photo_shown: bool,
    buttons: [Button; 4],
    icons: HashMap<String, Texture2D>,
    last_svg: String,
    drag: Option<(Vec2, f32)>,
    last_time: f64,
    shadow: crate::elf_shadow::ElfShadow,
    panel_shadow: Texture2D,
}
impl ElfMakerView {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            texture: None,
            previews: HashMap::new(),
            scroll: 0.,
            category_scroll: 0.,
            footer_scroll: 0.,
            shapes: Shapes::new()?,
            photo: None,
            photo_at: -100.,
            premult: crate::ui_render::premultiplied_material()?,
            photo_shown: false,
            buttons: std::array::from_fn(|i| {
                Button::new(Color::from_hex([0xad00ad, 0xff3f00, 0x32a658, 0xff3f00][i]))
            }),
            icons: {
                let mut m = HashMap::new();
                for(n,path)in [("camera","M9 2 7.17 4H4c-1.1 0-2 .9-2 2v12c0 1.1 .9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2h-3.17L15 2H9zm3 15a5 5 0 1 1 0-10 5 5 0 0 1 0 10zm0-1.8a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4z"),("save","M17 3H5c-1.1 0-2 .9-2 2v14c0 1.1 .9 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zM12 19a3 3 0 1 1 0-6 3 3 0 0 1 0 6zM15 9H5V5h10v4z")]{m.insert(n.into(),upload(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\"><path fill=\"white\" d=\"{path}\"/></svg>"),96,96));}
                m
            },
            last_svg: String::new(),
            drag: None,
            last_time: 0.,
            shadow: crate::elf_shadow::ElfShadow::new()?,
            panel_shadow: Texture2D::from_file_with_format(
                include_bytes!("../assets/elfmaker-panel-shadow.png"),
                Some(ImageFormat::Png),
            ),
        })
    }
    pub fn reset(&mut self) {
        self.photo = None;
        self.photo_shown = false;
        self.scroll = 0.;
        self.footer_scroll = 0.;
    }
    pub fn draw(&mut self, a: &Assets, g: &mut ElfMaker, input: bool) {
        let (w, h) = (screen_width(), screen_height());
        let mobile = w < 768.;
        let fy = if mobile { h * 0.5 } else { h - 138. };
        let main = Rect::new(0., 0., w, fy);
        let bg = g.art["categories"]["backgrounds"][g.choices[6]]
            .as_str()
            .unwrap();
        cover(a.scene("elfmaker", bg.strip_prefix("img/").unwrap()), main);
        // #main paints its inset box-shadow above its background, below children.
        // The original CSS shadow is rasterized into a 96px nine-slice mask;
        // its fixed 36px edges preserve the blur at every panel size.
        let source = [0., 36., 60., 96.];
        let xs = [0., 36., w - 36., w];
        let ys = [0., 36., fy - 36., fy];
        for y in 0..3 {
            for x in 0..3 {
                draw_texture_ex(
                    &self.panel_shadow,
                    xs[x],
                    ys[y],
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(
                            source[x],
                            source[y],
                            source[x + 1] - source[x],
                            source[y + 1] - source[y],
                        )),
                        dest_size: Some(vec2(xs[x + 1] - xs[x], ys[y + 1] - ys[y])),
                        ..Default::default()
                    },
                );
            }
        }
        let eh = (fy - 32.).max(1.);
        let ew = eh * 400. / 560.;
        let er = Rect::new((w - ew) / 2., 16., ew, eh);
        let svg = g.svg("-40 -100 400 560", None);
        if svg != self.last_svg
            || self
                .texture
                .as_ref()
                .is_none_or(|t| (t.height() - eh).abs() > 1.)
        {
            let im = raster_premult(&svg, ew.ceil() as u32, eh.ceil() as u32);
            if let Some(t) = self
                .texture
                .as_ref()
                .filter(|t| t.width() == im.width as f32 && t.height() == im.height as f32)
            {
                t.update(&im)
            } else {
                self.texture = Some(Texture2D::from_image(&im));
            }
            self.last_svg = svg;
        }
        if let Some(t) = &self.texture {
            self.shadow.draw(t, er, &self.premult);
            gl_use_material(&self.premult);
            image(t, er);
            gl_use_default_material();
        }
        let m = Vec2::from(mouse_position());
        let mut click = input
            && is_mouse_button_released(MouseButton::Left)
            && !crate::chrome::over_controls(m, 2);
        if click && er.contains(m) && self.photo.as_ref().is_none_or(|_| !self.photo_shown) {
            g.poke();
        }
        let dt = (g.time - self.last_time).max(0.) as f32;
        self.last_time = g.time;
        let size = if mobile { 55. } else { 72. };
        let left = if mobile { 32. } else { 32. };
        let top = fy / 2. - size - 12.;
        let right = w - size - 32.;
        let tray = a.scene("elfmaker", "tray_lg.svg");
        let tr = Rect::new(0., top - 32., size + 64., 2. * size + 88.);
        let ts = (tr.w / tray.width()).min(tr.h / tray.height());
        image(
            tray,
            Rect::new(
                tr.center().x - tray.width() * ts / 2.,
                tr.center().y - tray.height() * ts / 2.,
                tray.width() * ts,
                tray.height() * ts,
            ),
        );
        for (i, (x, y, key, _color)) in [
            (left, top, "shuffle.svg", Color::from_hex(0xad00ad)),
            (left, top + size + 24., "camera", Color::from_hex(0xff3333)),
            (right, top, "share.svg", Color::from_hex(0x32a658)),
            (right, top + size + 24., "save", Color::from_hex(0xff3333)),
        ]
        .into_iter()
        .enumerate()
        {
            if i == 3 && !self.photo_shown {
                continue;
            }
            let r = Rect::new(x, y, size, size);
            let _interaction = self.buttons[i].update(r, g.time, input, false, false);
            if _interaction.entered {
                g.sounds.push("generic_button_over");
            }
            if _interaction.pressed {
                g.sounds.push("generic_button_click");
            }
            let delta = self.buttons[i].draw(&self.shapes, r, g.time, false, 1.);
            let t = if key.ends_with("svg") {
                a.texture(&format!("scenes/_shared/img/button/{key}"))
            } else {
                &self.icons[key]
            };
            let icon = if mobile { 32. } else { 48. };
            if key.ends_with("svg") {
                gl_use_material(&self.premult);
            }
            image(
                t,
                Rect::new(
                    x + (size - icon) / 2. + delta.x,
                    y + (size - icon) / 2. + delta.y,
                    icon,
                    icon,
                ),
            );
            gl_use_default_material();
            if _interaction.clicked {
                match i {
                    0 => {
                        g.randomize();
                        self.previews.clear()
                    }
                    1 => self.capture(a, g),
                    2 => {
                        if !self.photo_shown {
                            self.capture(a, g);
                            self.photo_at = -100.;
                        }
                        self.export(g);
                    }
                    3 => {
                        self.export(g);
                        self.photo_shown = false
                    }
                    _ => {}
                }
            }
        }
        draw_rectangle(0., fy, w, h - fy, WHITE);
        let mut cy = fy - self.footer_scroll;
        let catw = if mobile { w } else { 256. };
        draw_rectangle(
            0.,
            cy,
            catw,
            if mobile { 71. } else { 138. },
            Color::from_hex(0xf0f0f0),
        );
        for (i, c) in CATEGORIES.iter().enumerate() {
            let (x, y) = if mobile {
                (8. + i as f32 * 59. - self.category_scroll, cy + 8.)
            } else {
                (12. + (i % 4) as f32 * 59., cy + 12. + (i / 4) as f32 * 59.)
            };
            let r = Rect::new(x, y, 55., 55.);
            if g.category == i {
                self.shapes
                    .circle(r.center(), 27.5, Color::from_hex(0xf8ce3c), 0.);
            }
            let t = a.scene("elfmaker", &format!("btn_{c}.svg"));
            if x >= -55. && x < catw {
                image(t, r)
            }
            if click && r.contains(m) && m.x < catw && m.y >= fy {
                g.select_category(i);
                self.scroll = 0.;
                self.previews.clear();
            }
        }
        let mut px = catw;
        let mut py = cy + 6.;
        let mut ph = 126.;
        if mobile {
            cy += 71.;
            px = 0.;
            py = cy + 6.;
            ph = 96.;
        }
        if let Some(p) = g.color_property() {
            let opts = g.art["options"][MODES[p]].as_array().unwrap().clone();
            let half = opts.len().div_ceil(2);
            let cw = if mobile { w } else { (half * 59 + 20) as f32 };
            for (i, col) in opts.iter().enumerate() {
                let x = if mobile {
                    (w - opts.len() as f32 * 59.) / 2. + i as f32 * 59. + 2.
                } else {
                    px + 12.
                        + if i >= half {
                            (half - (opts.len() - half)) as f32 * 29.5 + (i - half) as f32 * 59.
                        } else {
                            i as f32 * 59.
                        }
                };
                let y = if mobile {
                    cy + 8.
                } else {
                    cy + 12. + (i / half) as f32 * 59.
                };
                let r = Rect::new(x, y, 55., 55.);
                let hex = g.art["colors"][col.as_str().unwrap()][0].as_str().unwrap();
                let color = Color::from_hex({
                    let h = hex.trim_start_matches('#');
                    let h = if h.len() == 3 {
                        h.chars().flat_map(|c| [c, c]).collect::<String>()
                    } else {
                        h.into()
                    };
                    u32::from_str_radix(&h, 16).unwrap_or(0)
                });
                self.shapes.circle(r.center(), 27.5, color, 0.);
                if g.colors[p] == col.as_str().unwrap() {
                    draw_line(x + 17., y + 28., x + 25., y + 36., 3., WHITE);
                    draw_line(x + 25., y + 36., x + 40., y + 19., 3., WHITE);
                }
                if click && r.contains(m) && m.y >= fy {
                    g.choose_color(i);
                    self.previews.clear();
                }
            }
            if mobile {
                cy += 71.;
                py = cy + 2.
            } else {
                px += cw;
            }
        }
        let count = g.art["categories"][CATEGORIES[g.category]]
            .as_array()
            .map_or(0, Vec::len);
        let saved = SAVED.iter().position(|s| *s == CATEGORIES[g.category]);
        let available = (w - px - 8.).max(1.);
        let max_scroll = (count as f32 * 84. - available).max(0.);
        let show_arrows = !mobile && count as f32 * 84. > available + 42.;
        let arrow_left = Rect::new(px + 4., fy + 4., 42., 130.);
        let arrow_right = Rect::new(w - 46., fy + 4., 42., 130.);
        let over_arrow = show_arrows
            && ((self.scroll > 0. && arrow_left.contains(m))
                || (self.scroll < max_scroll && arrow_right.contains(m)));
        if input && !mobile {
            if is_mouse_button_pressed(MouseButton::Left) && m.x >= px && m.y >= fy && !over_arrow {
                self.drag = Some((m, self.scroll));
            }
            if let Some((start, scroll)) = self.drag {
                self.scroll = (scroll + start.x - m.x).clamp(0., max_scroll);
                if is_mouse_button_released(MouseButton::Left) {
                    if (m.x - start.x).abs() > 16. {
                        click = false;
                    }
                    self.drag = None;
                }
            }
            if is_mouse_button_down(MouseButton::Left) && over_arrow {
                self.scroll = (self.scroll
                    + if arrow_left.contains(m) {
                        -500. * dt
                    } else {
                        500. * dt
                    })
                .clamp(0., max_scroll);
            }
        }
        if over_arrow {
            click = false;
        }

        let cols = ((w - 8.) / 64.).floor().max(1.) as usize;
        let previeww = if mobile {
            (w - 8.) / cols as f32 - 4.
        } else {
            80.
        };
        for i in 0..count {
            let r = Rect::new(
                if mobile {
                    6. + (i % cols) as f32 * (previeww + 4.)
                } else {
                    px + 6. + i as f32 * 84. - self.scroll
                },
                if mobile {
                    py + (i / cols) as f32 * 100.
                } else {
                    py
                },
                previeww,
                ph,
            );
            if r.right() < px || r.x > w || r.bottom() < fy || r.y > h {
                continue;
            }
            self.shapes.rounded(
                r,
                12.,
                Color::from_hex(if saved.is_some_and(|k| g.choices[k] == i) {
                    0x2cc4d6
                } else {
                    0xf3f3f3
                }),
                0.,
            );
            let inner = Rect::new(r.x + 2., r.y + 2., r.w - 4., r.h - 4.);
            self.shapes
                .rounded(inner, 10., Color::from_hex(0xf3f3f3), 0.);
            if g.category == 7 {
                let s = g.art["categories"]["backgrounds"][i].as_str().unwrap();
                cover(a.scene("elfmaker", s.strip_prefix("img/").unwrap()), inner);
            } else {
                let key = format!("{}-{i}-{}", g.category, g.save());
                if !self.previews.contains_key(&key) {
                    let mut choices = g.choices;
                    if let Some(k) = saved {
                        choices[k] = i;
                    }
                    let svg = if g.category == 0 {
                        g.svg("-60 -30 440 660", Some(choices))
                    } else {
                        let indent = if g.category == 5 { 0 } else { 80 };
                        let part = g.art["categories"][CATEGORIES[g.category]][i]
                            .as_str()
                            .unwrap_or("");
                        let (head, pre, post) = (
                            g.art["head"].as_str().unwrap(),
                            if g.category == 5 { part } else { "" },
                            if g.category == 5 { "" } else { part },
                        );
                        format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"64 {indent} 192 {}\">{}<style>{}{}</style><g transform=\"translate(55)\"><g class=\"{}\">{pre}</g>{head}<g class=\"{}\">{post}</g></g></svg>",345-indent,g.art["baseSvgDefs"].as_str().unwrap(),g.art["baseSvgStyle"].as_str().unwrap(),g.style(),CATEGORIES[g.category],CATEGORIES[g.category])
                    };
                    self.previews
                        .insert(key.clone(), upload(&svg, 120, ph.max(140.) as u32));
                }
                let t = &self.previews[&key];
                let rr = Rect::new(inner.center().x - 60., inner.y, 120., inner.h.max(140.));
                unsafe {
                    get_internal_gl().quad_gl.scissor(Some((
                        inner.x.max(px) as i32,
                        inner.y.max(fy) as i32,
                        (inner.w.min(w - inner.x)).max(0.) as i32,
                        (inner.h.min(h - inner.y)).max(0.) as i32,
                    )));
                }
                image(t, rr);
                unsafe {
                    get_internal_gl().quad_gl.scissor(None);
                }
            }
            if click && r.contains(m) && m.y >= fy && m.x >= px {
                g.choose(i);
            }
        }
        if show_arrows {
            for (r, left, visible) in [
                (arrow_left, true, self.scroll > 0.),
                (arrow_right, false, self.scroll < max_scroll),
            ] {
                if !visible {
                    continue;
                }
                draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.5, 0.5, 0.5, 0.8));
                let x = r.center().x;
                let y = r.center().y;
                draw_circle_lines(x, y, 16., 1.5, WHITE);
                let d = if left { 1. } else { -1. };
                draw_line(x + 3. * d, y - 6., x - 3. * d, y, 1.5, WHITE);
                draw_line(x - 3. * d, y, x + 3. * d, y + 6., 1.5, WHITE);
            }
        }
        if input {
            let wheel = mouse_wheel();
            if m.y >= fy {
                if mobile {
                    if m.y < fy + 71. {
                        self.category_scroll =
                            (self.category_scroll - wheel.1 * 25.).clamp(0., (472. - w).max(0.))
                    } else {
                        self.footer_scroll = (self.footer_scroll - wheel.1 * 25.).max(0.)
                    }
                } else {
                    self.scroll = (self.scroll - wheel.1 * 25.)
                        .clamp(0., (count as f32 * 84. - available).max(0.));
                }
            }
        }
        if self.photo_shown && g.time - self.photo_at >= 0.15 {
            if let Some(t) = &self.photo {
                let elapsed = (g.time - self.photo_at - 0.15) as f32;
                let p = santa_tracker::ui::Ease::Default.at((elapsed / 1.2) as f64) as f32;
                let sw = 224. * p;
                let sh = 350. * p;
                let center = vec2(
                    w - if mobile { 50. } else { 80. } - 168.,
                    if mobile { h * 0.3 } else { h * 0.5 - 75. },
                );
                let r = Rect::new(center.x - sw / 2., center.y - sh / 2., sw, sh);
                draw_rectangle(r.x - 6., r.y - 6., r.w + 12., r.h + 12., WHITE);
                draw_texture_ex(
                    t,
                    r.x,
                    r.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(r.size()),
                        rotation: ((g.time * std::f64::consts::PI / 2.).cos() * 2.).to_radians()
                            as f32,
                        ..Default::default()
                    },
                );
                if click && r.contains(m) {
                    self.photo_shown = false;
                }
            }
        }
        let flash = (g.time - self.photo_at) as f32;
        if (0.0..0.3).contains(&flash) {
            let alpha = if flash < 0.15 {
                flash / 0.15
            } else {
                (0.3 - flash) / 0.15
            };
            draw_rectangle(0., 0., w, h, Color::new(1., 1., 1., alpha));
        }
    }
    fn capture(&mut self, a: &Assets, g: &mut ElfMaker) {
        let target = render_target(640, 1000);
        set_camera(&Camera2D {
            render_target: Some(target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0., 0., 640., 1000.))
        });
        clear_background(WHITE);
        let bg = g.art["categories"]["backgrounds"][g.choices[6]]
            .as_str()
            .unwrap();
        let t = a.scene("elfmaker", bg.strip_prefix("img/").unwrap());
        let source = Rect::new(
            (t.width() - 640. / 1.2) / 2.,
            (t.height() - 1000. / 1.2) / 2.,
            640. / 1.2,
            1000. / 1.2,
        );
        draw_texture_ex(
            t,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(640., 1000.)),
                source: Some(source),
                ..Default::default()
            },
        );
        let elf = upload(&g.svg("-40 -100 400 560", None), 640, 1000);
        image(&elf, Rect::new(0., 0., 640., 1000.));
        set_default_camera();
        let im = target.texture.get_texture_data();
        self.photo = Some(Texture2D::from_image(&im));
        self.photo_at = g.time;
        self.photo_shown = true;
        g.sounds.push("elfmaker_photo");
    }
    fn export(&self, g: &ElfMaker) {
        if let Some(t) = &self.photo {
            let _ = crate::capture::save_png(
                &t.get_texture_data(),
                std::path::Path::new("elfmaker.png"),
            );
            let _ = std::fs::write("elfmaker.elf", g.save());
        }
    }
}

#[allow(dead_code)] // CPU reference filter retained for before/after diagnostics.
pub fn shadow(svg: &str, unit: f32) -> String {
    let start = svg.find('>').unwrap() + 1;
    format!("{}<defs><filter id=\"native-elf-shadow\" x=\"-50%\" y=\"-50%\" width=\"200%\" height=\"200%\"><feDropShadow dx=\"{}\" dy=\"{}\" stdDeviation=\"{}\" flood-opacity=\"0.125\"/></filter></defs><g filter=\"url(#native-elf-shadow)\">{}</g></svg>",&svg[..start],4.*unit,4.*unit,2.*unit,svg[start..].strip_suffix("</svg>").unwrap())
}
