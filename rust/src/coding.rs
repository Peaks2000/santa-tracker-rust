//! Native block graphs and source Code Lab / Code Boogie execution.
//! Original algorithms: Copyright Google LLC, Apache-2.0.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub kind: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
    #[serde(default)]
    pub body: Vec<Block>,
    #[serde(default = "yes")]
    pub editable: bool,
    #[serde(default = "yes")]
    pub movable: bool,
    #[serde(default = "yes")]
    pub deletable: bool,
}
fn yes() -> bool {
    true
}
impl Block {
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.into(),
            fields: HashMap::from([
                ("TIMES".into(), "2".into()),
                ("LENGTH".into(), "1".into()),
                ("DIRECTION".into(), "west".into()),
            ]),
            body: vec![],
            editable: true,
            movable: true,
            deletable: true,
        }
    }
    pub fn number(&self, key: &str) -> usize {
        self.fields
            .get(key)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }
}
#[derive(Clone, Debug)]
pub struct Node {
    pub block: Block,
    pub next: Option<usize>,
    pub body: Option<usize>,
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Debug, Default)]
pub struct Workspace {
    pub nodes: Vec<Node>,
    pub roots: Vec<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub id: usize,
    pub kind: String,
    pub length: usize,
    pub direction: String,
}
impl Workspace {
    pub fn level(level: &Level) -> Self {
        let mut w = Self::default();
        for (i, blocks) in level.start.iter().enumerate() {
            if let Some(id) = w.chain(blocks) {
                w.nodes[id].x = 20.;
                w.nodes[id].y = 24.
                    + i as f32
                        * if level.kind == "puzzle" {
                            300. / level.num_pieces as f32 + 20.
                        } else {
                            170.
                        };
                w.roots.push(id);
            }
        }
        if level.insert_when_run {
            let mut block = Block::new("when_run");
            block.movable = false;
            block.deletable = false;
            let id = w.add(block);
            if !w.roots.is_empty() {
                w.nodes[id].next = Some(w.roots.remove(0));
            }
            w.roots.insert(0, id);
        }
        w
    }
    fn chain(&mut self, blocks: &[Block]) -> Option<usize> {
        let mut first: Option<usize> = None;
        let mut prev: Option<usize> = None;
        for b in blocks {
            let id = self.add(b.clone());
            let body = self.chain(&b.body);
            self.nodes[id].body = body;
            if let Some(p) = prev {
                self.nodes[p].next = Some(id);
            } else {
                first = Some(id);
            }
            prev = Some(id);
        }
        first
    }
    pub fn add(&mut self, mut block: Block) -> usize {
        block.body.clear();
        let id = self.nodes.len();
        self.nodes.push(Node {
            block,
            next: None,
            body: None,
            x: 0.,
            y: 0.,
        });
        id
    }
    pub fn detach(&mut self, id: usize) {
        for node in &mut self.nodes {
            if node.next == Some(id) {
                node.next = None;
            }
            if node.body == Some(id) {
                node.body = None;
            }
        }
        self.roots.retain(|&r| r != id);
        self.roots.push(id);
    }
    pub fn connect(&mut self, id: usize, parent: usize, body: bool) -> bool {
        if self.nodes[id].block.kind == "puzzle_0A"
            || (!body && self.nodes[parent].block.kind == "puzzle_0B")
        {
            return false;
        }
        if self.descendants(id).contains(&parent) {
            return false;
        }
        self.roots.retain(|&r| r != id);
        let old = if body {
            self.nodes[parent].body.replace(id)
        } else {
            self.nodes[parent].next.replace(id)
        };
        let mut last = id;
        while let Some(n) = self.nodes[last].next {
            last = n;
        }
        self.nodes[last].next = old;
        true
    }
    pub fn descendants(&self, id: usize) -> HashSet<usize> {
        let mut out = HashSet::new();
        self.visit(Some(id), &mut out);
        out
    }
    fn visit(&self, id: Option<usize>, set: &mut HashSet<usize>) {
        if let Some(id) = id {
            if set.insert(id) {
                self.visit(self.nodes[id].next, set);
                self.visit(self.nodes[id].body, set);
            }
        }
    }
    pub fn all(&self) -> HashSet<usize> {
        let mut out = HashSet::new();
        for &r in &self.roots {
            self.visit(Some(r), &mut out);
        }
        out
    }
    pub fn extra(&self) -> bool {
        self.roots
            .iter()
            .any(|&id| self.nodes[id].block.kind != "when_run")
    }
    pub fn empty(&self) -> bool {
        self.all().iter().any(|&id| {
            self.nodes[id].block.kind == "controls_repeat" && self.nodes[id].body.is_none()
        })
    }
    pub fn missing(&self, required: &[String]) -> Vec<String> {
        let all = self.all();
        required
            .iter()
            .filter(|kind| {
                !all.iter()
                    .any(|&id| self.nodes[id].block.editable && self.nodes[id].block.kind == **kind)
            })
            .cloned()
            .collect()
    }
    pub fn count(&self) -> usize {
        self.all()
            .iter()
            .filter(|&&id| self.nodes[id].block.kind != "when_run")
            .count()
    }
    pub fn actions(&self) -> Result<Vec<Action>, String> {
        let mut out = vec![];
        for &r in &self.roots {
            if self.nodes[r].block.kind == "when_run" {
                self.expand(self.nodes[r].next, &mut out)?;
            }
        }
        Ok(out)
    }
    fn expand(&self, mut node: Option<usize>, out: &mut Vec<Action>) -> Result<(), String> {
        while let Some(id) = node {
            let n = &self.nodes[id];
            if n.block.kind == "controls_repeat" {
                for _ in 0..n.block.number("TIMES") {
                    self.expand(n.body, out)?;
                }
            } else {
                if out.len() >= 100000 {
                    return Err("Program is too large to preview".into());
                }
                out.push(Action {
                    id,
                    kind: n.block.kind.clone(),
                    length: n.block.number("LENGTH"),
                    direction: n.block.fields.get("DIRECTION").cloned().unwrap_or_default(),
                });
            }
            node = n.next;
        }
        Ok(())
    }
    pub fn puzzle_success(&self, level: &Level) -> bool {
        if self.roots.len() != 1 {
            return false;
        }
        let mut node = Some(self.roots[0]);
        for n in 0..level.num_pieces {
            let Some(id) = node else {
                return false;
            };
            if self.nodes[id].block.kind
                != format!("puzzle_{}{}", level.id, (b'A' + n as u8) as char)
                || self.nodes[id].body.is_some()
            {
                return false;
            }
            node = self.nodes[id].next;
        }
        node.is_none()
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub id: usize,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub insert_when_run: bool,
    pub start: Vec<Vec<Block>>,
    pub tools: Vec<String>,
    #[serde(default)]
    pub num_pieces: usize,
    #[serde(default)]
    pub min_y: i32,
    #[serde(default)]
    pub max_y: i32,
    #[serde(default)]
    pub player_x: i32,
    #[serde(default)]
    pub player_y: i32,
    #[serde(default)]
    pub presents: Vec<Point>,
    pub ideal_block_count: Option<usize>,
    #[serde(default)]
    pub required_blocks: Vec<String>,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub bpm: f64,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub track: usize,
    #[serde(default)]
    pub freestyle: bool,
    #[serde(default)]
    pub longer_intro: bool,
    #[serde(default)]
    pub special_move: String,
    #[serde(default)]
    pub fade_tiles: bool,
    #[serde(default)]
    pub ceiling_lights: bool,
    #[serde(default)]
    pub floor_lights: bool,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Deserialize)]
pub struct LabData {
    pub map: Vec<String>,
    pub levels: Vec<Level>,
}
#[derive(Deserialize)]
pub struct DanceData {
    pub levels: Vec<Level>,
    pub freestyle: Vec<Level>,
    pub sprites: HashMap<String, Sprite>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}
pub fn lab_data() -> LabData {
    serde_json::from_str(include_str!("../data/codelab-levels.json")).unwrap()
}
pub fn dance_data() -> DanceData {
    serde_json::from_str(include_str!("../data/codeboogie-levels.json")).unwrap()
}
#[derive(Clone, Copy, Debug)]
pub struct LabViewport {
    pub toolbar: f64,
    pub width: f64,
    pub portrait: bool,
    pub tile: f64,
}
/// Original chooser CSS layout, including the freestyle caption's normal-flow
/// `choose-mode-info` class (the source does not use the teacher's absolute class).
#[derive(Debug)]
pub struct ChooserLayout {
    pub title: [f64; 4],
    pub title_size: f64,
    pub options: [[f64; 4]; 2],
    pub art: [[f64; 4]; 2],
    pub labels: [[f64; 4]; 2],
    pub descriptions: [[f64; 4]; 2],
    pub label_size: f64,
    pub dialog_height: f64,
}
pub fn dance_chooser_layout(width: f64, height: f64) -> ChooserLayout {
    // Blink stores layout lengths in 1/64px units. Retain CSS rounding before
    // computing inline baselines, percentage padding and collapsed margins.
    let px = |n: f64| (n * 64.).floor() / 64.;
    let desktop = width >= 680.;
    let dw = px((width * 0.8).min(960.));
    let dx = (width - dw) / 2. - 1. / 64.;
    let fs = if width >= 1025. {
        96.
    } else if desktop {
        width * 0.08
    } else {
        36.
    };
    let th = px(fs * 0.95);
    let gap = if desktop { 96. } else { 48. };
    let ow = if desktop { px(dw * 0.45) } else { dw };
    let label_size = if desktop { 36. } else { 24. };
    let lh = px(label_size * 1.4);
    let margin = px(label_size * 0.8);
    let dh = px(12. * 1.4);
    let teacher_art = px(ow * if desktop { 1. } else { 0.5 } * 397. / 360.234);
    let free_art = px(ow * if desktop { 0.75 } else { 0.5 });
    let teacher_h = teacher_art + 3.;
    let free_h = free_art + 3. - margin + lh + 4. + dh + if desktop { 20. } else { 0. };
    let grid_h = if desktop {
        teacher_h.max(free_h) + 40.
    } else {
        teacher_h + free_h + 80.
    };
    let dialog_height = th + gap + grid_h;
    let top = ((height - dialog_height) / 2.).max(20.);
    let gy = top + th + gap;
    let tx = if desktop { dx + px(dw * 0.025) } else { dx };
    let fx = if desktop {
        dx + dw / 2. + px(dw * 0.025)
    } else {
        dx
    };
    let ty = gy
        + if desktop {
            (teacher_h.max(free_h) - teacher_h) / 2.
        } else {
            0.
        };
    let fy = if desktop {
        gy + (teacher_h.max(free_h) - free_h) / 2.
    } else {
        ty + teacher_h + 40.
    };
    let teacher_label_y = ty + if desktop { ow } else { ow * 0.5 } - margin;
    let free_label_y = fy + free_art + 3. - margin;
    ChooserLayout {
        title: [dx, top, dw, th],
        title_size: fs,
        label_size,
        dialog_height,
        options: [[tx, ty, ow, teacher_h], [fx, fy, ow, free_h]],
        art: [
            [
                tx + (ow - teacher_art - (teacher_art - if desktop { ow } else { ow * 0.5 })) / 2.,
                ty,
                teacher_art,
                teacher_art,
            ],
            [fx + (ow - free_art) / 2., fy, free_art, free_art],
        ],
        labels: [[tx, teacher_label_y, ow, lh], [fx, free_label_y, ow, lh]],
        descriptions: [
            [tx, teacher_label_y + lh + 4., ow, dh],
            [fx, free_label_y + lh + 4., ow, dh],
        ],
    }
}

pub fn lab_viewport(width: f64, height: f64) -> LabViewport {
    let (w, h) = (width.max(320.), height.max(320.));
    let toolbar = if w > 1024. {
        236.
    } else if w > 660. {
        95.
    } else {
        67.
    };
    let desired = h * (w / 2. / h).clamp(5.45 / 12.6, 7.6 / 9.6);
    let tile = (h / 12.6).max(desired / 7.6);
    let portrait = w - toolbar - desired < 220.;
    LabViewport {
        toolbar,
        width: if portrait { w - 48. } else { desired },
        portrait,
        tile,
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Motion {
    pub id: usize,
    pub from: Point,
    pub to: Point,
    pub old_direction: i32,
    pub direction: i32,
    pub duration: f64,
    pub kind: String,
}
#[derive(Clone, Debug, Default)]
pub struct ResultInfo {
    pub success: bool,
    pub message: String,
    pub retry: bool,
    pub skip: bool,
    pub status: String,
    pub queue: Vec<DanceItem>,
    pub missing_blocks: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DanceItem {
    pub teacher_step: String,
    pub player_step: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_countdown: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_intro: bool,
}
fn item(t: &str, p: &str, title: &str) -> DanceItem {
    DanceItem {
        teacher_step: t.into(),
        player_step: p.into(),
        title: title.into(),
        tile: None,
        block_id: None,
        is_countdown: false,
        is_intro: false,
    }
}
pub fn dance_queue(level: &Level, steps: &[String]) -> (String, Vec<DanceItem>) {
    let mut status = "SUCCESS";
    let mut used = steps.len();
    if level.freestyle {
        if steps.is_empty() {
            status = "NO_STEPS";
        }
    } else {
        for (i, s) in steps.iter().enumerate() {
            if i >= level.steps.len() {
                status = "TOO_MANY_STEPS";
                break;
            }
            if s != &level.steps[i] {
                status = "WRONG_STEPS";
                used = i + 1;
                break;
            }
        }
        if status == "SUCCESS" {
            if steps.is_empty() {
                status = "NO_STEPS";
            } else if steps.len() < level.steps.len() {
                status = "NOT_ENOUGH_STEPS";
            }
        }
    }
    let teacher = if level.freestyle {
        &steps[..used]
    } else {
        &level.steps
    };
    let mut queue = vec![];
    for (i, t) in teacher.iter().enumerate() {
        let mut a = item(
            t,
            steps
                .get(i)
                .filter(|_| i < used)
                .map(String::as_str)
                .unwrap_or("watch"),
            step_title(t),
        );
        a.tile = Some(t.clone());
        a.block_id = (i < used).then(|| i.to_string());
        queue.push(a);
    }
    if status == "WRONG_STEPS" {
        if used == teacher.len() {
            queue.push(item("watch", "fail", "codelab-oops"));
        } else {
            queue[used].player_step = "fail".into();
            queue[used].title = "codelab-oops".into();
        }
    }
    if status == "TOO_MANY_STEPS" {
        let mut a = item("watch", &steps[teacher.len()], "codelab-oops");
        a.block_id = Some(teacher.len().to_string());
        queue.push(a);
        queue.push(item("watch", "fail", "codelab-oops"));
    }
    if status == "SUCCESS" && !level.freestyle {
        let s = if level.special_move.is_empty() {
            "pony"
        } else {
            &level.special_move
        };
        queue.push(item(s, s, "codeboogie-success"));
    }
    if !(level.freestyle && status == "NO_STEPS") {
        let mut a = item(
            "idle",
            "idle",
            if status == "NO_STEPS" {
                "codeboogie-watch-closely"
            } else {
                "codeboogie-lets-dance"
            },
        );
        a.is_countdown = true;
        a.is_intro = true;
        queue.insert(0, a);
    }
    (status.into(), queue)
}
fn step_title(step: &str) -> &str {
    match step {
        "pointLeft" => "codeboogie-block-leftarm",
        "pointRight" => "codeboogie-block-rightarm",
        "stepLeft" => "codeboogie-block-leftfoot",
        "stepRight" => "codeboogie-block-rightfoot",
        "jump" => "codeboogie-block-jump",
        "splits" => "codeboogie-block-split",
        "hip" => "codeboogie-block-shake",
        _ => "?",
    }
}
#[derive(Clone, Debug)]
pub struct LabRun {
    pub motions: Vec<Motion>,
    pub result: ResultInfo,
    pub position: Point,
    pub direction: i32,
}
pub fn lab_run(data: &LabData, l: &Level, w: &Workspace, start: Point, direction: i32) -> LabRun {
    let mut r = LabRun {
        motions: vec![],
        result: ResultInfo {
            retry: true,
            ..Default::default()
        },
        position: start,
        direction,
    };
    for a in w.actions().unwrap_or_default() {
        let (dir, len, jump) = match a.kind.as_str() {
            "maze_moveNorth" => (0, 1, false),
            "maze_moveEast" => (90, 1, false),
            "maze_moveSouth" => (180, 1, false),
            "maze_moveWest" => (270, 1, false),
            "controls_jump" => (
                match a.direction.as_str() {
                    "north" => 0,
                    "east" => 90,
                    "south" => 180,
                    _ => 270,
                },
                a.length as i32 + 1,
                true,
            ),
            _ => continue,
        };
        let (dx, dy) = match dir {
            0 => (0, -1),
            90 => (1, 0),
            180 => (0, 1),
            _ => (-1, 0),
        };
        let to = Point {
            x: r.position.x + dx * len,
            y: r.position.y + dy * len,
        };
        let blocked = to.x < 0
            || to.x >= 7
            || to.y < l.min_y
            || to.y >= l.max_y
            || data
                .map
                .get(to.y as usize)
                .and_then(|row| row.as_bytes().get(to.x as usize))
                .copied()
                == Some(b'T');
        let old = r.direction;
        let turn = if old != dir { 0.2 } else { 0. };
        let kind = if blocked {
            "lose"
        } else if jump {
            "jump"
        } else {
            "walk"
        };
        let end = if blocked { r.position } else { to };
        r.motions.push(Motion {
            id: a.id,
            from: r.position,
            to: end,
            old_direction: old,
            direction: dir,
            duration: if blocked {
                0.96 + turn
            } else {
                (if jump { 1.0 } else { 0.5 }) + turn
            },
            kind: kind.into(),
        });
        r.direction = dir;
        r.position = end;
        if blocked {
            break;
        }
        if l.presents.iter().filter(|&&p| p == r.position).count() == l.presents.len() {
            r.result.success = true;
            r.motions.push(Motion {
                id: a.id,
                from: end,
                to: end,
                old_direction: dir,
                direction: dir,
                duration: 0.8,
                kind: "pickup".into(),
            });
            break;
        }
    }
    if !r.result.success && r.motions.last().is_none_or(|m| m.kind != "lose") {
        r.motions.push(Motion {
            id: usize::MAX,
            from: r.position,
            to: r.position,
            old_direction: r.direction,
            direction: r.direction,
            duration: 0.8,
            kind: "lose".into(),
        });
    }
    r.result = evaluate(l, w, r.result.success, false);
    r
}
pub fn evaluate(l: &Level, w: &Workspace, success: bool, dance: bool) -> ResultInfo {
    let mut r = ResultInfo {
        success,
        retry: true,
        ..Default::default()
    };
    if w.empty() {
        r.success = false;
        r.skip = true;
        r.message = "codelab-result-empty-repeat-fail".into();
        return r;
    }
    if w.extra() {
        r.success = false;
        r.skip = true;
        r.message = "codelab-result-extra-top-block-fail".into();
        return r;
    }
    let prefix = if dance { "codeboogie" } else { "codelab" };
    r.missing_blocks = w.missing(&l.required_blocks);
    if !r.missing_blocks.is_empty() {
        r.message = format!(
            "{prefix}-result-missing-block-{}",
            if success { "success" } else { "fail" }
        );
    } else if !success {
        r.message = if l.ideal_block_count.is_some_and(|n| w.count() < n) {
            format!("{prefix}-result-too-few-blocks-fail")
        } else {
            "codelab-result-generic-fail".into()
        };
    } else if l.ideal_block_count.is_some_and(|n| w.count() > n) {
        r.message = format!("{prefix}-result-too-many-blocks-success");
    } else if l.freestyle {
        r.message = "codeboogie-share-moves".into();
    } else {
        r.retry = false;
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chooser_layout_matches_unmodified_original_dom() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/codeboogie-chooser-reference.json"))
                .unwrap();
        for row in data["cases"].as_array().unwrap() {
            let l = dance_chooser_layout(
                row["width"].as_f64().unwrap(),
                row["height"].as_f64().unwrap(),
            );
            for (selector, boxes) in [
                (".overlay-title", vec![l.title]),
                (".grid__option", l.options.to_vec()),
                (".choose-mode__bg", l.art.to_vec()),
                (".choose-mode__text", l.labels.to_vec()),
                (".choose-mode__description", l.descriptions.to_vec()),
            ] {
                for (native, original) in
                    boxes.iter().zip(row["items"][selector].as_array().unwrap())
                {
                    for (i, key) in ["x", "y", "w", "h"].iter().enumerate() {
                        assert!(
                            (native[i] - original[key].as_f64().unwrap()).abs() < 0.05,
                            "{}px {} {}: {:?} vs {}",
                            row["width"],
                            selector,
                            key,
                            native,
                            original
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn viewport_and_fixed_world_origin_match_source_dom() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/codelab-viewport-reference.json")).unwrap();
        for c in fixture["cases"].as_array().unwrap() {
            let w = c["width"].as_f64().unwrap();
            let h = c["height"].as_f64().unwrap();
            let v = lab_viewport(w, h);
            assert!((v.width - c["scene"]["w"].as_f64().unwrap()).abs() < 0.03);
            assert_eq!(v.toolbar, c["toolbar"].as_f64().unwrap());
            let em = v.tile / 8.4;
            let source_em = c["fontSize"]
                .as_str()
                .unwrap()
                .trim_end_matches("px")
                .parse::<f64>()
                .unwrap();
            assert!((em - source_em).abs() < 0.00002);
            let x = w - v.width + v.tile * 3.3 + 4. * em;
            let y = v.tile * 3.3 + 4. * em;
            assert!(
                (x - c["player"]["x"].as_f64().unwrap()).abs() < 0.03,
                "{c}: {x}"
            );
            assert!(
                (y - c["player"]["y"].as_f64().unwrap()).abs() < 0.03,
                "{c}: {y}"
            );
        }
    }
    #[test]
    fn exported_scene_svg_assets_are_valid_native_images() {
        for scene in ["codelab", "codeboogie"] {
            let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(scene);
            for file in std::fs::read_dir(folder).unwrap() {
                let path = file.unwrap().path();
                if path.extension().is_some_and(|ext| ext == "svg") {
                    let bytes = std::fs::read(&path).unwrap();
                    assert!(
                        !String::from_utf8_lossy(&bytes).contains("href=\"img/"),
                        "Unresolved asset in {}",
                        path.display()
                    );
                    assert!(
                        resvg::usvg::Tree::from_data(&bytes, &resvg::usvg::Options::default())
                            .is_ok(),
                        "{}",
                        path.display()
                    );
                }
            }
        }
    }
    #[test]
    fn original_dance_status_and_queues() {
        let data = dance_data();
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../data/codeboogie-evaluation-reference.json"))
                .unwrap();
        let msgs: HashMap<String, String> =
            serde_json::from_str(include_str!("../data/coding-messages.json")).unwrap();
        for c in cases.as_array().unwrap() {
            let level = &data.levels[c["level"].as_u64().unwrap() as usize];
            let steps: Vec<String> = serde_json::from_value(c["steps"].clone()).unwrap();
            let (s, mut q) = dance_queue(level, &steps);
            assert_eq!(s, c["status"]);
            for a in &mut q {
                a.title = msgs
                    .get(&a.title)
                    .cloned()
                    .unwrap_or_else(|| a.title.clone());
            }
            assert_eq!(serde_json::to_value(q).unwrap(), c["queue"]);
        }
    }
    #[test]
    fn movement_matches_16128_actual_source_player_calls() {
        let data = lab_data();
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/codelab-movement-reference.json")).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let l = &data.levels[case["level"].as_u64().unwrap() as usize];
            let dir = case["direction"].as_i64().unwrap() as i32;
            let length = case["length"].as_u64().unwrap() as usize;
            let mut w = Workspace::default();
            let root = w.add(Block::new("when_run"));
            w.roots.push(root);
            let mut b = Block::new(if length > 0 {
                "controls_jump"
            } else {
                match dir {
                    0 => "maze_moveNorth",
                    90 => "maze_moveEast",
                    180 => "maze_moveSouth",
                    _ => "maze_moveWest",
                }
            });
            b.fields.insert("LENGTH".into(), length.to_string());
            b.fields.insert(
                "DIRECTION".into(),
                match dir {
                    0 => "north",
                    90 => "east",
                    180 => "south",
                    _ => "west",
                }
                .into(),
            );
            let id = w.add(b);
            w.connect(id, root, false);
            let start = Point {
                x: case["x"].as_i64().unwrap() as i32,
                y: case["y"].as_i64().unwrap() as i32,
            };
            let run = lab_run(&data, l, &w, start, 270);
            let m = &run.motions[0];
            assert_eq!(
                m.kind != "lose",
                case["success"].as_bool().unwrap(),
                "{case}"
            );
            assert_eq!(m.to.x, case["endX"].as_i64().unwrap() as i32, "{case}");
            assert_eq!(m.to.y, case["endY"].as_i64().unwrap() as i32, "{case}");
            if case["success"] == true {
                assert_eq!(m.direction, case["endDirection"].as_i64().unwrap() as i32);
                assert!(
                    (m.duration * 1000. - case["duration"].as_f64().unwrap()).abs() < 1e-7,
                    "{case}: {}",
                    m.duration
                );
            }
        }
    }
    #[test]
    fn every_original_maze_can_be_completed_with_its_available_blocks() {
        let data = lab_data();
        for l in &data.levels[2..] {
            let start = Point {
                x: l.player_x,
                y: l.player_y,
            };
            let target = l.presents[0];
            let mut queue = std::collections::VecDeque::from([(start, Vec::<Block>::new())]);
            let mut seen = HashSet::new();
            let mut solution = None;
            while let Some((at, path)) = queue.pop_front() {
                if at == target {
                    solution = Some(path);
                    break;
                }
                if !seen.insert((at.x, at.y)) {
                    continue;
                }
                for d in ["North", "East", "South", "West"] {
                    for n in 0..=if l.tools.iter().any(|t| t == "controls_jump") {
                        3
                    } else {
                        0
                    } {
                        let mut b = Block::new(&if n == 0 {
                            format!("maze_move{d}")
                        } else {
                            "controls_jump".into()
                        });
                        b.fields.insert("LENGTH".into(), n.to_string());
                        b.fields.insert("DIRECTION".into(), d.to_lowercase());
                        let mut w = Workspace::default();
                        let root = w.add(Block::new("when_run"));
                        w.roots.push(root);
                        let id = w.add(b.clone());
                        w.connect(id, root, false);
                        let run = lab_run(&data, l, &w, at, 0);
                        if run.motions[0].kind != "lose" {
                            let mut next = path.clone();
                            next.push(b);
                            queue.push_back((run.motions[0].to, next));
                        }
                    }
                }
            }
            let blocks = solution.unwrap_or_else(|| panic!("Level {} has no solution", l.id));
            let mut w = Workspace::default();
            let root = w.add(Block::new("when_run"));
            w.roots.push(root);
            let mut parent = root;
            for b in blocks {
                let id = w.add(b);
                w.connect(id, parent, false);
                parent = id;
            }
            assert!(
                lab_run(&data, l, &w, start, 0).result.success,
                "level {}",
                l.id
            );
        }
    }
    #[test]
    fn both_jigsaws_require_exact_order_and_one_connected_stack() {
        for l in &lab_data().levels[..2] {
            let mut w = Workspace::level(l);
            assert!(!w.puzzle_success(l));
            let mut ordered: Vec<_> = w.roots.clone();
            ordered.reverse();
            for pair in ordered.windows(2) {
                w.detach(pair[1]);
                assert!(w.connect(pair[1], pair[0], false));
            }
            assert!(w.puzzle_success(l));
        }
    }
    #[test]
    fn graph_connections_cannot_cycle_and_repeats_execute() {
        let mut w = Workspace::default();
        let root = w.add(Block::new("when_run"));
        w.roots.push(root);
        let repeat = w.add(Block::new("controls_repeat"));
        w.connect(repeat, root, false);
        let step = w.add(Block::new("maze_moveNorth"));
        w.connect(step, repeat, true);
        assert!(!w.connect(root, step, false));
        assert_eq!(w.actions().unwrap().len(), 2);
        assert!(!w.empty());
        w.detach(step);
        assert!(w.empty());
        assert!(w.extra());
    }
}
