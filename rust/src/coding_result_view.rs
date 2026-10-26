//! Code Boogie result rendering. This is a child of coding_view so the live
//! game and diagnostic use the same renderer and state.
use super::*;
use santa_tracker::codeboogie_result::{Action, Font as ResultFont, Layout, Spec, Text};
use santa_tracker::ui::Ease;

impl CodingView {
    fn result_text(
        &mut self,
        text: &Text,
        font: ResultFont,
        opacity: f64,
        shift: f64,
        color: &str,
    ) {
        if opacity <= 0. || text.lines.is_empty() {
            return;
        }
        let (family, weight) = match font {
            ResultFont::Lobster => ("Lobster", 400),
            ResultFont::Bold => ("Roboto", 800),
            ResultFont::Regular => ("Roboto", 400),
            ResultFont::Mono => ("Noto Sans Mono", 400),
        };
        let pad = text.size;
        let width = text.bounds[2] + pad * 2.;
        let height = text.bounds[3] + pad * 2.;
        let escape = |s: &str| {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        };
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}"><g font-family="{family}" font-weight="{weight}" font-size="{}" text-anchor="middle" fill="{color}">"#,
            text.size
        );
        for (i, line) in text.lines.iter().enumerate() {
            svg.push_str(&format!(
                r#"<text xml:space="preserve" x="{}" y="{}">{}</text>"#,
                width / 2.,
                pad + font.baseline(text.size, text.line_height) + i as f64 * text.line_height,
                escape(line)
            ));
        }
        svg.push_str("</g></svg>");
        if !self.images.contains_key(&svg) {
            self.images.insert(
                svg.clone(),
                svg_image(svg.as_bytes()).expect("result text SVG"),
            );
        }
        let image = &self.images[&svg];
        gl_use_material(&self.premult);
        image.draw(
            Rect::new(
                (text.bounds[0] - pad) as f32,
                (text.bounds[1] - pad + shift) as f32,
                width as f32,
                height as f32,
            ),
            Color::new(1., 1., 1., opacity as f32),
        );
        gl_use_default_material();
    }
    pub(super) fn dance_result_overlay(&mut self, input: bool) {
        let (w, h) = (screen_width(), screen_height());
        let top = (0..2)
            .rev()
            .find(|&i| self.result_panels[i].composed(self.elapsed));
        let mut chosen = None;
        for index in 0..2 {
            if !self.result_panels[index].composed(self.elapsed) {
                continue;
            }
            let spec = self.result_panels[index].spec.clone().unwrap();
            if self.result_layouts[index]
                .as_ref()
                .is_none_or(|(cw, ch, s, _)| *cw != w || *ch != h || s != &spec)
            {
                self.result_layouts[index] =
                    Some((w, h, spec.clone(), Layout::new(w as f64, h as f64, &spec)));
            }
            let layout = self.result_layouts[index].as_ref().unwrap().3.clone();
            let title_opacity = self.result_panels[index].title.value(self.elapsed)[0];
            let body_opacity = self.result_panels[index].body.value(self.elapsed)[0];
            let buttons_opacity = self.result_panels[index].buttons.value(self.elapsed)[0];
            let interactive = input && top == Some(index);
            // The original root transition declaration is invalid, so its
            // visibility/background change immediately. Only children fade.
            self.shapes.rounded(
                Rect::new(0., 0., w, h),
                0.,
                Color::new(63. / 255., 73. / 255., 163. / 255., 0.9),
                0.,
            );
            self.result_text(
                &layout.title,
                ResultFont::Lobster,
                title_opacity,
                -100. * (1. - title_opacity),
                "white",
            );
            if let Some(text) = &layout.details {
                self.result_text(text, ResultFont::Bold, body_opacity, 0., "white");
            }
            let rect = |r: [f64; 4]| Rect::new(r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32);
            if let Some(r) = layout.missing {
                // The original result symbol differs from the editor's full
                // repeat block. Preserve its 48px square inside the SVG box.
                if spec.missing == "controls_repeat" && r[3] > 0. {
                    let size = r[2].min(r[3]);
                    gl_use_material(&self.premult);
                    self.image("result-missing-controls_repeat").draw(
                        Rect::new(
                            (r[0] + (r[2] - size) / 2.) as f32,
                            r[1] as f32,
                            size as f32,
                            size as f32,
                        ),
                        Color::new(1., 1., 1., body_opacity as f32),
                    );
                    gl_use_default_material();
                }
            }
            if let Some(r) = layout.graphic {
                if r[3] > 0. {
                    let image = self.image(if spec.success {
                        "result-success"
                    } else {
                        "result-failure"
                    });
                    let scale = (r[2] / image.size.x as f64).min(r[3] / image.size.y as f64);
                    gl_use_material(&self.premult);
                    image.draw(
                        Rect::new(
                            (r[0] + (r[2] - image.size.x as f64 * scale) / 2.) as f32,
                            (r[1] + (r[3] - image.size.y as f64 * scale) / 2.) as f32,
                            image.size.x * scale as f32,
                            image.size.y * scale as f32,
                        ),
                        Color::new(1., 1., 1., body_opacity as f32),
                    );
                    gl_use_default_material();
                }
            }
            if let Some(text) = &layout.code_label {
                self.result_text(text, ResultFont::Bold, 1., 0., "white");
            }
            if let Some(r) = layout.code {
                self.shapes
                    .stroke(rect(r), 20., 3., Color::new(1., 1., 1., 0.14));
                let large = w >= 570. && h >= 500.;
                let (px, py, size) = if large {
                    (23., 19., 16.)
                } else {
                    (13., 9., 14.)
                };
                let inner = Rect::new(
                    r[0] as f32 + px,
                    r[1] as f32 + py,
                    (r[2] as f32 - 2. * px).max(0.),
                    (r[3] as f32 - 2. * py).max(0.),
                );
                let columns = (inner.w as f64 / ResultFont::Mono.width("M", size))
                    .floor()
                    .max(1.) as usize;
                let mut lines = Vec::new();
                for line in spec.code.split('\n') {
                    let chars: Vec<_> = line.chars().collect();
                    if chars.is_empty() {
                        lines.push(String::new());
                    } else {
                        for part in chars.chunks(columns) {
                            lines.push(part.iter().collect::<String>());
                        }
                    }
                }
                let total = lines.len() as f32 * 32.;
                if interactive && inner.contains(Vec2::from(mouse_position())) {
                    self.result_code_scroll[index] = (self.result_code_scroll[index]
                        - mouse_wheel().1 * 32.)
                        .clamp(0., (total - inner.h).max(0.));
                }
                let scroll = self.result_code_scroll[index].clamp(0., (total - inner.h).max(0.));
                unsafe {
                    get_internal_gl().quad_gl.scissor(Some((
                        inner.x.ceil() as i32,
                        inner.y.ceil() as i32,
                        inner.w.floor() as i32,
                        inner.h.floor() as i32,
                    )));
                }
                let advance = ResultFont::Mono.width("M", size);
                // Position each preformatted line independently to preserve
                // leading spaces. Soft-wrap break selection still needs a
                // wider original textarea comparison.
                for (i, line) in lines.iter().enumerate() {
                    let y = inner.y - scroll + i as f32 * 32.;
                    if y + 32. < inner.y || y > inner.bottom() {
                        continue;
                    }
                    let text = Text {
                        lines: vec![line.clone()],
                        bounds: [
                            inner.x as f64,
                            y as f64,
                            advance * line.chars().count() as f64,
                            32.,
                        ],
                        size,
                        line_height: 32.,
                    };
                    self.result_text(&text, ResultFont::Mono, 1., 0., "white");
                }
                unsafe {
                    get_internal_gl().quad_gl.scissor(None);
                }
            }
            // CSS opacity on the buttons container composites the complete
            // subtree once, including shadows and overlapping label glyphs.
            if self
                .result_buttons_target
                .as_ref()
                .is_none_or(|t| t.texture.width() != w || t.texture.height() != h)
            {
                self.result_buttons_target = Some(render_target(w as u32, h as u32));
            }
            push_camera_state();
            let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., w, h));
            camera.render_target = self.result_buttons_target.clone();
            set_camera(&camera);
            clear_background(BLANK);
            for button in &layout.buttons {
                let i = match button.action {
                    Action::Retry => 0,
                    Action::Share => 1,
                    Action::Continue => 2,
                };
                let mut r = rect(button.bounds);
                let old = self.result_presses[index][i].value(self.elapsed)[0] as f32;
                let visual = Rect::new(r.x - 3. * (1. - old), r.y - 6. * (1. - old), r.w, r.h);
                let hover = interactive && visual.contains(Vec2::from(mouse_position()));
                if hover && is_mouse_button_pressed(MouseButton::Left) {
                    self.result_captured[index][i] = true;
                }
                if interactive
                    && self.result_captured[index][i]
                    && hover
                    && is_mouse_button_released(MouseButton::Left)
                {
                    chosen = Some((index, button.action));
                }
                if !interactive || is_mouse_button_released(MouseButton::Left) {
                    self.result_captured[index][i] = false;
                }
                let down =
                    self.result_captured[index][i] && is_mouse_button_down(MouseButton::Left);
                self.result_presses[index][i].set(
                    self.elapsed,
                    [if down { 1. } else { 0. }],
                    if down { 0.05 } else { 0.25 },
                    Ease::InOut,
                );
                let p = self.result_presses[index][i].value(self.elapsed)[0] as f32;
                r.x -= 3. * (1. - p);
                r.y -= 6. * (1. - p);
                let shadow = Rect::new(r.x + 8. * (1. - p), r.y + 9. * (1. - p), r.w, r.h);
                self.shapes.rounded(
                    shadow,
                    layout.button_radius as f32,
                    Color::new(0., 0., 0., 0.16),
                    2. * p,
                );
                self.shapes.rounded(
                    Rect::new(r.x + 3. * (1. - p), r.y + 6. * (1. - p), r.w, r.h),
                    layout.button_radius as f32,
                    Color::from_hex(0x8f0093),
                    0.,
                );
                self.shapes.rounded(
                    r,
                    layout.button_radius as f32,
                    Color::from_hex(0xad00ad),
                    0.,
                );
                self.result_text(
                    &Text {
                        lines: vec![button.label.clone()],
                        bounds: [r.x as f64, r.y as f64, r.w as f64, r.h as f64],
                        size: layout.button_size,
                        line_height: r.h as f64,
                    },
                    ResultFont::Regular,
                    1.,
                    0.,
                    "white",
                );
            }
            if let Some(r) = layout.code_link {
                let hover = interactive && rect(r).contains(Vec2::from(mouse_position()));
                let text = Text {
                    lines: vec!["View code".into()],
                    bounds: [r[0], r[1] + 5., r[2], 18.],
                    size: 15.,
                    line_height: 18.,
                };
                self.result_text(&text, ResultFont::Bold, 1., 0., "#F5C637");
                if hover {
                    self.shapes.rounded(
                        Rect::new(r[0] as f32 + 5., (r[1] + 23.) as f32, r[2] as f32 - 10., 1.),
                        0.,
                        Color::from_hex(0xf5c637),
                        0.,
                    );
                }
                if hover && is_mouse_button_released(MouseButton::Left) {
                    self.result_panels[index].spec.as_mut().unwrap().show_code = true;
                }
            }
            pop_camera_state();
            gl_use_material(&self.premult);
            draw_texture_ex(
                &self.result_buttons_target.as_ref().unwrap().texture,
                0.,
                0.,
                Color::new(1., 1., 1., buttons_opacity as f32),
                DrawTextureParams {
                    flip_y: true,
                    ..Default::default()
                },
            );
            gl_use_default_material();
        }
        if let Some((index, action)) = chosen {
            self.result_panels[index].hide(self.elapsed);
            match action {
                Action::Retry => {
                    self.result = None;
                    self.dance_report_pending = false;
                }
                Action::Continue => {
                    if self.level().freestyle {
                        self.levels = self.dance_data.levels.clone();
                        self.set_level(0);
                    } else {
                        self.set_level(self.level_index + 1);
                    }
                }
                // Share is not yet supplied by live result specs. The layout
                // diagnostic measures its source box; host sharing remains
                // explicitly unfinished rather than installing a fake action.
                Action::Share => {}
            }
        }
    }
    #[allow(dead_code)]
    pub fn prepare_result_capture(&mut self, spec: Spec, time: f64) {
        let show_code = spec.show_code;
        let i = usize::from(spec.success);
        self.result_panels = std::array::from_fn(|_| Default::default());
        self.result_panels[i].show(0., spec);
        self.result_panels[i].spec.as_mut().unwrap().show_code = show_code;
        self.elapsed = time;
    }
    /// Headless input-independent integration check of the live Run/Finish
    /// path. This does not certify drag, pointer or keyboard handling.
    #[allow(dead_code)]
    pub fn prepare_result_flow_capture(&mut self) {
        self.chooser = false;
        self.stage = DanceStage::default();
        self.set_level(0);
        for _ in 0..6000 {
            self.update(1. / 60.);
            if !self.playing {
                break;
            }
        }
        assert!(!self.playing, "first lesson intro did not finish");
        let root = self.workspace.roots[0];
        let mut last = root;
        for step in self.level().steps.clone() {
            let id = self.workspace.add(Block::new(&format!("dance_{step}")));
            assert!(self.workspace.connect(id, last, false));
            last = id;
        }
        self.run();
        let submitted = self.dance_user_code.clone();
        assert_eq!(submitted, "pointLeft();\npointRight();");
        // The original editor remains editable while dancing. This extra
        // block must not alter the already-submitted result code or verdict.
        let extra = self.workspace.add(Block::new("dance_pointLeft"));
        assert!(self.workspace.connect(extra, last, false));
        for _ in 0..6000 {
            self.update(1. / 60.);
            if self.result_panels[1].visible {
                break;
            }
        }
        assert!(
            self.result_panels[1].visible,
            "successful first lesson did not report"
        );
        let spec = self.result_panels[1].spec.as_mut().unwrap();
        assert_eq!(spec.code, submitted);
        assert!(!spec.retry);
        assert_ne!(self.workspace.dance_user_code().unwrap(), submitted);
        spec.show_code = true;
        self.elapsed += 2.;
    }
    #[allow(dead_code)]
    pub fn draw_result_capture(&mut self) {
        clear_background(Color::from_hex(0x364189));
        self.dance_result_overlay(false);
    }
}
