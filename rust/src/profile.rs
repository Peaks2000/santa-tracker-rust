//! Opt-in workload/frame-interval measurements; never changes simulation rules.
use serde::Serialize;
use std::{path::Path, time::Instant};
#[derive(Serialize)]
pub struct Frame {
    pub interval_ms: f64,
    pub update_ms: f64,
    pub scene_ms: f64,
    pub ui_ms: f64,
    pub work_ms: f64,
}
pub struct Profile {
    started: Instant,
    pub frames: Vec<Frame>,
    warmup: u32,
    seen: u32,
}
impl Profile {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            frames: Vec::new(),
            warmup: 30,
            seen: 0,
        }
    }
    pub fn record(&mut self, f: Frame) {
        self.seen += 1;
        if self.seen > self.warmup {
            self.frames.push(f);
        }
    }
    pub fn save(&self, path: &Path, scene: &str, width: f32, height: f32) -> Result<(), String> {
        fn stats(v: impl Iterator<Item = f64>) -> serde_json::Value {
            let mut v: Vec<_> = v.filter(|v| v.is_finite()).collect();
            v.sort_by(f64::total_cmp);
            if v.is_empty() {
                return serde_json::json!({});
            }
            let p = |q: f64| v[((v.len() - 1) as f64 * q).round() as usize];
            serde_json::json!({"mean":v.iter().sum::<f64>()/v.len() as f64,"p50":p(0.5),"p95":p(0.95),"p99":p(0.99),"max":v[v.len()-1]})
        }
        let report = serde_json::json!({"scene":scene,"width":width,"height":height,"frames":self.frames.len(),"warmup_frames":self.warmup,"elapsed_seconds":self.started.elapsed().as_secs_f64(),"scope":"Main-thread work and observed presentation intervals; not input-to-photon latency. No gameplay timing changes.","interval_ms":stats(self.frames.iter().map(|f|f.interval_ms)),"update_ms":stats(self.frames.iter().map(|f|f.update_ms)),"scene_ms":stats(self.frames.iter().map(|f|f.scene_ms)),"ui_ms":stats(self.frames.iter().map(|f|f.ui_ms)),"work_ms":stats(self.frames.iter().map(|f|f.work_ms)),"intervals_over_25ms":self.frames.iter().filter(|f|f.interval_ms>25.).count(),"samples":self.frames});
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n")
            .map_err(|e| e.to_string())
    }
}
