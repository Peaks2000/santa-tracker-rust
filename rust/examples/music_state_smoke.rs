//! CPU-only music decode/render smoke. Opens no device, window or audio context.
use santa_tracker::{
    audio::Sample,
    music_control::Spec,
    music_runtime::{Event, Runtime},
    music_transition::TRACKS,
};
use std::{path::PathBuf, sync::Arc};
fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Pass the original asset root containing audio/"),
    );
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../data/audio.json")).unwrap();
    let mut samples = vec![];
    let mut metadata = vec![];
    for id in TRACKS {
        let file = config["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["id"] == config["audio"][id]["file_id"])
            .unwrap();
        let sample = Sample::load(
            &root
                .join("audio")
                .join(format!("{}.ogg", file["url"].as_str().unwrap())),
        )
        .unwrap();
        metadata.push(serde_json::json!({"id":id,"rate":sample.rate,"frames":sample.frames(),"channels":sample.channels,"duration":sample.frames() as f64/sample.rate as f64}));
        samples.push(Arc::new(sample));
    }
    let mut runtime = Runtime::new(samples.try_into().ok().unwrap(), Spec::music().unwrap(), 0.);
    let events = [
        (0, Event::Village),
        (48000, Event::Game),
        (144000, Event::Hurry),
        (240000, Event::GameOver),
        (249600, Event::Game),
        (432000, Event::Lounge),
    ];
    let mut peak = 0f32;
    let mut power = 0f64;
    let mut nonzero = 0;
    for frame in 0..12 * 48000 {
        let now = frame as f64 / 48000.;
        if frame % 1024 == 0 {
            runtime.timers(now, now);
            runtime.collect_finished();
        }
        for &(at, event) in &events {
            if at == frame {
                runtime.event(now, now, event);
            }
        }
        if frame == 288000 {
            runtime.paused(now, true);
        }
        if frame == 384000 {
            runtime.paused(now, false);
        }
        if frame % 128 == 0 {
            runtime.begin_quantum(now);
        }
        let out = runtime.frame(now, 1. / 48000.);
        for sample in out {
            assert!(sample.is_finite());
            peak = peak.max(sample.abs());
            power += (sample as f64).powi(2);
            if sample != 0. {
                nonzero += 1;
            }
        }
    }
    assert!(nonzero > 0);
    assert!(peak > 0.);
    let report = serde_json::json!({"scope":"CPU-only native decode/render stability. Not original/browser PCM, audible, device, timer-task or whole-scene parity.","seconds":12,"output_frames":12*48000,"peak":peak,"rms":(power/(12.*48000.*2.)).sqrt(),"nonzero_samples":nonzero,"samples":metadata,"final_control":runtime.control});
    let text = serde_json::to_string_pretty(&report).unwrap() + "\n";
    if let Some(path) = std::env::args().nth(2) {
        std::fs::write(path, &text).unwrap();
    }
    println!("{{\"seconds\":12,\"peak\":{peak},\"nonzero_samples\":{nonzero}}}");
}
