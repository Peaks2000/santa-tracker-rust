//! Source score CSS animation and reused splash CSS transitions.
use crate::ui::{Ease, Transition};
#[derive(Clone, Copy, Debug)]
pub struct ScoreEffect {
    pub x: f64,
    pub y: f64,
    pub value: u64,
    start: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct ScoreVisual {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub opacity: f64,
    pub value: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Ring {
    pub x: f64,
    pub y: f64,
    pub diameter: f64,
    pub border: f64,
    pub scale: f64,
}
#[derive(Debug)]
pub struct Effects {
    pub score: Option<ScoreEffect>,
    pub wall_time: f64,
    pub animation_time: f64,
    splash_position: Option<(f64, f64)>,
    splash_visible: bool,
    splash_progress: Transition<1>,
}
impl Default for Effects {
    fn default() -> Self {
        Self {
            score: None,
            wall_time: 0.,
            animation_time: 0.,
            splash_position: None,
            splash_visible: false,
            splash_progress: Transition::new([0.]),
        }
    }
}
impl Effects {
    pub fn advance(&mut self, dt: f64, playing: bool) {
        self.wall_time += dt;
        if playing {
            self.animation_time += dt;
        }
        self.settle();
    }
    fn settle(&mut self) {
        if self
            .score
            .is_some_and(|s| self.animation_time - s.start >= 0.4 - 1e-12)
        {
            self.score = None;
        }
        let end = self.splash_progress.end_time();
        if self.splash_visible && self.wall_time >= end - 1e-12 {
            self.splash_visible = false;
            // Removing .animate starts the reverse transition under .hidden.
            self.splash_progress.set(end, [0.], 0.5, Ease::Default);
        }
    }
    pub fn hit(&mut self, x: f64, y: f64, value: u64) {
        self.settle();
        // Adding an already present CSS class changes text/position, not its clock.
        let start = self.score.map(|s| s.start).unwrap_or(self.animation_time);
        self.score = Some(ScoreEffect { x, y, value, start });
    }
    pub fn miss(&mut self, x: f64, y: f64) {
        self.settle();
        self.splash_position = Some((x, y));
        self.splash_visible = true;
        self.splash_progress
            .set(self.wall_time, [1.], 0.5, Ease::Default);
    }
    pub fn score_visual(&self, width: f64) -> Option<ScoreVisual> {
        let s = self.score?;
        let age = self.animation_time - s.start;
        if age >= 0.4 {
            return None;
        }
        let scale = 0.5 + 0.5 * Ease::Default.at(age / 0.4);
        let opacity = if age <= 0.2 {
            1.
        } else {
            1. - Ease::Default.at((age - 0.2) / 0.2)
        };
        // CSS normal Open Sans 48px line box: 51px ascent + 14px descent.
        // Layout lengths are truncated to 1/64px before transforms.
        let margin = (-0.7_f64 * 48. * 64.).trunc() / 64.;
        Some(ScoreVisual {
            x: s.x - 20. + width * 0.5 * (1. - scale),
            y: s.y + margin + 65. * 0.7 * (1. - scale),
            scale,
            opacity,
            value: s.value,
        })
    }
    pub fn rings(&self) -> Option<[Ring; 4]> {
        if !self.splash_visible {
            return None;
        }
        let (x, y) = self.splash_position?;
        let t = self.splash_progress.value(self.wall_time)[0];
        Some(std::array::from_fn(|i| {
            let start = [8., 24., 36., 46.][i];
            let end = [18., 30., 40., 48.][i];
            let content = ((start + (end - start) * t) * 64. + 1e-8).floor() / 64.;
            let b = [5., 3., 2., 1.][i] * (1. - t);
            // CSS border widths are integer CSS pixels and remain at least 1px
            // until the transition reaches zero. This moves transform-origin.
            let border = if b > 1e-10 { b.floor().max(1.) } else { 0. };
            let diameter = content + 2. * border;
            let offset = [15., 9., 4., 0.][i];
            let scale = 1. + 0.3 * t;
            Ring {
                x: x + offset + diameter / 2.,
                y: y + offset + diameter / 2.,
                diameter,
                border,
                scale,
            }
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_css_score_and_splash_phases() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/boatload-effects-reference.json")).unwrap();
        let close =
            |a: f64, b: f64, label: &str| assert!((a - b).abs() < 0.003, "{label}: {a} vs {b}");
        for c in fixture["cases"].as_array().unwrap() {
            let mut e = Effects::default();
            e.hit(700., 400., 100);
            e.miss(600., 350.);
            e.advance(c["time"].as_f64().unwrap() / 1000., true);
            let scale = c["scale"].as_f64().unwrap();
            let base = c["base"].as_f64().unwrap();
            if let Some(score) = e.score_visual(84.375) {
                let r = &c["score"]["rect"];
                close(base + score.x * scale, r["x"].as_f64().unwrap(), "score x");
                close(score.y * scale, r["y"].as_f64().unwrap(), "score y");
                close(
                    84.375 * score.scale * scale,
                    r["w"].as_f64().unwrap(),
                    "score w",
                );
                close(
                    65. * score.scale * scale,
                    r["h"].as_f64().unwrap(),
                    "score h",
                );
                close(
                    score.opacity,
                    c["score"]["opacity"].as_f64().unwrap(),
                    "opacity",
                );
            } else {
                assert!(c["score"]["rect"]["y"].as_f64().unwrap() > 1000.);
            }
            for (i, ring) in e.rings().unwrap().iter().enumerate() {
                let r = &c["rings"][i]["rect"];
                let d = ring.diameter * ring.scale;
                close(
                    base + (ring.x - d / 2.) * scale,
                    r["x"].as_f64().unwrap(),
                    "ring x",
                );
                close(
                    (ring.y - d / 2.) * scale,
                    r["y"].as_f64().unwrap(),
                    "ring y",
                );
                close(d * scale, r["w"].as_f64().unwrap(), "ring w");
                close(
                    ring.border,
                    c["rings"][i]["border"]
                        .as_str()
                        .unwrap()
                        .trim_end_matches("px")
                        .parse()
                        .unwrap(),
                    "ring border",
                );
            }
        }
    }
    #[test]
    fn original_reused_effect_lifecycle_and_pause() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/boatload-effect-lifecycle.json")).unwrap();
        assert_eq!(fixture["pauseStates"]["score"], "paused");
        let mut game = crate::boatload::Boatload::new("test1234", 1440., 800.);
        let close =
            |a: f64, b: f64, label: &str| assert!((a - b).abs() < 0.003, "{label}: {a} vs {b}");
        for row in fixture["rows"].as_array().unwrap() {
            match row["action"].as_str().unwrap() {
                "trigger" => {
                    game.effects.hit(700., 400., 100);
                    game.effects.miss(600., 350.);
                }
                "150ms" => game.wall_step(0.15),
                "reuse" => {
                    game.effects.hit(720., 410., 200);
                    game.effects.miss(610., 360.);
                }
                "paused500ms" => {
                    game.paused = true;
                    game.wall_step(0.35);
                }
                "reverse700ms" => {
                    game.wall_step(0.2);
                    game.effects.miss(620., 370.);
                    for duration in row["durations"].as_array().unwrap() {
                        close(
                            (game.effects.splash_progress.end_time() - game.effects.wall_time)
                                * 1000.,
                            duration.as_f64().unwrap(),
                            "reversal duration",
                        );
                    }
                }
                "reverse800ms" => game.wall_step(0.1),
                "paused1050ms" => game.wall_step(0.25),
                "restart" => {
                    game.restart();
                    assert!(game.playing && !game.paused);
                    assert_eq!(row["scorePlayState"], "running");
                }
                x => panic!("unexpected action {x}"),
            }
            let state = &row["state"];
            let score = game.effects.score_visual(84.375).unwrap();
            let r = &state["score"]["rect"];
            close(164.796875 + score.x, r["x"].as_f64().unwrap(), "score x");
            close(score.y, r["y"].as_f64().unwrap(), "score y");
            close(
                score.opacity,
                state["score"]["opacity"].as_f64().unwrap(),
                "score opacity",
            );
            assert_eq!(
                game.effects.rings().is_none(),
                state["splashHidden"].as_bool().unwrap()
            );
            if let Some(rings) = game.effects.rings() {
                for (i, ring) in rings.iter().enumerate() {
                    let r = &state["rings"][i];
                    let d = ring.diameter * ring.scale;
                    close(
                        164.796875 + ring.x - d / 2.,
                        r["rect"]["x"].as_f64().unwrap(),
                        "ring x",
                    );
                    close(ring.y - d / 2., r["rect"]["y"].as_f64().unwrap(), "ring y");
                    close(d, r["rect"]["w"].as_f64().unwrap(), "ring diameter");
                    close(ring.border, r["border"].as_f64().unwrap(), "border");
                }
            }
        }
    }
    #[test]
    fn effect_clocks_pause_reuse_restart_and_reverse() {
        let mut e = Effects::default();
        e.hit(700., 400., 100);
        e.miss(600., 350.);
        e.advance(0.15, true);
        let before = e.score_visual(84.375).unwrap();
        e.hit(720., 410., 200);
        e.miss(610., 360.);
        let after = e.score_visual(84.375).unwrap();
        assert_eq!(after.scale, before.scale);
        assert_eq!(after.opacity, before.opacity);
        assert_eq!(after.value, 200);
        e.advance(0.35, false);
        assert!(e.rings().is_none());
        assert_eq!(e.score_visual(84.375).unwrap().scale, after.scale);
        e.advance(0.2, false);
        e.miss(600., 350.);
        let returning = e.rings().unwrap();
        assert!(returning[0].scale > 1. && returning[0].scale < 1.3);
        e.advance(0.35, false);
        assert!(e.rings().is_none());
        assert!(e.score.is_some());
        e.advance(0.25, true);
        assert!(e.score.is_none());
    }
}
