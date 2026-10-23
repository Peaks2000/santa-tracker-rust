//! santa-interlude.js/scss: four staggered layers and the original loader art.
use macroquad::prelude::*;
use santa_tracker::ui::Ease;
pub struct Interlude {
    start: Option<f64>,
    out: Option<f64>,
    target: Option<String>,
}
impl Interlude {
    pub fn new() -> Self {
        Self {
            start: None,
            out: None,
            target: None,
        }
    }
    pub fn busy(&self) -> bool {
        self.start.is_some()
    }
    pub fn begin(&mut self, target: String) -> bool {
        if self.busy() {
            return false;
        }
        self.start = Some(get_time());
        self.out = None;
        self.target = Some(target);
        true
    }
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
    pub fn ready(&mut self) -> Option<String> {
        if self.out.is_none() && self.start.is_some_and(|start| get_time() - start >= 1.25) {
            self.target.take()
        } else {
            None
        }
    }
    pub fn hide(&mut self) {
        self.out = Some(get_time());
    }
    pub fn layers(&mut self) -> Option<[(f32, Option<Color>); 4]> {
        let start = self.start?;
        let now = get_time();
        if self.out.is_some_and(|out| now - out >= 1.25) {
            self.start = None;
            self.out = None;
            return None;
        }
        Some(std::array::from_fn(|i| {
            let y = if let Some(out) = self.out {
                -screen_height()
                    * Ease::InterludeOut.at((now - out - (1. - i as f64 / 3.) * 0.5) / 0.75) as f32
            } else {
                screen_height()
                    * (1. - Ease::Sidebar.at((now - start - i as f64 / 3. * 0.5) / 0.75) as f32)
            };
            (
                y,
                match i {
                    0 => Some(Color::from_hex(0x6bb4fd)),
                    1 => Some(Color::from_hex(0xff3333)),
                    2 => Some(Color::from_hex(0x1a844b)),
                    _ => None,
                },
            )
        }))
    }
    pub fn animation_ratio(&self, duration: f32) -> f32 {
        self.start
            .map(|start| ((get_time() - start) as f32 / duration).fract())
            .unwrap_or(0.)
    }
}
