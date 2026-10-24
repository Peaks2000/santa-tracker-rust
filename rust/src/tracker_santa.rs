//! santa-santa.js and its two independent six-second CSS bounce animations.
//! Original behavior/art: Copyright 2020 Google LLC, Apache-2.0.
use crate::ui::Ease;

pub const DIRECTIONS: [&str; 8] = ["n", "ne", "e", "se", "s", "sw", "w", "nw"];
pub fn direction(heading: f64) -> Option<&'static str> {
    if heading.is_nan() {
        return Some("s");
    }
    // Infinity is not NaN in JS; its remainder produces an undefined direction.
    if !heading.is_finite() {
        return None;
    }
    let normalized = (((heading + 22.5) % 360.) + 360.) % 360.;
    Some(DIRECTIONS[(normalized / 360. * 8.).floor() as usize])
}
pub fn chimney_mode(heading: f64) -> Option<usize> {
    heading
        .is_finite()
        .then(|| (heading.abs() % 2.).floor() as usize)
}
pub fn outer_size(viewport_width: f64) -> (f64, f64) {
    if viewport_width >= 768. {
        (102., 126.)
    } else {
        (68., 84.)
    }
}
/// Returns the middle Santa layer and the front/back layers' translations.
/// The source swaps the apparent animation names; keep the actual assignments.
pub fn bounce(elapsed_ms: f64) -> (f64, f64) {
    const TIMES: [f64; 10] = [0., 0.1, 0.2, 0.3, 0.4, 0.55, 0.65, 0.8, 0.9, 1.];
    const SANTA: [f64; 10] = [0., -3., 3., -2., 2., -6., 7., -3., 3., 0.];
    const OTHER: [f64; 10] = [0., -4., 4., -3., 3., -7., 8., -4., 4., 0.];
    let p = elapsed_ms.max(0.) % 6000. / 6000.;
    let i = TIMES.windows(2).position(|t| p < t[1]).unwrap_or(8);
    let t = Ease::InOut.at((p - TIMES[i]) / (TIMES[i + 1] - TIMES[i]));
    (
        SANTA[i] + (SANTA[i + 1] - SANTA[i]) * t,
        OTHER[i] + (OTHER[i + 1] - OTHER[i]) * t,
    )
}

/// display:none cancels the source CSS animations. Showing the sleigh again
/// starts new animation objects; changing direction alone leaves them alive.
#[derive(Default)]
pub struct SleighClock {
    stopped: bool,
    started_ms: f64,
}
impl SleighClock {
    pub fn elapsed(&mut self, now_ms: f64, stop: bool) -> f64 {
        if self.stopped && !stop {
            self.started_ms = now_ms;
        }
        self.stopped = stop;
        (now_ms - self.started_ms).max(0.)
    }
}

/// Fully composited original GIF frames. Decode once, advance by source delays;
/// transparent delta frames must never be treated as full replacement images.
pub struct Chimney {
    pub width: u16,
    pub height: u16,
    pub frames: Vec<Vec<u8>>,
    pub ends_ms: Vec<u64>,
}
impl Chimney {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut reader = options.read_info(bytes).map_err(|e| e.to_string())?;
        let (width, height) = (reader.width(), reader.height());
        let mut canvas = vec![0; width as usize * height as usize * 4];
        let mut frames = vec![];
        let mut ends_ms = vec![];
        let mut time = 0;
        while let Some(frame) = reader.read_next_frame().map_err(|e| e.to_string())? {
            let previous = (frame.dispose == gif::DisposalMethod::Previous).then(|| canvas.clone());
            for y in 0..frame.height as usize {
                for x in 0..frame.width as usize {
                    let (dx, dy) = (frame.left as usize + x, frame.top as usize + y);
                    if dx >= width as usize || dy >= height as usize {
                        return Err("GIF frame outside canvas".into());
                    }
                    let from = (y * frame.width as usize + x) * 4;
                    let to = (dy * width as usize + dx) * 4;
                    if frame.buffer[from + 3] != 0 {
                        canvas[to..to + 4].copy_from_slice(&frame.buffer[from..from + 4]);
                    }
                }
            }
            frames.push(canvas.clone());
            // The original chimney files use 80ms on every frame (no clamp).
            time += u64::from(frame.delay) * 10;
            ends_ms.push(time);
            match frame.dispose {
                gif::DisposalMethod::Background => {
                    for y in frame.top as usize..(frame.top as usize + frame.height as usize) {
                        let start = (y * width as usize + frame.left as usize) * 4;
                        canvas[start..start + frame.width as usize * 4].fill(0);
                    }
                }
                gif::DisposalMethod::Previous => canvas = previous.unwrap(),
                _ => {}
            }
        }
        if time == 0 {
            return Err("Empty or zero-duration chimney animation".into());
        }
        Ok(Self {
            width,
            height,
            frames,
            ends_ms,
        })
    }
    pub fn frame_at(&self, elapsed_ms: f64) -> usize {
        let time = elapsed_ms.max(0.) % *self.ends_ms.last().unwrap() as f64;
        self.ends_ms.partition_point(|&end| end as f64 <= time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_actual_browser_component() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/tracker-santa-reference.json")).unwrap();
        for row in data["cases"].as_array().unwrap() {
            if let Some(ms) = row["time"].as_f64() {
                let (a, b) = bounce(ms);
                assert!(
                    (a - row["santaY"].as_f64().unwrap()).abs() < 0.0001,
                    "{row}: {a}"
                );
                assert!(
                    (b - row["otherY"].as_f64().unwrap()).abs() < 0.0001,
                    "{row}: {b}"
                );
                let size = outer_size(row["width"].as_f64().unwrap());
                assert_eq!(
                    size,
                    (
                        row["size"][0].as_f64().unwrap(),
                        row["size"][1].as_f64().unwrap()
                    )
                );
            } else {
                let heading = row["heading"].as_f64().unwrap_or_else(|| {
                    if row["heading"] == "NaN" {
                        f64::NAN
                    } else {
                        f64::INFINITY
                    }
                });
                assert_eq!(
                    direction(heading).unwrap_or("undefined"),
                    row["dir"].as_str().unwrap()
                );
                let mode = chimney_mode(heading)
                    .map_or("presents modeNaN".into(), |m| format!("presents mode{m}"));
                assert_eq!(mode, row["mode"].as_str().unwrap());
            }
        }
    }
    #[test]
    fn leaving_a_stop_starts_new_bounce_animations() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/tracker-santa-lifecycle.json")).unwrap();
        assert_eq!(data["result"]["hiddenCount"], 0);
        assert_eq!(data["result"]["freshCount"], 3);
        assert_eq!(data["result"]["newInstances"], true);
        let mut clock = SleighClock::default();
        assert_eq!(clock.elapsed(1230., false), 1230.);
        clock.elapsed(1300., true);
        assert_eq!(clock.elapsed(4000., false), 0.);
        assert_eq!(clock.elapsed(4300., false), 300.);
    }
    #[test]
    fn original_gifs_preserve_every_delta_frame_and_delay() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/tracker-chimney-reference.json")).unwrap();
        for mode in 0..2 {
            let bytes = std::fs::read(format!(
                "{}/../static/img/sleigh/chimney{mode}.gif",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap();
            let gif = Chimney::decode(&bytes).unwrap();
            assert_eq!((gif.width, gif.height), (383, 333));
            assert_eq!(gif.frames.len(), 36);
            for (i, frame) in gif.frames.iter().enumerate() {
                // Compare visible RGBA; transparent RGB is unspecified by GIF.
                let hash = frame
                    .chunks_exact(4)
                    .flat_map(|p| {
                        if p[3] == 0 {
                            [0; 4]
                        } else {
                            [p[0], p[1], p[2], p[3]]
                        }
                    })
                    .fold(0xcbf29ce484222325u64, |h, b| {
                        (h ^ b as u64).wrapping_mul(0x100000001b3)
                    });
                assert_eq!(
                    format!("{hash:016x}"),
                    data[mode]["hashes"][i].as_str().unwrap(),
                    "mode {mode} frame {i}"
                );
                assert_eq!(gif.frame_at((i * 80) as f64), i);
                assert_eq!(gif.frame_at((i * 80 + 79) as f64), i);
            }
            assert_eq!(gif.frame_at(2880.), 0);
        }
    }
}
