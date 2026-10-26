//! Native editor, puzzle/maze and dance views. Browser code is reference-only.
use crate::{
    assets::Assets,
    ui_render::{premultiplied_material, Shapes},
};
use macroquad::prelude::*;
use santa_tracker::codeboogie_stage::{DanceStage, Event as DanceEvent, SpriteClock};
use santa_tracker::coding::{self, Block, DanceItem, LabRun, Level, Point, ResultInfo, Workspace};
use std::collections::HashMap;

struct Drag {
    id: usize,
    offset: Vec2,
}
struct Hit {
    id: usize,
    rect: Rect,
}
struct Slot {
    parent: usize,
    body: bool,
    point: Vec2,
}
#[derive(Clone)]
struct Image {
    texture: Texture2D,
    size: Vec2,
    offset: Vec2,
    paint_size: Vec2,
}
impl Image {
    fn draw(&self, r: Rect, color: Color) {
        draw_texture_ex(
            &self.texture,
            r.x + self.offset.x * r.w / self.size.x,
            r.y + self.offset.y * r.h / self.size.y,
            color,
            DrawTextureParams {
                dest_size: Some(self.paint_size * r.size() / self.size),
                ..Default::default()
            },
        );
    }
}
fn svg_image(bytes: &[u8]) -> Result<Image, String> {
    let mut opts = resvg::usvg::Options::default();
    opts.fontdb_mut()
        .load_font_data(include_bytes!("../assets/fonts/Roboto-SemiBold.ttf").to_vec());
    for data in [
        include_bytes!("../assets/fonts/Roboto-Regular.ttf").as_slice(),
        include_bytes!("../assets/fonts/Roboto-ExtraBold.ttf").as_slice(),
        include_bytes!("../assets/fonts/Lobster-Regular.ttf").as_slice(),
    ] {
        opts.fontdb_mut().load_font_data(data.to_vec());
    }
    let tree = resvg::usvg::Tree::from_data(bytes, &opts).map_err(|e| e.to_string())?;
    let size = vec2(tree.size().width(), tree.size().height());
    let mut pix =
        resvg::tiny_skia::Pixmap::new((size.x * 2.).ceil() as u32, (size.y * 2.).ceil() as u32)
            .ok_or("SVG allocation")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(2., 2.),
        &mut pix.as_mut(),
    );
    let texture = Texture2D::from_rgba8(pix.width() as u16, pix.height() as u16, pix.data());
    texture.set_filter(FilterMode::Linear);
    Ok(Image {
        texture,
        size,
        offset: Vec2::ZERO,
        paint_size: size,
    })
}
fn embedded(scene: &str, name: &str) -> Option<&'static [u8]> {
    match (scene, name) {
        ("codelab", "block-maze_moveNorth_mini") => Some(include_bytes!(
            "../assets/codelab/block-maze_moveNorth_mini.svg"
        )),
        ("codelab", "block-maze_moveSouth_mini") => Some(include_bytes!(
            "../assets/codelab/block-maze_moveSouth_mini.svg"
        )),
        ("codelab", "block-maze_moveEast_mini") => Some(include_bytes!(
            "../assets/codelab/block-maze_moveEast_mini.svg"
        )),
        ("codelab", "block-maze_moveWest_mini") => Some(include_bytes!(
            "../assets/codelab/block-maze_moveWest_mini.svg"
        )),
        ("codelab", "block-controls_jump_mini") => Some(include_bytes!(
            "../assets/codelab/block-controls_jump_mini.svg"
        )),
        ("codelab", "block-controls_repeat_mini") => Some(include_bytes!(
            "../assets/codelab/block-controls_repeat_mini.svg"
        )),
        ("codeboogie", "block-dance_pointLeft_mini") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_pointLeft_mini.svg"
        )),
        ("codeboogie", "block-dance_pointRight_mini") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_pointRight_mini.svg"
        )),
        ("codeboogie", "block-dance_stepLeft_mini") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_stepLeft_mini.svg"
        )),
        ("codeboogie", "block-dance_stepRight_mini") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_stepRight_mini.svg"
        )),
        ("codeboogie", "block-dance_jump_mini") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_jump_mini.svg"
        )),
        ("codeboogie", "block-dance_splits_mini") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_splits_mini.svg"
        )),
        ("codeboogie", "block-dance_hip_mini") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_hip_mini.svg"
        )),
        ("codeboogie", "block-controls_repeat_mini") => Some(include_bytes!(
            "../assets/codeboogie/block-controls_repeat_mini.svg"
        )),
        ("codelab", "trash") => Some(include_bytes!("../assets/codelab/trash.svg")),
        ("codelab", "puzzle-ghost-1") => {
            Some(include_bytes!("../assets/codelab/puzzle-ghost-1.svg"))
        }
        ("codelab", "puzzle-ghost-2") => {
            Some(include_bytes!("../assets/codelab/puzzle-ghost-2.svg"))
        }
        ("codelab", "maze-tree") => Some(include_bytes!("../assets/codelab/maze-tree.svg")),
        ("codelab", "maze-present") => Some(include_bytes!("../assets/codelab/maze-present.svg")),
        ("codelab", "maze-yeti") => Some(include_bytes!("../assets/codelab/maze-yeti.svg")),
        ("codelab", "maze-sled") => Some(include_bytes!("../assets/codelab/maze-sled.svg")),
        ("codelab", "maze-arrow") => Some(include_bytes!("../assets/codelab/maze-arrow.svg")),
        ("codelab", "player-walk") => Some(include_bytes!("../assets/codelab/player-walk.svg")),
        ("codelab", "player-lost") => Some(include_bytes!("../assets/codelab/player-lost.svg")),
        ("codelab", "player-victory") => {
            Some(include_bytes!("../assets/codelab/player-victory.svg"))
        }
        ("codelab", "result-trees") => Some(include_bytes!("../assets/codelab/result-trees.svg")),
        ("codelab", "result-failure") => {
            Some(include_bytes!("../assets/codelab/result-failure.svg"))
        }
        ("codelab", "result-success") => {
            Some(include_bytes!("../assets/codelab/result-success.svg"))
        }
        ("codelab", "result-final") => Some(include_bytes!("../assets/codelab/result-final.svg")),
        ("codelab", "result-puzzle") => Some(include_bytes!("../assets/codelab/result-puzzle.svg")),
        ("codelab", "block-when_run") => {
            Some(include_bytes!("../assets/codelab/block-when_run.svg"))
        }
        ("codelab", "block-maze_moveNorth") => {
            Some(include_bytes!("../assets/codelab/block-maze_moveNorth.svg"))
        }
        ("codelab", "block-maze_moveSouth") => {
            Some(include_bytes!("../assets/codelab/block-maze_moveSouth.svg"))
        }
        ("codelab", "block-maze_moveEast") => {
            Some(include_bytes!("../assets/codelab/block-maze_moveEast.svg"))
        }
        ("codelab", "block-maze_moveWest") => {
            Some(include_bytes!("../assets/codelab/block-maze_moveWest.svg"))
        }
        ("codelab", "block-controls_repeat") => Some(include_bytes!(
            "../assets/codelab/block-controls_repeat.svg"
        )),
        ("codelab", "block-controls_jump") => {
            Some(include_bytes!("../assets/codelab/block-controls_jump.svg"))
        }
        ("codelab", "block-puzzle_0A") => {
            Some(include_bytes!("../assets/codelab/block-puzzle_0A.svg"))
        }
        ("codelab", "block-puzzle_0B") => {
            Some(include_bytes!("../assets/codelab/block-puzzle_0B.svg"))
        }
        ("codelab", "block-puzzle_1A") => {
            Some(include_bytes!("../assets/codelab/block-puzzle_1A.svg"))
        }
        ("codelab", "block-puzzle_1B") => {
            Some(include_bytes!("../assets/codelab/block-puzzle_1B.svg"))
        }
        ("codelab", "block-puzzle_1C") => {
            Some(include_bytes!("../assets/codelab/block-puzzle_1C.svg"))
        }
        ("codeboogie", "result-success") => {
            Some(include_bytes!("../assets/codeboogie/result-success.svg"))
        }
        ("codeboogie", "result-failure") => {
            Some(include_bytes!("../assets/codeboogie/result-failure.svg"))
        }
        ("codeboogie", "block-when_run") => {
            Some(include_bytes!("../assets/codeboogie/block-when_run.svg"))
        }
        ("codeboogie", "block-dance_pointLeft") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_pointLeft.svg"
        )),
        ("codeboogie", "block-dance_pointRight") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_pointRight.svg"
        )),
        ("codeboogie", "block-dance_stepLeft") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_stepLeft.svg"
        )),
        ("codeboogie", "block-dance_stepRight") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_stepRight.svg"
        )),
        ("codeboogie", "block-dance_jump") => {
            Some(include_bytes!("../assets/codeboogie/block-dance_jump.svg"))
        }
        ("codeboogie", "block-dance_splits") => Some(include_bytes!(
            "../assets/codeboogie/block-dance_splits.svg"
        )),
        ("codeboogie", "block-dance_hip") => {
            Some(include_bytes!("../assets/codeboogie/block-dance_hip.svg"))
        }
        ("codeboogie", "block-controls_repeat") => Some(include_bytes!(
            "../assets/codeboogie/block-controls_repeat.svg"
        )),
        _ => None,
    }
}

pub struct CodingView {
    pub dance: bool,
    pub level_index: usize,
    pub chooser: bool,
    pub finished: bool,
    pub sounds: Vec<String>,
    pub track_events: Vec<usize>,
    lab: coding::LabData,
    dance_data: coding::DanceData,
    levels: Vec<Level>,
    workspace: Workspace,
    images: HashMap<String, Image>,
    shapes: Shapes,
    premult: Material,
    font: Font,
    lobster: Font,
    messages: HashMap<String, String>,
    drag: Option<Drag>,
    hits: Vec<Hit>,
    slots: Vec<Slot>,
    dropdown: Option<(usize, String, Rect)>,
    pub result: Option<ResultInfo>,
    result_at: f64,
    dance_user_code: String,
    show_code: bool,
    position: Point,
    direction: i32,
    start_direction: i32,
    run: Option<LabRun>,
    run_time: f64,
    last_motion: usize,
    stage: DanceStage,
    dance_report_pending: bool,
    playing: bool,
    watching: bool,
    highlight: Option<usize>,
    actions: Vec<coding::Action>,
    last_time: Option<f64>,
    elapsed: f64,
    portrait_open: bool,
    entered_at: f64,
    portrait_hide_at: Vec<f64>,
    chooser_hover: [bool; 2],
    chooser_hover_at: [f64; 2],
    chooser_art_fade: [santa_tracker::ui::Transition<1>; 2],
    chooser_go: [santa_tracker::ui::Transition<2>; 2],
}
impl CodingView {
    pub fn new(dance: bool) -> Result<Self, String> {
        let lab = coding::lab_data();
        let dd = coding::dance_data();
        let levels = if dance {
            dd.levels.clone()
        } else {
            lab.levels.clone()
        };
        let mut s = Self {
            dance,
            level_index: 0,
            chooser: dance,
            finished: false,
            sounds: vec![],
            track_events: vec![],
            lab,
            dance_data: dd,
            levels,
            workspace: Workspace::default(),
            images: HashMap::new(),
            shapes: Shapes::new()?,
            premult: premultiplied_material()?,
            font: load_ttf_font_from_bytes(include_bytes!("../assets/fonts/Roboto-SemiBold.ttf"))
                .map_err(|e| e.to_string())?,
            lobster: load_ttf_font_from_bytes(include_bytes!(
                "../assets/fonts/Lobster-Regular.ttf"
            ))
            .map_err(|e| e.to_string())?,
            messages: serde_json::from_str(include_str!("../data/coding-messages.json")).unwrap(),
            drag: None,
            hits: vec![],
            slots: vec![],
            dropdown: None,
            result: None,
            result_at: 0.,
            dance_user_code: String::new(),
            show_code: false,
            position: Point { x: 0, y: 0 },
            direction: 0,
            start_direction: 0,
            run: None,
            run_time: 0.,
            last_motion: usize::MAX,
            stage: DanceStage::default(),
            dance_report_pending: false,
            playing: false,
            watching: false,
            highlight: None,
            actions: vec![],
            last_time: None,
            elapsed: 0.,
            portrait_open: true,
            entered_at: 0.,
            portrait_hide_at: Vec::new(),
            chooser_hover: [false; 2],
            chooser_hover_at: [0.; 2],
            chooser_art_fade: std::array::from_fn(|_| santa_tracker::ui::Transition::new([1.])),
            chooser_go: std::array::from_fn(|_| santa_tracker::ui::Transition::new([0., -43.8])),
        };
        s.set_level(0);
        if dance {
            s.track_events.clear();
            s.stage = DanceStage::default();
            s.playing = false;
        }
        Ok(s)
    }
    pub fn reset(&mut self) {
        self.levels = if self.dance {
            self.dance_data.levels.clone()
        } else {
            self.lab.levels.clone()
        };
        self.set_level(0);
        self.chooser = self.dance;
        self.chooser_hover = [false; 2];
        self.chooser_art_fade = std::array::from_fn(|_| santa_tracker::ui::Transition::new([1.]));
        self.chooser_go = std::array::from_fn(|_| santa_tracker::ui::Transition::new([0., -43.8]));
        if self.dance {
            self.track_events.clear();
            self.stage = DanceStage::default();
            self.dance_report_pending = false;
            self.playing = false;
        }
    }
    pub fn level(&self) -> &Level {
        &self.levels[self.level_index]
    }
    fn msg(&self, key: &str) -> String {
        self.messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_owned())
    }
    fn image(&mut self, name: &str) -> Image {
        let key = format!("{}:{name}", self.dance);
        if !self.images.contains_key(&key) {
            let scene = if self.dance
                && (name.starts_with("block-")
                    || name == "result-success"
                    || name == "result-failure")
            {
                "codeboogie"
            } else {
                "codelab"
            };
            let data = embedded(scene, name).expect("Extracted original SVG");
            self.images
                .insert(key.clone(), svg_image(data).expect("Original SVG decoding"));
        }
        self.images[&key].clone()
    }
    fn block_image(&mut self, b: &Block) -> Image {
        let key = format!("{}:{:?}", b.kind, b.fields);
        if let Some(t) = self.images.get(&key) {
            return t.clone();
        }
        let scene = if self.dance { "codeboogie" } else { "codelab" };
        let bytes = embedded(scene, &format!("block-{}", b.kind)).unwrap();
        let mut source = String::from_utf8_lossy(bytes).into_owned();
        if b.kind == "controls_repeat" {
            source = source.replace(">2 ▾<", &format!(">{} ▾<", b.number("TIMES")));
        }
        if b.kind == "controls_jump" {
            source = source
                .replace(">1 ▾<", &format!(">{} ▾<", b.number("LENGTH")))
                .replace(
                    ">WEST ▾<",
                    &format!(
                        ">{} ▾<",
                        b.fields
                            .get("DIRECTION")
                            .unwrap_or(&"west".into())
                            .to_uppercase()
                    ),
                );
        }
        let mut t = svg_image(source.as_bytes()).unwrap();
        let geometry: serde_json::Value = serde_json::from_str(if self.dance {
            include_str!("../data/codeboogie-block-geometry.json")
        } else {
            include_str!("../data/codelab-block-geometry.json")
        })
        .unwrap();
        let g = &geometry[&b.kind];
        t.size = vec2(
            g["width"].as_f64().unwrap() as f32,
            g["height"].as_f64().unwrap() as f32,
        );
        t.offset = vec2(
            g["bounds"]["x"].as_f64().unwrap() as f32,
            g["bounds"]["y"].as_f64().unwrap() as f32,
        );

        self.images.insert(key, t.clone());
        t
    }
    fn set_level(&mut self, i: usize) {
        if i >= self.levels.len() {
            self.finished = true;
            return;
        }
        self.level_index = i;
        self.finished = false;
        let l = self.level().clone();
        self.workspace = Workspace::level(&l);
        self.result = None;
        self.show_code = false;
        self.run = None;
        self.drag = None;
        self.dropdown = None;
        self.highlight = None;
        self.position = Point {
            x: l.player_x,
            y: l.player_y,
        };
        self.start_direction = self.direction;
        self.portrait_open = true;
        self.entered_at = self.elapsed;
        if !self.dance {
            self.portrait_hide_at.push(self.elapsed + 3.);
        }
        self.playing = false;
        self.dance_report_pending = false;
        if self.dance {
            self.stage.now = self.elapsed;
            self.stage.set_level(&l);
            if !l.freestyle {
                let (_, mut q) = coding::dance_queue(&l, &[]);
                for _ in 1..if l.longer_intro { 4 } else { 2 } {
                    q.insert(
                        0,
                        DanceItem {
                            teacher_step: "idle".into(),
                            player_step: "idle".into(),
                            tile: None,
                            block_id: None,
                            title: "codeboogie-watch-closely".into(),
                            is_countdown: false,
                            is_intro: true,
                        },
                    );
                }
                self.stage
                    .start(q, Some("codeboogie-your-turn".into()), true, false);
                self.dance_report_pending = true;
                self.playing = true;
                self.watching = true;
            }
        }
        self.flush_dance_events();
    }
    fn show_result(&mut self, r: ResultInfo) {
        self.sounds.push(
            if self.dance {
                if r.success {
                    "cb_win"
                } else {
                    "cb_fail"
                }
            } else if r.success {
                "computer_success"
            } else {
                "computer_fail"
            }
            .into(),
        );
        self.result = Some(r);
        self.result_at = self.elapsed;
        self.show_code = false;
        self.highlight = None;
    }
    fn run(&mut self) {
        if (!self.dance && self.playing) || self.run.is_some() {
            return;
        }
        self.dropdown = None;
        self.dance_report_pending = false;
        self.dance_user_code.clear();
        let l = self.level().clone();
        self.actions = match self.workspace.actions() {
            Ok(actions) => actions,
            Err(message) => {
                self.show_result(ResultInfo {
                    message,
                    retry: true,
                    skip: true,
                    ..Default::default()
                });
                return;
            }
        };
        if self.dance {
            let validation = coding::evaluate(&l, &self.workspace, false, true);
            if validation.skip {
                self.show_result(validation);
                return;
            }
            let steps: Vec<_> = self
                .actions
                .iter()
                .filter_map(|a| a.kind.strip_prefix("dance_").map(str::to_string))
                .collect();
            if l.freestyle && steps.len() > 32 {
                self.show_result(ResultInfo {
                    message: "codeboogie-result-too-many-steps".into(),
                    retry: true,
                    skip: true,
                    ..Default::default()
                });
                return;
            }
            // processResult snapshots getUserCode before playback. Editing
            // blocks during a dance must not alter the result's code text.
            self.dance_user_code = match self.workspace.dance_user_code() {
                Ok(code) => code,
                Err(message) => {
                    self.show_result(ResultInfo {
                        message,
                        retry: true,
                        skip: true,
                        ..Default::default()
                    });
                    return;
                }
            };
            let (status, queue) = coding::dance_queue(&l, &steps);
            if queue.is_empty() && l.freestyle {
                self.result = None;
                self.stage.start(queue, None, false, true);
                self.playing = false;
                return;
            }
            let mut r = coding::evaluate(&l, &self.workspace, status == "SUCCESS", true);
            r.status = status;
            r.queue = queue.clone();
            if r.skip {
                self.show_result(r);
            } else {
                self.watching = r.status == "NO_STEPS" && !l.freestyle;
                let end = if l.freestyle {
                    "codeboogie-keep-dancing"
                } else if self.watching {
                    "codeboogie-your-turn"
                } else if r.success {
                    "codeboogie-success"
                } else {
                    "codelab-try-again"
                };
                self.stage
                    .start(queue, Some(end.into()), self.watching, l.freestyle);
                self.dance_report_pending = true;
                self.playing = true;
                self.result = Some(r);
                self.result_at = f64::INFINITY;
            }
        } else {
            let r = coding::lab_run(
                &self.lab,
                &l,
                &self.workspace,
                self.position,
                self.direction,
            );
            self.position = r.position;
            self.direction = r.direction;
            self.run_time = 0.;
            self.last_motion = usize::MAX;
            if r.result.skip {
                self.show_result(r.result);
            } else {
                self.run = Some(r);
            }
            self.sounds.push("computer_play".into());
        }
        self.flush_dance_events();
    }
    pub fn advance(&mut self, seconds: f64) {
        self.update(seconds);
    }
    fn update(&mut self, dt: f64) {
        self.elapsed += dt;
        // Original setLevel schedules one uncancelled three-second callback.
        // Do not continually close a tray the user has reopened afterward.
        if self.portrait_hide_at.iter().any(|at| *at <= self.elapsed) {
            self.portrait_open = false;
            self.portrait_hide_at.retain(|at| *at > self.elapsed);
        }
        if self.chooser {
            return;
        }
        if let Some(run) = &self.run {
            self.run_time += dt / if run.result.success { 1. } else { 1.5 };
            let total: f64 = run.motions.iter().map(|m| m.duration).sum();
            if self.run_time >= total {
                let result = self.run.take().unwrap().result;
                self.sounds.push("computer_walk_stop".into());
                self.show_result(result);
            }
        }
        if self.dance {
            self.stage
                .update(dt, &mut || macroquad::rand::gen_range(0.0f64, 1.0f64));
            self.flush_dance_events();
        }
    }
    fn flush_dance_events(&mut self) {
        let events: Vec<_> = self.stage.events.drain(..).collect();
        for event in events {
            match event {
                DanceEvent::Track(track) => self.track_events.push(track),
                DanceEvent::Sound(sound) => self.sounds.push(sound),
                DanceEvent::Step(id) => {
                    self.highlight = id
                        .as_ref()
                        .and_then(|s| s.parse::<usize>().ok())
                        .and_then(|i| self.actions.get(i))
                        .map(|a| a.id)
                }
                DanceEvent::Finish => {
                    self.playing = false;
                    self.highlight = None;
                    if self.dance_report_pending {
                        self.dance_report_pending = false;
                        if !self.watching {
                            if let Some(result) = self.result.take() {
                                self.show_result(result);
                            }
                        }
                    }
                }
            }
        }
    }
    fn text(&self, s: &str, p: Vec2, size: f32, color: Color, center: bool, lobster: bool) {
        let f = if lobster { &self.lobster } else { &self.font };
        let m = measure_text(s, Some(f), size as u16, 1.);
        draw_text_ex(
            s,
            p.x - if center { m.width / 2. } else { 0. },
            p.y,
            TextParams {
                font: Some(f),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    }
    fn button(&self, text: &str, r: Rect, input: bool) -> bool {
        let hover = r.contains(Vec2::from(mouse_position()));
        self.shapes.rounded(
            r,
            8.,
            Color::from_rgba(if hover { 255 } else { 240 }, 110, 32, 255),
            0.,
        );
        self.text(
            text,
            vec2(r.x + r.w / 2., r.y + r.h * 0.62),
            18.,
            WHITE,
            true,
            false,
        );
        input && hover && is_mouse_button_released(MouseButton::Left)
    }
    fn node_height(&mut self, id: Option<usize>) -> f32 {
        let Some(id) = id else {
            return 0.;
        };
        let b = self.workspace.nodes[id].block.clone();
        let h = self.block_image(&b).size.y;
        let own = if b.kind == "controls_repeat" {
            h + (self.node_height(self.workspace.nodes[id].body) - 41.).max(0.)
        } else {
            h
        };
        own - 5. + self.node_height(self.workspace.nodes[id].next)
    }
    fn chain_draw(&mut self, mut id: Option<usize>, p: Vec2, dragging: bool) {
        let mut y = p.y;
        while let Some(i) = id {
            let b = self.workspace.nodes[i].block.clone();
            let image = self.block_image(&b);
            let body = self.workspace.nodes[i].body;
            let bh = self.node_height(body);
            let h = image.size.y
                + if b.kind == "controls_repeat" {
                    (bh - 41.).max(0.)
                } else {
                    0.
                };
            let r = Rect::new(p.x, y, image.size.x, h);
            let color = if self.highlight == Some(i) {
                Color::new(1., 1., 0.6, 1.)
            } else {
                WHITE
            };
            gl_use_material(&self.premult);
            if b.kind == "controls_repeat" && h > image.size.y {
                let texture = &image.texture;
                let sy = texture.height() / image.paint_size.y;
                for (dy, dh, sy0, sh) in [
                    (
                        image.offset.y,
                        50. - image.offset.y,
                        0.,
                        50. - image.offset.y,
                    ),
                    (50., h - image.size.y + 25., 50. - image.offset.y, 25.),
                    (
                        h - (image.size.y - 75.),
                        image.paint_size.y + image.offset.y - 75.,
                        75. - image.offset.y,
                        image.paint_size.y + image.offset.y - 75.,
                    ),
                ] {
                    draw_texture_ex(
                        texture,
                        r.x,
                        r.y + dy,
                        color,
                        DrawTextureParams {
                            dest_size: Some(vec2(r.w, dh)),
                            source: Some(Rect::new(0., sy0 * sy, texture.width(), sh * sy)),
                            ..Default::default()
                        },
                    );
                }
            } else {
                image.draw(r, color);
            }
            gl_use_default_material();
            self.hits.push(Hit {
                id: i,
                rect: Rect::new(
                    r.x,
                    r.y,
                    r.w,
                    if b.kind == "controls_repeat" {
                        42.
                    } else {
                        r.h
                    },
                ),
            });
            if !dragging {
                self.slots.push(Slot {
                    parent: i,
                    body: false,
                    point: vec2(r.x, r.y + r.h - 5.),
                });
                if b.kind == "controls_repeat" {
                    self.slots.push(Slot {
                        parent: i,
                        body: true,
                        point: vec2(r.x + 24., r.y + 42.),
                    });
                }
            }
            if b.kind == "controls_repeat" {
                self.chain_draw(body, vec2(p.x + 24., y + 42.), dragging);
            }
            y += h - 5.;
            id = self.workspace.nodes[i].next;
        }
    }
    fn layout(&self, w: f32, h: f32) -> (f32, f32, bool) {
        if !self.dance && self.level().kind != "puzzle" {
            let v = coding::lab_viewport(w as f64, h as f64);
            return (v.toolbar as f32, v.width as f32, v.portrait);
        }
        let toolbar = if self.level().kind == "puzzle" {
            0.
        } else if w > 1024. {
            if self.dance {
                238.
            } else {
                236.
            }
        } else if w > 660. {
            if self.dance {
                96.
            } else {
                95.
            }
        } else {
            if self.dance {
                68.
            } else {
                67.
            }
        };
        let scene = if self.level().kind == "puzzle" {
            0.
        } else if self.dance {
            h * (w / 2. / h).clamp(0.5, 1.)
        } else {
            h * (w.max(320.) / 2. / h.max(320.)).clamp(5.45 / 12.6, 7.6 / 9.6)
        };
        let portrait = w - toolbar - scene < 220.;
        (toolbar, if portrait { w - 48. } else { scene }, portrait)
    }
    pub fn draw(&mut self, assets: &mut Assets, input: bool, frozen: bool) {
        let now = get_time();
        let dt = if frozen {
            0.
        } else {
            self.last_time.map_or(0., |last| now - last)
        };
        self.last_time = Some(now);
        self.update(dt);
        let (w, h) = (screen_width(), screen_height());
        clear_background(Color::from_rgba(54, 65, 137, 255));
        if self.chooser {
            self.choose(assets, input);
            return;
        }
        if self.finished {
            self.text(
                &self.msg("codelab-great"),
                vec2(w / 2., h / 2. - 70.),
                85.,
                WHITE,
                true,
                true,
            );
            if self.button(
                "Play again",
                Rect::new(w / 2. - 90., h / 2., 180., 50.),
                input,
            ) {
                self.reset();
            }
            return;
        }
        let (toolbar, scene_width, portrait) = self.layout(w, h);
        let l = self.level().clone();
        let panel_x = if portrait && !self.portrait_open {
            w - scene_width * 0.1
        } else {
            w - scene_width
        };
        let showing_result = self.result.is_some() && self.result_at.is_finite();
        let editable =
            input && !showing_result && (!self.playing || self.dance) && self.run.is_none();
        if l.kind == "puzzle" {
            if w > 1024. {
                draw_rectangle(0., 0., 244., h, Color::from_rgba(63, 77, 161, 255));
            }
            if w > 660. {
                let x = if w > 1024. {
                    w / 2. + 122. - 100.
                } else {
                    w * 0.75 - 100.
                };
                let key = format!("puzzle-ghost-{}", l.id + 1);
                gl_use_material(&self.premult);
                self.image(&key).draw(
                    Rect::new(x, h / 2. - 150., 200., 300.),
                    Color::new(1., 1., 1., 0.5),
                );
                gl_use_default_material();
            }
        } else {
            draw_rectangle(0., 0., toolbar, h, Color::from_rgba(63, 77, 161, 255));
        }
        self.hits.clear();
        self.slots.clear();
        let origin = vec2(toolbar, 0.);
        let roots = self.workspace.roots.clone();
        for id in roots {
            if self.drag.as_ref().is_some_and(|d| d.id == id) {
                continue;
            }
            if self.workspace.nodes[id].block.kind == "when_run" {
                self.workspace.nodes[id].x = if w > 1024. {
                    32.
                } else if w > 660. {
                    24.
                } else {
                    10.
                };
                self.workspace.nodes[id].y = if w > 660. { 105. } else { 80. };
            }
            let p = origin + vec2(self.workspace.nodes[id].x, self.workspace.nodes[id].y);
            self.chain_draw(Some(id), p, false);
        }
        let mut tools = vec![];
        for (i, kind) in l.tools.iter().enumerate() {
            let top = if w > 660. { 105. } else { 80. };
            let p = if w > 1024. {
                if self.dance {
                    vec2(59. + (i % 2) as f32 * 72., top + (i / 2) as f32 * 72.)
                } else {
                    match i {
                        0 => vec2(94.5, top),
                        1 => vec2(94.5, top + 71.),
                        2 => vec2(28.5, top + 71.),
                        3 => vec2(160.5, top + 71.),
                        _ => vec2(94.5, top + (i - 2) as f32 * 71.),
                    }
                }
            } else {
                vec2(if w > 660. { 24. } else { 10. }, top + i as f32 * 70.)
            };
            let im = self.image(&format!("block-{kind}_mini"));
            let r = Rect::new(p.x, p.y, im.size.x, im.size.y);
            gl_use_material(&self.premult);
            im.draw(r, WHITE);
            gl_use_default_material();
            tools.push((kind.clone(), r));
        }
        if l.kind != "puzzle" {
            let diameter = if w > 1024. { 102. } else { 52. };
            let bottom = if w > 660. { 24. } else { 8. };
            let center = vec2(toolbar / 2., h - bottom - diameter / 2.);
            self.shapes
                .circle(center, diameter / 2., Color::new(0., 0., 0., 0.15), 0.);
            let icon = if w > 1024. { 72. } else { 32. };
            gl_use_material(&self.premult);
            self.image("trash").draw(
                Rect::new(center.x - icon / 2., center.y - icon / 2., icon, icon),
                WHITE,
            );
            gl_use_default_material();
        }
        let mouse = Vec2::from(mouse_position());

        if editable {
            if let Some(d) = &self.drag {
                self.workspace.nodes[d.id].x = mouse.x - origin.x - d.offset.x;
                self.workspace.nodes[d.id].y = mouse.y - origin.y - d.offset.y;
            }
            if is_mouse_button_pressed(MouseButton::Left) && self.dropdown.is_none() {
                if let Some((kind, r)) = tools.iter().find(|(_, r)| r.contains(mouse)) {
                    let id = self.workspace.add(Block::new(kind));
                    self.workspace.roots.push(id);
                    self.workspace.nodes[id].x = mouse.x - origin.x;
                    self.workspace.nodes[id].y = mouse.y - origin.y;
                    self.drag = Some(Drag {
                        id,
                        offset: vec2(10., 10.),
                    });
                    self.sounds.push("computer_pickup".into());
                    let _ = r;
                } else if let Some(hit) =
                    self.hits.iter().rev().find(|hit| hit.rect.contains(mouse))
                {
                    let (id, r) = (hit.id, hit.rect);
                    let b = &self.workspace.nodes[id].block;
                    if b.kind == "controls_repeat" && mouse.x > r.x + 60. {
                        self.dropdown = Some((
                            id,
                            "TIMES".into(),
                            Rect::new(r.x + 65., r.y + 38., 96., 30.),
                        ));
                    } else if b.kind == "controls_jump" && mouse.x > r.x + 100. {
                        let field = if mouse.x < r.x + 165. {
                            "LENGTH"
                        } else {
                            "DIRECTION"
                        };
                        self.dropdown =
                            Some((id, field.into(), Rect::new(mouse.x, r.y + 40., 110., 30.)));
                    } else if b.movable {
                        self.workspace.detach(id);
                        self.drag = Some(Drag {
                            id,
                            offset: mouse - r.point(),
                        });
                        self.sounds.push("computer_pickup".into());
                    }
                }
            }
            if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
                if let Some(d) = self.drag.take() {
                    if self.workspace.nodes[d.id].block.deletable {
                        self.workspace.roots.retain(|&id| id != d.id);
                    }
                }
            }
            if is_mouse_button_released(MouseButton::Left) {
                if let Some(d) = self.drag.take() {
                    let ids = self.workspace.descendants(d.id);
                    if mouse.x < toolbar
                        && l.kind != "puzzle"
                        && self.workspace.nodes[d.id].block.deletable
                    {
                        self.workspace.roots.retain(|&id| id != d.id);
                        self.sounds.push("computer_trash".into());
                    } else {
                        let p = origin
                            + vec2(self.workspace.nodes[d.id].x, self.workspace.nodes[d.id].y);
                        let nearest = self
                            .slots
                            .iter()
                            .filter(|s| !ids.contains(&s.parent))
                            .min_by(|a, b| a.point.distance(p).total_cmp(&b.point.distance(p)));
                        if let Some(s) = nearest.filter(|s| s.point.distance(p) <= 60.) {
                            let (parent, body) = (s.parent, s.body);
                            if self.workspace.connect(d.id, parent, body) {
                                self.sounds.push("computer_connect".into());
                            }
                        } else {
                            self.sounds.push("computer_place".into());
                        }
                        if l.kind == "puzzle" && self.workspace.puzzle_success(&l) {
                            self.show_result(ResultInfo {
                                success: true,
                                retry: false,
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }
        if let Some(d) = &self.drag {
            let id = d.id;
            let p = origin + vec2(self.workspace.nodes[id].x, self.workspace.nodes[id].y);
            self.chain_draw(Some(id), p, true);
            if let Some(s) = self
                .slots
                .iter()
                .filter(|s| !self.workspace.descendants(id).contains(&s.parent))
                .min_by(|a, b| a.point.distance(p).total_cmp(&b.point.distance(p)))
                .filter(|s| s.point.distance(p) <= 60.)
            {
                draw_line(
                    s.point.x,
                    s.point.y,
                    s.point.x + 100.,
                    s.point.y,
                    3.,
                    YELLOW,
                );
            }
        }
        if l.kind != "puzzle" {
            if portrait && self.portrait_open {
                draw_rectangle(0., 0., panel_x, h, Color::new(0., 0., 0., 0.5));
            }
            if self.dance {
                self.dance_scene(assets, panel_x, scene_width, h);
            } else {
                self.lab_scene(assets, panel_x, scene_width, h);
            }
            unsafe {
                get_internal_gl().quad_gl.scissor(None);
            }
            let r = Rect::new(
                panel_x - 60.,
                h - if w > 1024. { 135. } else { 200. },
                120.,
                120.,
            );
            self.shapes
                .circle(r.center(), 60., Color::from_rgba(54, 65, 137, 255), 0.);
            self.shapes
                .circle(r.center(), 44., Color::from_rgba(255, 70, 0, 255), 0.);
            draw_triangle(
                r.center() + vec2(-10., -15.),
                r.center() + vec2(-10., 15.),
                r.center() + vec2(15., 0.),
                WHITE,
            );
            if editable && is_mouse_button_released(MouseButton::Left) && r.contains(mouse) {
                self.portrait_open = true;
                self.run();
            } else if input
                && portrait
                && is_mouse_button_released(MouseButton::Left)
                && self.drag.is_none()
                && !showing_result
            {
                if mouse.x > panel_x {
                    self.portrait_open = true;
                } else if mouse.x < panel_x {
                    self.portrait_open = false;
                }
            }
        }
        self.draw_dropdown(editable);
        if showing_result {
            self.result_overlay(assets, input);
        }
    }
    fn draw_dropdown(&mut self, input: bool) {
        if let Some((id, field, r)) = self.dropdown.clone() {
            let choices: Vec<String> = match field.as_str() {
                "TIMES" => (2..=6).map(|n| n.to_string()).collect(),
                "LENGTH" => (1..=3).map(|n| n.to_string()).collect(),
                _ => ["west", "east", "north", "south"]
                    .map(String::from)
                    .to_vec(),
            };
            for (i, v) in choices.iter().enumerate() {
                let rect = Rect::new(r.x, r.y + i as f32 * 30., r.w, 30.);
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::from_rgba(37, 42, 91, 255),
                );
                self.text(
                    v,
                    vec2(rect.x + 10., rect.y + 21.),
                    16.,
                    WHITE,
                    false,
                    false,
                );
                if input
                    && rect.contains(Vec2::from(mouse_position()))
                    && is_mouse_button_released(MouseButton::Left)
                {
                    self.workspace.nodes[id]
                        .block
                        .fields
                        .insert(field.clone(), v.clone());
                    self.dropdown = None;
                }
            }
            if input && is_key_pressed(KeyCode::Escape) {
                self.dropdown = None;
            }
        }
    }
    /// Rasterize text from the source font metrics; SVG is an asset renderer,
    /// not a browser/CSS engine. The original letter spacing and shadow are explicit.
    fn chooser_text(
        &mut self,
        text: &str,
        baseline: Vec2,
        size: f64,
        family: &str,
        weight: u16,
        spacing: f64,
        shadow: bool,
        alpha: f32,
    ) {
        let escaped = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let width = 1200.;
        let shadow_offset = if spacing == -1.5 { 5. } else { 6. };
        let shadow_text = if shadow {
            format!(
                r##"<text x="{}" y="{}" fill="#000" fill-opacity="0.15">{escaped}</text>"##,
                width / 2. + shadow_offset,
                size + shadow_offset
            )
        } else {
            String::new()
        };
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{}"><g font-family="{family}" font-size="{size}" font-weight="{weight}" letter-spacing="{spacing}" text-anchor="middle">{shadow_text}<text x="{}" y="{size}" fill="white">{escaped}</text></g></svg>"#,
            size * 1.6,
            width / 2.
        );
        if !self.images.contains_key(&svg) {
            self.images.insert(
                svg.clone(),
                svg_image(svg.as_bytes()).expect("chooser font SVG"),
            );
        }
        let image = &self.images[&svg];
        gl_use_material(&self.premult);
        image.draw(
            Rect::new(
                baseline.x - width as f32 / 2.,
                baseline.y - size as f32,
                image.size.x,
                image.size.y,
            ),
            Color::new(alpha, alpha, alpha, alpha),
        );
        gl_use_default_material();
    }
    fn choose(&mut self, assets: &Assets, input: bool) {
        use santa_tracker::ui::Ease;
        let (w, h) = (screen_width(), screen_height());
        let layout = coding::dance_chooser_layout(w as f64, h as f64);
        let t = self.elapsed - self.entered_at;
        let opacity = (t / 0.5).clamp(0., 1.) as f32;
        draw_rectangle(
            0.,
            0.,
            w,
            h,
            Color::new(63. / 255., 73. / 255., 163. / 255., opacity),
        );
        // Shared overlay-dialog-show: hold -200% for 20%, then ease-in to zero.
        let shift = if w >= 661. {
            -2. * layout.dialog_height * (1. - Ease::In.at(((t / 0.7 - 0.2) / 0.8).clamp(0., 1.)))
        } else {
            0.
        };
        let rect =
            |r: [f64; 4]| Rect::new(r[0] as f32, (r[1] + shift) as f32, r[2] as f32, r[3] as f32);
        let title = rect(layout.title);
        self.chooser_text(
            &self.msg("codeboogie-choose-game-mode"),
            vec2(
                title.center().x,
                title.y + (layout.title_size * 0.84375).round() as f32,
            ),
            layout.title_size,
            "Lobster",
            400,
            -2.,
            true,
            opacity,
        );
        for (i, (file, key, description)) in [
            (
                "mode-class.svg",
                "codeboogie-mode-dance-class",
                "codeboogie-mode-dance-class-description",
            ),
            (
                "mode-freestyle.svg",
                "codeboogie-mode-freestyle",
                "codeboogie-mode-freestyle-description",
            ),
        ]
        .iter()
        .enumerate()
        {
            let option = rect(layout.options[i]);
            let art = rect(layout.art[i]);
            let hover = input && option.contains(Vec2::from(mouse_position()));
            if hover != self.chooser_hover[i] {
                self.chooser_hover[i] = hover;
                self.chooser_hover_at[i] = self.elapsed;
                self.chooser_art_fade[i].set(
                    self.elapsed,
                    [if hover && w >= 680. { 0.7 } else { 1. }],
                    0.2,
                    Ease::InOut,
                );
            }
            if self.elapsed >= self.chooser_hover_at[i] + 0.2 {
                self.chooser_go[i].set(
                    self.elapsed,
                    if hover && w >= 680. {
                        [1., 0.]
                    } else {
                        [0., -43.8]
                    },
                    0.4,
                    Ease::Default,
                );
            }
            let alpha = opacity * self.chooser_art_fade[i].value(self.elapsed)[0] as f32;
            let texture = assets.scene("codeboogie", &format!("choose-mode/{file}"));
            // background-size: contain preserves the teacher SVG's 397:371.2
            // aspect ratio inside its square padding box.
            let scale = (art.w / texture.width()).min(art.h / texture.height());
            draw_texture_ex(
                texture,
                art.x,
                art.y,
                Color::new(1., 1., 1., alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(texture.width(), texture.height()) * scale),
                    ..Default::default()
                },
            );
            let label = rect(layout.labels[i]);
            self.chooser_text(
                &self.msg(key),
                vec2(
                    label.center().x,
                    label.y + if w >= 680. { 37. } else { 25. },
                ),
                layout.label_size,
                "Roboto",
                800,
                0.,
                false,
                opacity,
            );
            let desc = rect(layout.descriptions[i]);
            self.chooser_text(
                &self.msg(description),
                vec2(desc.center().x, desc.y + 12.),
                12.,
                "Roboto",
                400,
                0.,
                false,
                opacity,
            );
            let go = self.chooser_go[i].value(self.elapsed);
            if go[0] > 0. {
                // .go's source margins place its baseline at top + 87.6 + 74.
                self.chooser_text(
                    &self.msg("codeboogie-go"),
                    vec2(option.center().x - 7.76, option.y + 161.6 + go[1] as f32),
                    87.6,
                    "Lobster",
                    700,
                    -1.5,
                    true,
                    opacity * go[0] as f32,
                );
            }
            if input
                && option.contains(Vec2::from(mouse_position()))
                && is_mouse_button_released(MouseButton::Left)
            {
                self.chooser = false;
                if i == 1 {
                    let stage = macroquad::rand::gen_range(0, 3);
                    self.levels = vec![self.dance_data.freestyle[stage].clone()];
                } else {
                    self.levels = self.dance_data.levels.clone();
                }
                self.set_level(0);
                self.sounds.push("cb_ingame_win".into());
            }
        }
    }
    fn lab_scene(&mut self, assets: &Assets, x: f32, w: f32, h: f32) {
        unsafe {
            get_internal_gl()
                .quad_gl
                .scissor(Some((x as i32, 0, w.ceil() as i32, h as i32)));
        }
        let l = self.level().clone();
        draw_rectangle(x, 0., w, h, WHITE);
        let tile = coding::lab_viewport(screen_width() as f64, h as f64).tile as f32;
        let em = tile / 8.4;
        // Source getWorldTransform uses its fixed 12.6-row viewport, even
        // when the width-driven tile size makes that viewport taller than the window.
        let y0 = 0.3 * tile;
        gl_use_material(&self.premult);
        for y in (l.min_y - 1).max(0)..=(l.max_y).min(self.lab.map.len() as i32 - 1) {
            for col in (0..8).rev() {
                let tile_kind = self.lab.map[y as usize].as_bytes()[col];
                let p = vec2(
                    x + tile * 0.3 + col as f32 * tile,
                    y0 + (y - l.min_y) as f32 * tile,
                );
                if p.y > h || p.y + tile < 0. {
                    continue;
                }
                if tile_kind == b'T' {
                    let t = self.image("maze-tree");
                    t.draw(Rect::new(p.x, p.y, 15. * em, 8. * em), WHITE);
                } else {
                    gl_use_default_material();
                    self.shapes.rounded(
                        Rect::new(p.x, p.y, 8. * em, 8. * em),
                        1.6 * em,
                        Color::from_rgba(225, 243, 248, 255),
                        0.,
                    );
                    gl_use_material(&self.premult);
                    let deco = match tile_kind {
                        b'Y' => Some(("maze-yeti", vec2(0.9, 0.), vec2(14.7, 16.))),
                        b'S' => Some(("maze-sled", vec2(-8., -40.), vec2(36.6, 69.1))),
                        b'A' if l.id == 2 => Some(("maze-arrow", vec2(1., -4.), vec2(6.8, 14.5))),
                        _ => None,
                    };
                    if let Some((name, offset, size)) = deco {
                        self.image(name).draw(
                            Rect::new(
                                p.x + offset.x * em,
                                p.y + offset.y * em,
                                size.x * em,
                                size.y * em,
                            ),
                            WHITE,
                        );
                    }
                }
            }
        }
        let mut pos = vec2(self.position.x as f32, self.position.y as f32);
        let mut direction = self.direction as f32;
        let mut moving = 0.;
        let mut kind = String::from("idle");
        let mut picked = false;
        if let Some(run) = &self.run {
            let mut t = self.run_time;
            for (i, m) in run.motions.iter().enumerate() {
                if t <= m.duration {
                    let turn = if m.old_direction != m.direction {
                        0.2
                    } else {
                        0.
                    };
                    let mut old = m.old_direction as f32;
                    let target = m.direction as f32;
                    if old < target && target - old > 180. {
                        old += 360.;
                    } else if old > target && old - target > 180. {
                        old -= 360.;
                    }
                    direction = old + (target - old) * (t / 0.2).clamp(0., 1.) as f32;
                    let progress = ((t - turn - if m.kind == "walk" { 0.1 } else { 0. }) / 0.4)
                        .clamp(0., 1.) as f32;
                    pos = vec2(m.from.x as f32, m.from.y as f32)
                        .lerp(vec2(m.to.x as f32, m.to.y as f32), progress);
                    moving = (t - turn - if m.kind == "walk" { 0.1 } else { 0. }).max(0.);
                    kind = m.kind.clone();
                    picked = m.kind == "pickup";
                    self.highlight = Some(m.id);
                    if self.last_motion != i {
                        self.last_motion = i;
                        self.sounds.push(
                            match m.kind.as_str() {
                                "walk" => "computer_walk_start",
                                "jump" | "pickup" => "computer_success",
                                _ => "computer_huh",
                            }
                            .into(),
                        );
                    }
                    break;
                }
                t -= m.duration;
            }
        }
        for p in &l.presents {
            if !picked {
                self.image("maze-present").draw(
                    Rect::new(
                        x + tile * 0.3 + p.x as f32 * tile + 1.5 * em,
                        y0 + (p.y - l.min_y) as f32 * tile + 2. * em,
                        6.1 * em,
                        4. * em,
                    ),
                    WHITE,
                );
            }
        }
        let center = vec2(
            x + tile * 0.3 + pos.x * tile + 4. * em,
            y0 + (pos.y - l.min_y as f32) * tile + 4. * em,
        );
        let image = self.image("player-walk");
        let frame = if kind == "walk" {
            ((moving / 0.4 * 8.).floor() as u32).min(7)
        } else {
            0
        };
        let size = vec2(6.05, 6.975) * em;
        draw_texture_ex(
            &image.texture,
            center.x - size.x / 2.,
            center.y - size.y / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                source: Some(Rect::new(
                    0.,
                    image.texture.height() / 8. * frame as f32,
                    image.texture.width(),
                    image.texture.height() / 8.,
                )),
                rotation: direction.to_radians(),
                pivot: Some(center),
                ..Default::default()
            },
        );
        if kind == "jump" {
            let t = assets.scene("codelab", "jump.svg");
            let frame = ((moving / 1. * 15.).floor() as u32).min(14);
            let texh = t.width() * 8.904 / 6.68;
            draw_texture_ex(
                t,
                center.x - 3.34 * em,
                center.y - 4.45 * em,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(6.68, 8.9) * em),
                    source: Some(Rect::new(0., frame as f32 * texh, t.width(), texh)),
                    rotation: direction.to_radians(),
                    pivot: Some(center),
                    ..Default::default()
                },
            );
        }
        if picked {
            self.image("player-victory").draw(
                Rect::new(center.x - 3.2 * em, center.y - 4.5 * em, 7. * em, 12.2 * em),
                WHITE,
            );
        }
        if kind == "lose" {
            self.image("player-lost").draw(
                Rect::new(center.x - 3. * em, center.y - 6. * em, 6. * em, 2.5 * em),
                WHITE,
            );
        }
        gl_use_default_material();
    }
    /// Controlled scene-only rendering for the comparison example. Gameplay
    /// uses the same model and renderer; this does not install game controls.
    #[allow(dead_code)]
    pub fn prepare_stage_capture(&mut self, level: usize, watching: bool) {
        self.levels = self.dance_data.levels.clone();
        self.level_index = level;
        self.chooser = false;
        self.stage = DanceStage::default();
        let level = self.level().clone();
        self.stage.set_level(&level);
        let mut seed = 12345u32;
        self.stage.lights.on_beat(0., 0, false, &mut || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed as f64 / 4294967296.
        });
        self.stage.player = Some(SpriteClock::new("idle".into(), self.level().bpm));
        self.stage.teacher = Some(SpriteClock::new("idle".into(), self.level().bpm));
        self.stage.watching = watching;
        self.stage.showing = watching;
        if watching {
            self.stage
                .player_watch
                .set(0., [1.], 1., santa_tracker::ui::Ease::Linear);
            self.stage
                .teacher_show
                .set(0., [1.], 1., santa_tracker::ui::Ease::Linear);
        }
        self.stage
            .title
            .set(0., Some("Watch closely".into()), true, 0);
        self.stage.moves.set_length(8);
        for step in ["pointLeft", "jump", "hip", "splits", "pointRight"] {
            self.stage.moves.add(0., step.into());
        }
        self.stage.now = 2.;
    }
    #[allow(dead_code)]
    pub fn draw_stage_capture(&mut self, assets: &mut Assets) {
        clear_background(Color::from_rgba(54, 65, 137, 255));
        let (_, width, _) = self.layout(screen_width(), screen_height());
        self.dance_scene(assets, screen_width() - width, width, screen_height());
        unsafe {
            get_internal_gl().quad_gl.scissor(None);
        }
    }
    fn dance_scene(&mut self, assets: &mut Assets, x: f32, w: f32, h: f32) {
        self.stage_clip(x, w, h);
        let level = self.level().clone();
        draw_rectangle(x, 0., w, h, Color::from_rgba(63, 77, 161, 255));
        self.cover(
            assets.scene("codeboogie", &format!("stages/{}.svg", level.stage)),
            Rect::new(x, 0., w, h),
            1.,
        );
        let layout = santa_tracker::codeboogie_stage::StageLayout::new(
            screen_width() as f64,
            x as f64,
            w as f64,
            h as f64,
        );
        if level.stage != "stage1" {
            for (i, box_) in layout.ceiling.iter().enumerate() {
                let r = Rect::new(
                    box_[0] as f32,
                    box_[1] as f32,
                    box_[2] as f32,
                    box_[3] as f32,
                );
                self.cover(assets.scene("codeboogie", "lights/ceiling.svg"), r, 1.);
                for (color, file) in [
                    "ceiling-light.svg",
                    "ceiling-light--red.svg",
                    "ceiling-light--green.svg",
                ]
                .iter()
                .enumerate()
                {
                    let opacity =
                        self.stage.lights.beams[i * 3 + color].value(self.stage.now)[0] as f32;
                    if opacity > 0. {
                        self.cover(
                            assets.scene("codeboogie", &format!("lights/{file}")),
                            r,
                            opacity,
                        );
                    }
                }
            }
            for (i, active) in self.stage.lights.tiles.iter().enumerate() {
                if *active {
                    self.cover(
                        assets.scene("codeboogie", &format!("lights/disco_{}.svg", i + 1)),
                        Rect::new(x, 0., w, h),
                        1.,
                    );
                }
            }
        }
        let teacher = self.stage.teacher.clone();
        let player = self.stage.player.clone();
        // is-showing has z-index:1 inside the transformed character container.
        for is_teacher in if self.stage.showing {
            [false, true]
        } else {
            [true, false]
        } {
            if is_teacher && level.freestyle {
                continue;
            }
            let sprite = if is_teacher { &teacher } else { &player };
            if let Some(sprite) = sprite {
                let state = if is_teacher {
                    self.stage.teacher_show.value(self.stage.now)[0]
                } else {
                    self.stage.player_watch.value(self.stage.now)[0]
                };
                let r = layout.character(is_teacher, level.freestyle, state);
                let clip_left = (r[0] as f32).max(x);
                let clip_top = (r[1] as f32).max(0.);
                let clip_right = ((r[0] + r[2]) as f32).min(x + w);
                let clip_bottom = ((r[1] + r[3]) as f32).min(h);
                unsafe {
                    get_internal_gl().quad_gl.scissor(Some((
                        clip_left.floor() as i32,
                        clip_top.floor() as i32,
                        (clip_right - clip_left).max(0.).ceil() as i32,
                        (clip_bottom - clip_top).max(0.).ceil() as i32,
                    )));
                }
                self.character(
                    assets,
                    if is_teacher || level.freestyle {
                        "green"
                    } else {
                        "red"
                    },
                    sprite,
                    vec2(r[0] as f32, r[1] as f32),
                    (r[2] / 622.) as f32,
                );
                self.stage_clip(x, w, h);
            }
        }
        let tiles = self.stage.moves.tiles.clone();
        let translate = self.stage.moves.translate.value(self.stage.now)[0];
        for (i, tile) in tiles.iter().enumerate() {
            let (opacity, scale) = tile.appearance(self.stage.now);
            if opacity <= 0. {
                continue;
            }
            let b = layout.tile(i, self.stage.moves.length, translate, scale);
            let r = Rect::new(b[0] as f32, b[1] as f32, b[2] as f32, b[3] as f32);
            self.shapes.stroke(
                r,
                (layout.em * 0.8 * scale) as f32,
                (3. * scale) as f32,
                Color::new(1., 1., 1., (opacity * 0.3) as f32),
            );
            let icon = match tile.step.as_str() {
                "pointLeft" => "point-left",
                "pointRight" => "point-right",
                "stepLeft" => "step-left",
                "stepRight" => "step-right",
                "hip" => "hip-shake",
                other => other,
            };
            let border = (3. * scale) as f32;
            draw_texture_ex(
                assets.scene("codeboogie", &format!("icons_{icon}-white.svg")),
                r.x + border,
                r.y + border,
                Color::new(1., 1., 1., opacity as f32),
                DrawTextureParams {
                    dest_size: Some(vec2(
                        (10. * layout.em * scale) as f32,
                        (8.5 * layout.em * scale) as f32,
                    )),
                    ..Default::default()
                },
            );
            let baseline = layout.moves_y + layout.tile_size[1] + 2.8 * layout.em
                - 3.
                - (0.35 * layout.em).round();
            let count = (i + 1).to_string();
            let text_scale = (layout.em * 1.4 * scale / 64.) as f32;
            let metrics = measure_text(&count, Some(&self.lobster), 64, text_scale);
            draw_text_ex(
                &count,
                r.center().x - metrics.width / 2.,
                r.center().y
                    + (baseline - (layout.moves_y + layout.tile_size[1] / 2.)) as f32
                        * scale as f32,
                TextParams {
                    font: Some(&self.lobster),
                    font_size: 64,
                    font_scale: text_scale,
                    color: Color::new(1., 1., 1., opacity as f32),
                    ..Default::default()
                },
            );
        }
        let nodes = self.stage.title.nodes.clone();
        for node in nodes {
            let (opacity, scale) = node.appearance(self.stage.now, self.stage.bpm);
            if opacity <= 0. {
                continue;
            }
            let font_size = layout.em
                * if node.sub {
                    25.
                } else if node.small {
                    5.
                } else {
                    8.
                };
            self.stage_title(
                &self.msg(&node.text),
                x + w / 2.,
                if node.sub { 80. } else { 48. },
                font_size,
                opacity,
                scale,
            );
        }
    }
    fn stage_clip(&self, x: f32, w: f32, h: f32) {
        unsafe {
            get_internal_gl()
                .quad_gl
                .scissor(Some((x as i32, 0, w.ceil() as i32, h as i32)));
        }
    }
    fn cover(&self, texture: &Texture2D, r: Rect, opacity: f32) {
        let scale = (r.w / texture.width()).max(r.h / texture.height());
        let size = vec2(texture.width(), texture.height()) * scale;
        draw_texture_ex(
            texture,
            r.center().x - size.x / 2.,
            r.center().y - size.y / 2.,
            Color::new(1., 1., 1., opacity),
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    }
    fn stage_title(
        &mut self,
        text: &str,
        center: f32,
        top: f32,
        size: f64,
        opacity: f64,
        scale: f64,
    ) {
        let escaped = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="{}"><defs><filter id="shadow" x="-10%" y="-30%" width="120%" height="160%"><feDropShadow dx="2" dy="2" stdDeviation="1" flood-color="black" flood-opacity="0.2"/></filter></defs><text x="600" y="{}" font-family="Lobster" font-size="{size}" text-anchor="middle" fill="white" filter="url(#shadow)">{escaped}</text></svg>"##,
            size * 1.6 + 20.,
            size + 10.
        );
        if !self.images.contains_key(&svg) {
            self.images.insert(
                svg.clone(),
                svg_image(svg.as_bytes()).expect("stage title SVG"),
            );
        }
        let image = &self.images[&svg];
        let middle = top + size as f32 / 2.;
        let scale = scale as f32;
        gl_use_material(&self.premult);
        image.draw(
            Rect::new(
                center - 600. * scale,
                middle + (top - 10. - middle) * scale,
                image.size.x * scale,
                image.size.y * scale,
            ),
            Color::new(
                opacity as f32,
                opacity as f32,
                opacity as f32,
                opacity as f32,
            ),
        );
        gl_use_default_material();
    }
    fn character(
        &mut self,
        assets: &mut Assets,
        color: &str,
        sprite_clock: &SpriteClock,
        position: Vec2,
        scale: f32,
    ) {
        let step = &sprite_clock.step;
        let frame = sprite_clock.frame;
        let sprite = format!("{step}_{}", frame / 24);
        let Some(info) = self.dance_data.sprites.get(&sprite) else {
            return;
        };
        let key = format!("scenes/codeboogie/img/steps/{color}/{sprite}.png");
        if !assets.has(&key) {
            if let Err(e) = assets.load(&key) {
                eprintln!("Code Boogie sprite: {e}");
                return;
            }
        }
        let t = assets.texture(&key);
        let offset = vec2(info.offset_x - 265., info.offset_y - 35.);
        draw_texture_ex(
            t,
            position.x + offset.x * scale,
            position.y + offset.y * scale,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(info.width, info.height) * scale),
                source: Some(Rect::new(
                    (frame % 24) as f32 * info.width,
                    0.,
                    info.width,
                    info.height,
                )),
                ..Default::default()
            },
        );
    }
    fn result_overlay(&mut self, _assets: &Assets, input: bool) {
        let r = self.result.clone().unwrap();
        let (w, h) = (screen_width(), screen_height());
        let a = ((self.elapsed - self.result_at) / 0.3).clamp(0., 1.) as f32;
        draw_rectangle(
            0.,
            0.,
            w,
            h,
            if self.dance {
                Color::new(63. / 255., 73. / 255., 163. / 255., 0.9 * a)
            } else {
                Color::new(0., 0., 0., 0.6 * a)
            },
        );
        let title = self.msg(if r.success {
            if self.dance {
                "codeboogie-nice-moves"
            } else {
                "codelab-great"
            }
        } else {
            "codelab-oops"
        });
        self.text(
            &title,
            vec2(w / 2., h * 0.20),
            if w >= 570. && h >= 500. { 100. } else { 65. },
            WHITE,
            true,
            true,
        );
        let message = self.msg(&r.message).replace(
            "{{ideal}}",
            &self.level().ideal_block_count.unwrap_or(0).to_string(),
        );
        self.wrapped(&message, vec2(w / 2., h * 0.28), w.min(550.), 23.);
        if self.show_code {
            let lines: Vec<_> = if self.dance {
                self.dance_user_code.lines().map(str::to_owned).collect()
            } else {
                self.actions
                    .iter()
                    .map(|a| {
                        if a.kind == "controls_jump" {
                            format!("moveJump({}, '{}');", a.length, a.direction)
                        } else {
                            format!(
                                "{}();",
                                a.kind
                                    .trim_start_matches("maze_")
                                    .trim_start_matches("dance_")
                            )
                        }
                    })
                    .collect()
            };
            for (i, line) in lines.iter().take(12).enumerate() {
                self.text(
                    line,
                    vec2(w / 2. - 200., h * 0.36 + i as f32 * 23.),
                    17.,
                    WHITE,
                    false,
                    false,
                );
            }
        } else {
            let graphic = if self.dance {
                if r.success {
                    "result-success"
                } else {
                    "result-failure"
                }
            } else if self.level().kind == "puzzle" {
                "result-puzzle"
            } else if r.success && self.level_index + 1 == self.levels.len() {
                "result-final"
            } else if r.success {
                "result-success"
            } else {
                "result-failure"
            };
            let size = vec2(w.min(480.), (h * 0.36).min(300.));
            gl_use_material(&self.premult);
            if !self.dance {
                self.image("result-trees").draw(
                    Rect::new(w / 2. - size.x / 2., h * 0.34, size.x, size.y),
                    WHITE,
                );
            }
            let im = self.image(graphic);
            let s = size.y / im.size.y;
            im.draw(
                Rect::new(w / 2. - im.size.x * s / 2., h * 0.34, im.size.x * s, size.y),
                WHITE,
            );
            gl_use_default_material();
        }
        let y = h * 0.78;
        if r.retry
            && self.button(
                &self.msg(if self.level().freestyle {
                    "codeboogie-keep-dancing"
                } else {
                    "codelab-try-again"
                }),
                Rect::new(w / 2. - 175., y, 160., 48.),
                input,
            )
        {
            self.result = None;
            self.position = Point {
                x: self.level().player_x,
                y: self.level().player_y,
            };
            self.direction = self.start_direction;
            self.run = None;
        }
        if r.success
            && self.button(
                &self.msg(if self.level_index + 1 == self.levels.len() {
                    "codelab-finish"
                } else {
                    if self.dance {
                        "codeboogie-next-dance"
                    } else {
                        "codelab-next-challenge"
                    }
                }),
                Rect::new(w / 2. + if r.retry { 15. } else { -90. }, y, 180., 48.),
                input,
            )
        {
            if self.level().freestyle {
                self.reset();
            } else {
                self.set_level(self.level_index + 1);
            }
        }
        if self.level().kind != "puzzle"
            && (!self.dance || (r.success && !self.dance_user_code.is_empty() && !self.show_code))
        {
            let p = vec2(w / 2., y + 85.);
            self.text("View code", p, 17., YELLOW, true, false);
            if input
                && Rect::new(p.x - 80., p.y - 22., 160., 32.).contains(Vec2::from(mouse_position()))
                && is_mouse_button_released(MouseButton::Left)
            {
                self.show_code = !self.show_code;
            }
        }
    }
    fn wrapped(&self, s: &str, p: Vec2, width: f32, size: f32) {
        let mut line = String::new();
        let mut y = p.y;
        for word in s.split_whitespace() {
            let next = format!("{line} {word}");
            if measure_text(&next, Some(&self.font), size as u16, 1.).width > width
                && !line.is_empty()
            {
                self.text(&line, vec2(p.x, y), size, WHITE, true, false);
                line = word.into();
                y += size * 1.3;
            } else {
                line = next;
            }
        }
        self.text(&line, vec2(p.x, y), size, WHITE, true, false);
    }
}
