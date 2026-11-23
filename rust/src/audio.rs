//! Ogg decoding, resampling and mixing for the native application.
use std::{
    path::Path,
    sync::{mpsc::Receiver, Arc},
};

pub struct Sample {
    pub samples: Vec<f32>,
    pub channels: usize,
    pub rate: u32,
}
impl Sample {
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut ogg = lewton::inside_ogg::OggStreamReader::new(file).map_err(|e| e.to_string())?;
        let channels = ogg.ident_hdr.audio_channels as usize;
        let rate = ogg.ident_hdr.audio_sample_rate;
        if channels == 0 || channels > 2 {
            return Err("Expected mono or stereo original audio".into());
        }
        let mut samples = Vec::new();
        while let Some(packet) = ogg.read_dec_packet_itl().map_err(|e| e.to_string())? {
            samples.extend(packet.into_iter().map(|s| s as f32 / 32768.));
        }
        if samples.is_empty() {
            return Err("Empty audio sample".into());
        }
        Ok(Self {
            samples,
            channels,
            rate,
        })
    }
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels
    }
}
#[derive(Clone, Copy, PartialEq)]
pub enum PlayPolicy {
    Retrigger,
    IfAbsent,
    FadeInLoop,
}
#[derive(Clone, Copy)]
struct LoopEnvelope {
    from: f32,
    to: f32,
    elapsed: usize,
    total: usize,
    stopping: bool,
}
impl LoopEnvelope {
    fn level(self) -> f32 {
        if self.total == 0 {
            return self.to;
        }
        self.from + (self.to - self.from) * (self.elapsed as f32 / self.total as f32).min(1.)
    }
}
pub enum Command {
    MusicLoad {
        samples: [Arc<Sample>; 5],
        specs: [crate::music_control::Spec; 5],
    },
    MusicTransition {
        event: crate::music_runtime::Event,
    },
    Play {
        pausable: bool,
        id: String,
        policy: PlayPolicy,
        delay: f64,
        fade_in: f64,
        sample: Arc<Sample>,
        volume: f32,
        rate: f64,
        loop_start: Option<f64>,
        offset: f64,
        end: Option<f64>,
    },
    GroupBusGain {
        ids: Vec<String>,
        gain: f32,
    },
    GroupPanSource {
        ids: Vec<String>,
        pan: f32,
    },
    SourceGain {
        id: String,
        volume: f32,
    },
    PanSource {
        id: String,
        pan: f32,
    },
    StopSource {
        id: String,
        fade: f64,
    },
    Stop,
    Paused(bool),
    Gain(f32),
}
struct Voice {
    pausable: bool,
    id: String,
    pan: Option<f32>,
    delay: usize,
    age: usize,
    fade_in: usize,
    fade_out: Option<(usize, usize)>,
    loop_envelope: Option<LoopEnvelope>,
    sample: Arc<Sample>,
    position: f64,
    step: f64,
    volume: f32,
    bus_gain: f32,
    loop_start: Option<f64>,
    end: f64,
}
pub struct Mixer {
    commands: Receiver<Command>,
    voices: Vec<Voice>,
    paused: bool,
    gain: f32,
    pans: std::collections::HashMap<String, f32>,
    bus_gains: std::collections::HashMap<String, f32>,
    music: Option<crate::music_runtime::Runtime>,
    rendered_frames: u64,
    wall_origin: std::time::Instant,
}
impl Mixer {
    pub fn new(commands: Receiver<Command>) -> Self {
        Self {
            commands,
            voices: Vec::with_capacity(64),
            paused: false,
            gain: 1.,
            pans: std::collections::HashMap::new(),
            bus_gains: std::collections::HashMap::new(),
            music: None,
            rendered_frames: 0,
            wall_origin: std::time::Instant::now(),
        }
    }
    pub fn mix(&mut self, output: &mut [f32]) {
        let now = self.rendered_frames as f64 / 48000.;
        let wall = self.wall_origin.elapsed().as_secs_f64();
        if let Some(music) = &mut self.music {
            music.timers(now, wall);
        }
        for command in self.commands.try_iter() {
            match command {
                Command::MusicLoad { samples, specs } => {
                    if self.music.is_none() {
                        self.music = Some(crate::music_runtime::Runtime::new(samples, specs, now));
                    }
                }
                Command::MusicTransition { event } => {
                    if let Some(music) = &mut self.music {
                        music.event(now, wall, event);
                    }
                }
                Command::GroupBusGain { ids, gain } => {
                    for id in ids {
                        for voice in self.voices.iter_mut().filter(|v| v.id == id) {
                            voice.bus_gain = gain;
                        }
                        self.bus_gains.insert(id, gain);
                    }
                }
                Command::GroupPanSource { ids, pan } => {
                    let pan = pan.clamp(-1., 1.);
                    for id in ids {
                        for voice in self.voices.iter_mut().filter(|v| v.id == id) {
                            voice.pan = Some(pan);
                        }
                        self.pans.insert(id, pan);
                    }
                }
                Command::SourceGain { id, volume } => {
                    for v in self.voices.iter_mut().filter(|v| v.id == id) {
                        v.volume = volume;
                    }
                }
                Command::Stop => {
                    self.voices.clear();
                    if let Some(music) = &mut self.music {
                        music.stop_all(now, wall);
                    }
                }
                Command::PanSource { id, pan } => {
                    let pan = pan.clamp(-1., 1.);
                    for voice in self.voices.iter_mut().filter(|v| v.id == id) {
                        voice.pan = Some(pan);
                    }
                    self.pans.insert(id, pan);
                }
                Command::StopSource { id, fade } => {
                    for voice in self.voices.iter_mut().filter(|v| v.id == id) {
                        let frames = (fade.max(0.) * 48000.) as usize;
                        if let Some(envelope) = &mut voice.loop_envelope {
                            let level = envelope.level();
                            *envelope = LoopEnvelope {
                                from: if level == 0. { 1. } else { level },
                                to: 0.,
                                elapsed: 0,
                                total: if voice.age == 0 { 0 } else { frames },
                                stopping: true,
                            };
                        } else {
                            voice.fade_out = Some((frames, frames));
                        }
                    }
                }
                Command::Paused(p) => {
                    self.paused = p;
                    if let Some(music) = &mut self.music {
                        music.paused(now, p);
                    }
                }
                Command::Gain(g) => self.gain = g,
                Command::Play {
                    pausable,
                    id,
                    policy,
                    delay,
                    fade_in,
                    sample,
                    volume,
                    rate,
                    loop_start,
                    offset,
                    end,
                } => {
                    if policy == PlayPolicy::FadeInLoop && loop_start.is_some() {
                        if let Some(voice) = self
                            .voices
                            .iter_mut()
                            .find(|v| v.id == id && v.loop_start.is_some() && v.delay == 0)
                        {
                            if let Some(envelope) = &mut voice.loop_envelope {
                                if envelope.stopping {
                                    *envelope = LoopEnvelope {
                                        from: envelope.level(),
                                        to: 1.,
                                        elapsed: 0,
                                        total: (fade_in.max(0.) * 48000.) as usize,
                                        stopping: false,
                                    };
                                }
                            } else if let Some((remaining, total)) = voice.fade_out {
                                let fade_in_level = if voice.fade_in > 0 {
                                    (voice.age as f32 / voice.fade_in as f32).min(1.)
                                } else {
                                    1.
                                };
                                voice.loop_envelope = Some(LoopEnvelope {
                                    from: fade_in_level * remaining as f32 / total.max(1) as f32,
                                    to: 1.,
                                    elapsed: 0,
                                    total: (fade_in.max(0.) * 48000.) as usize,
                                    stopping: false,
                                });
                                voice.fade_out = None;
                            }
                            // Active loop: original fadeInAndPlay returns without
                            // resetting phase, gain automation or sample.
                            continue;
                        }
                    }
                    if policy == PlayPolicy::IfAbsent
                        && self
                            .voices
                            .iter()
                            .any(|v| v.id == id && v.fade_out != Some((0, 0)))
                    {
                        continue;
                    }
                    let sr = sample.rate as f64;
                    let max = sample.frames() as f64;
                    let end = end.map(|v| v * sr).unwrap_or(max).min(max);
                    let position = (offset * sr).clamp(0., end);
                    let loop_start = loop_start.map(|p| (p * sr).clamp(0., (end - 1.).max(0.)));
                    self.voices.push(Voice {
                        pausable,
                        pan: self.pans.get(&id).copied(),
                        bus_gain: self.bus_gains.get(&id).copied().unwrap_or(1.),
                        id,
                        delay: (delay.max(0.) * 48000.) as usize,
                        age: 0,
                        fade_in: (fade_in.max(0.) * 48000.) as usize,
                        fade_out: None,
                        loop_envelope: if policy == PlayPolicy::FadeInLoop {
                            Some(LoopEnvelope {
                                from: 0.,
                                to: 1.,
                                elapsed: 0,
                                total: (fade_in.max(0.) * 48000.) as usize,
                                stopping: false,
                            })
                        } else {
                            None
                        },
                        step: sr / 48000. * rate,
                        sample,
                        position,
                        volume,
                        loop_start,
                        end,
                    });
                }
            }
        }
        output.fill(0.);
        for frame in output.chunks_exact_mut(2) {
            if let Some(music) = &mut self.music {
                let now = self.rendered_frames as f64 / 48000.;
                if self.rendered_frames % 128 == 0 {
                    music.begin_quantum(now);
                }
                let stereo = music.frame(now, 1. / 48000.);
                frame[0] += stereo[0] * self.gain;
                frame[1] += stereo[1] * self.gain;
            }
            self.rendered_frames += 1;
            for voice in &mut self.voices {
                if self.paused && voice.pausable {
                    continue;
                }
                if voice.delay > 0 {
                    voice.delay -= 1;
                    continue;
                }
                let mut envelope = if let Some(ramp) = &mut voice.loop_envelope {
                    let level = ramp.level();
                    ramp.elapsed = ramp.elapsed.saturating_add(1);
                    level
                } else if voice.fade_in > 0 {
                    (voice.age as f32 / voice.fade_in as f32).min(1.)
                } else {
                    1.
                };
                if let Some((remaining, total)) = &mut voice.fade_out {
                    if *remaining == 0 {
                        continue;
                    }
                    envelope *= *remaining as f32 / (*total).max(1) as f32;
                    *remaining -= 1;
                }
                voice.age += 1;
                if voice.position >= voice.end {
                    if let Some(start) = voice.loop_start {
                        let length = voice.end - start;
                        if length > 0. {
                            voice.position =
                                start + (voice.position - voice.end).rem_euclid(length);
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                let index = voice.position.floor() as usize;
                let fraction = voice.position.fract() as f32;
                let next = (index + 1).min(voice.sample.frames() - 1);
                let mut channels = [0.; 2];
                for (channel, out) in channels.iter_mut().enumerate() {
                    let c = channel.min(voice.sample.channels - 1);
                    let a = voice.sample.samples[index * voice.sample.channels + c];
                    let b = voice.sample.samples[next * voice.sample.channels + c];
                    *out = a + (b - a) * fraction;
                }
                if let Some(pan) = voice.pan {
                    if voice.sample.channels == 1 {
                        let angle = (pan + 1.) * std::f32::consts::FRAC_PI_4;
                        channels = [channels[0] * angle.cos(), channels[0] * angle.sin()];
                    } else if pan <= 0. {
                        let angle = (pan + 1.) * std::f32::consts::FRAC_PI_2;
                        channels = [
                            channels[0] + channels[1] * angle.cos(),
                            channels[1] * angle.sin(),
                        ];
                    } else {
                        let angle = pan * std::f32::consts::FRAC_PI_2;
                        channels = [
                            channels[0] * angle.cos(),
                            channels[1] + channels[0] * angle.sin(),
                        ];
                    }
                }
                for (out, sample) in frame.iter_mut().zip(channels) {
                    *out += sample * voice.volume * voice.bus_gain * self.gain * envelope;
                }
                voice.position += voice.step;
            }
        }
        if let Some(music) = &mut self.music {
            music.collect_finished();
        }
        self.voices.retain(|v| {
            !matches!(v.fade_out, Some((0, _)))
                && v.loop_envelope
                    .is_none_or(|e| !e.stopping || e.elapsed < e.total)
                && (v.loop_start.is_some() || v.position < v.end)
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gameover_music_transition_keeps_unrelated_game_voices() {
        use crate::{music_control::Spec, music_runtime::Event};
        let (tx, rx) = std::sync::mpsc::channel();
        let mut mixer = Mixer::new(rx);
        let sample = Arc::new(Sample {
            samples: vec![0.1; 48000],
            channels: 1,
            rate: 48000,
        });
        tx.send(Command::Play {
            pausable: true,
            id: "jetpack-ambience".into(),
            policy: PlayPolicy::Retrigger,
            delay: 0.,
            fade_in: 0.,
            sample: sample.clone(),
            volume: 1.,
            rate: 1.,
            loop_start: Some(0.),
            offset: 0.,
            end: None,
        })
        .unwrap();
        tx.send(Command::MusicLoad {
            samples: std::array::from_fn(|_| sample.clone()),
            specs: std::array::from_fn(|i| Spec {
                volume: 1.,
                rate: 1.,
                offset: 0.,
                duration: 0.,
                looping: i != 4,
                retrigger: i == 4,
                loop_start: 0.,
                loop_end: 0.,
                max_sources: -1,
            }),
        })
        .unwrap();
        tx.send(Command::MusicTransition { event: Event::Game })
            .unwrap();
        mixer.mix(&mut vec![0.; 960]);
        let phase = mixer.voices[0].position;
        tx.send(Command::MusicTransition {
            event: Event::GameOver,
        })
        .unwrap();
        mixer.mix(&mut []);
        assert_eq!(mixer.voices.len(), 1);
        assert_eq!(mixer.voices[0].position, phase);
        mixer.mix(&mut vec![0.; 960]);
        assert!(mixer.voices[0].position > phase);
        tx.send(Command::Stop).unwrap();
        mixer.mix(&mut []);
        assert!(mixer.voices.is_empty());
    }
    #[test]
    fn original_jetpack_loop_reentry_decisions_preserve_phase() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/jetpack-loop-reference.json")).unwrap();
        for case in data["cases"].as_array().unwrap() {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut mixer = Mixer::new(rx);
            let id = case["id"].as_str().unwrap();
            let volume = case["volume"].as_f64().unwrap() as f32;
            let play = || Command::Play {
                pausable: true,
                id: id.into(),
                policy: PlayPolicy::FadeInLoop,
                delay: 0.,
                fade_in: 0.3,
                sample: Arc::new(Sample {
                    samples: vec![0.1; 32],
                    channels: 2,
                    rate: 48000,
                }),
                volume,
                rate: 1.,
                loop_start: Some(0.),
                offset: 0.,
                end: None,
            };
            if case["present"] == true {
                tx.send(play()).unwrap();
                mixer.mix(&mut []);
                let voice = &mut mixer.voices[0];
                voice.position = 1.5;
                voice.age = 5;
                voice.loop_envelope = Some(LoopEnvelope {
                    from: case["initial_gain"].as_f64().unwrap() as f32 / volume,
                    to: if case["stopping"] == true { 0. } else { 1. },
                    elapsed: 0,
                    total: 48000,
                    stopping: case["stopping"].as_bool().unwrap(),
                });
            }
            tx.send(play()).unwrap();
            mixer.mix(&mut []);
            assert_eq!(mixer.voices.len(), 1, "{case}");
            let voice = &mixer.voices[0];
            let envelope = voice.loop_envelope.unwrap();
            assert_eq!(
                voice.position,
                if case["present"] == true { 1.5 } else { 0. },
                "{case}"
            );
            let ops = case["ops"].as_array().unwrap();
            if ops.is_empty() {
                assert_eq!(envelope.total, 48000, "active loop automation changed");
            } else {
                assert!(!envelope.stopping);
                let set = ops.iter().find(|op| op[0] == "set").unwrap();
                let ramp = ops.iter().find(|op| op[0] == "ramp").unwrap();
                assert!(
                    (envelope.from * volume - set[1].as_f64().unwrap() as f32).abs() < 1e-6,
                    "{case}"
                );
                assert!(
                    (envelope.to * volume - ramp[1].as_f64().unwrap() as f32).abs() < 1e-6,
                    "{case}"
                );
                assert_eq!(envelope.total, 14400);
                assert_eq!(
                    ops.iter().any(|op| op[0] == "play"),
                    case["present"] == false
                );
            }
        }
    }
    #[test]
    fn loop_restart_keeps_phase_and_reverses_fade_without_duplication() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut mixer = Mixer::new(rx);
        let play = || Command::Play {
            pausable: true,
            id: "loop".into(),
            policy: PlayPolicy::FadeInLoop,
            delay: 0.,
            fade_in: 4. / 48000.,
            sample: Arc::new(Sample {
                samples: vec![1.; 64],
                channels: 2,
                rate: 48000,
            }),
            volume: 1.,
            rate: 1.,
            loop_start: Some(0.),
            offset: 0.,
            end: None,
        };
        tx.send(play()).unwrap();
        let mut first = [0.; 4];
        mixer.mix(&mut first);
        assert_eq!(first, [0., 0., 0.25, 0.25]);
        tx.send(play()).unwrap();
        let mut one = [0.; 2];
        mixer.mix(&mut one);
        assert_eq!(one, [0.5, 0.5]);
        assert_eq!(mixer.voices.len(), 1);
        tx.send(Command::StopSource {
            id: "loop".into(),
            fade: 4. / 48000.,
        })
        .unwrap();
        let mut fading = [0.; 4];
        mixer.mix(&mut fading);
        assert_eq!(fading, [0.75, 0.75, 0.5625, 0.5625]);
        tx.send(play()).unwrap();
        let mut recovering = [0.; 8];
        mixer.mix(&mut recovering);
        assert_eq!(
            recovering,
            [0.375, 0.375, 0.53125, 0.53125, 0.6875, 0.6875, 0.84375, 0.84375]
        );
        assert_eq!(mixer.voices.len(), 1);
        assert_eq!(mixer.voices[0].position, 9.);
        tx.send(Command::StopSource {
            id: "loop".into(),
            fade: 2. / 48000.,
        })
        .unwrap();
        let mut tail = [0.; 4];
        mixer.mix(&mut tail);
        assert_eq!(tail, [1., 1., 0.5, 0.5]);
        assert!(mixer.voices.is_empty());
        // Source fadeOutAndStop stops immediately when when<=startTime.
        tx.send(play()).unwrap();
        tx.send(Command::StopSource {
            id: "loop".into(),
            fade: 0.3,
        })
        .unwrap();
        mixer.mix(&mut one);
        assert_eq!(one, [0., 0.]);
        assert!(mixer.voices.is_empty());
    }

    #[test]
    fn shared_bus_gain_and_pan_preserve_stem_levels_and_playback_position() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut mixer = Mixer::new(rx);
        let ids = vec!["wind".to_owned(), "jet".to_owned()];
        tx.send(Command::GroupBusGain {
            ids: ids.clone(),
            gain: 0.5,
        })
        .unwrap();
        let sample = Arc::new(Sample {
            samples: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            channels: 2,
            rate: 48000,
        });
        let play = |id: &str, volume| Command::Play {
            pausable: true,
            id: id.into(),
            policy: PlayPolicy::Retrigger,
            delay: 0.,
            fade_in: 0.,
            sample: sample.clone(),
            volume,
            rate: 1.,
            loop_start: Some(0.),
            offset: 0.,
            end: None,
        };
        tx.send(play("wind", 0.2)).unwrap();
        tx.send(play("jet", 0.4)).unwrap();
        let mut output = [0.; 2];
        mixer.mix(&mut output);
        assert!((output[0] - 0.03).abs() < 1e-6 && (output[1] - 0.06).abs() < 1e-6);
        tx.send(Command::GroupBusGain {
            ids: ids.clone(),
            gain: 1.5,
        })
        .unwrap();
        tx.send(Command::GroupPanSource {
            ids: ids.clone(),
            pan: 0.75,
        })
        .unwrap();
        mixer.mix(&mut output);
        let angle = 0.75f32 * std::f32::consts::FRAC_PI_2;
        assert!((output[0] - 0.3 * angle.cos() * 0.6 * 1.5).abs() < 1e-6);
        assert!((output[1] - (0.4 + 0.3 * angle.sin()) * 0.6 * 1.5).abs() < 1e-6);
        // Independent source volume remains independent of shared bus gain.
        tx.send(Command::SourceGain {
            id: "wind".into(),
            volume: 0.8,
        })
        .unwrap();
        mixer.mix(&mut output);
        assert!((output[0] - 0.5 * angle.cos() * 1.2 * 1.5).abs() < 1e-6);
        assert!((output[1] - (0.6 + 0.5 * angle.sin()) * 1.2 * 1.5).abs() < 1e-6);
        // Bus state persists when voices stop/restart, like the original bus.
        tx.send(Command::Stop).unwrap();
        tx.send(play("wind", 0.2)).unwrap();
        tx.send(play("jet", 0.4)).unwrap();
        mixer.mix(&mut output);
        assert!((output[0] - 0.1 * angle.cos() * 0.6 * 1.5).abs() < 1e-6);
        assert!((output[1] - (0.2 + 0.1 * angle.sin()) * 0.6 * 1.5).abs() < 1e-6);
    }

    #[test]
    fn stereo_pan_moves_audio_without_advancing_or_restarting_it() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut mixer = Mixer::new(rx);
        tx.send(Command::PanSource {
            id: "sled".into(),
            pan: -1.,
        })
        .unwrap();
        tx.send(Command::Play {
            pausable: true,
            id: "sled".into(),
            policy: PlayPolicy::IfAbsent,
            delay: 0.,
            fade_in: 0.,
            sample: Arc::new(Sample {
                samples: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
                channels: 2,
                rate: 48000,
            }),
            volume: 1.,
            rate: 1.,
            loop_start: None,
            offset: 0.,
            end: None,
        })
        .unwrap();
        let mut frame = [0.; 2];
        mixer.mix(&mut frame);
        assert!((frame[0] - 0.3).abs() < 1e-6 && frame[1].abs() < 1e-6);
        tx.send(Command::PanSource {
            id: "sled".into(),
            pan: 1.,
        })
        .unwrap();
        mixer.mix(&mut frame);
        assert!(frame[0].abs() < 1e-6 && (frame[1] - 0.7).abs() < 1e-6);
        tx.send(Command::PanSource {
            id: "sled".into(),
            pan: 0.,
        })
        .unwrap();
        mixer.mix(&mut frame);
        assert!((frame[0] - 0.5).abs() < 1e-6 && (frame[1] - 0.6).abs() < 1e-6);
    }
    #[test]
    fn mixing_pause_and_loop_preserve_position() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut mixer = Mixer::new(rx);
        let sample = Arc::new(Sample {
            samples: vec![0.25, 0.5, 0.75],
            channels: 1,
            rate: 48000,
        });
        tx.send(Command::Play {
            pausable: true,
            id: "test".into(),
            policy: PlayPolicy::Retrigger,
            delay: 0.,
            fade_in: 0.,
            sample,
            volume: 1.,
            rate: 1.,
            loop_start: Some(1. / 48000.),
            offset: 0.,
            end: None,
        })
        .ok();
        let mut output = [0.; 8];
        mixer.mix(&mut output);
        assert_eq!(output, [0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 0.5, 0.5]);
        tx.send(Command::Paused(true)).ok();
        mixer.mix(&mut output);
        assert_eq!(output, [0.; 8]);
        tx.send(Command::Paused(false)).ok();
        mixer.mix(&mut output);
        assert_eq!(output, [0.75, 0.75, 0.5, 0.5, 0.75, 0.75, 0.5, 0.5]);
    }
    #[test]
    fn scheduled_loop_fades_and_stops_without_restarting() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut mixer = Mixer::new(rx);
        let sample = Arc::new(Sample {
            samples: vec![1.; 16],
            channels: 1,
            rate: 48000,
        });
        tx.send(Command::Play {
            pausable: true,
            id: "dryer".into(),
            policy: PlayPolicy::IfAbsent,
            delay: 2. / 48000.,
            fade_in: 2. / 48000.,
            sample,
            volume: 1.,
            rate: 1.,
            loop_start: Some(0.),
            offset: 0.,
            end: None,
        })
        .unwrap();
        let mut out = [0.; 12];
        mixer.mix(&mut out);
        assert_eq!(out, [0., 0., 0., 0., 0., 0., 0.5, 0.5, 1., 1., 1., 1.]);
        tx.send(Command::StopSource {
            id: "dryer".into(),
            fade: 2. / 48000.,
        })
        .unwrap();
        mixer.mix(&mut out);
        assert_eq!(out, [1., 1., 0.5, 0.5, 0., 0., 0., 0., 0., 0., 0., 0.]);
        assert!(mixer.voices.is_empty());
    }
    #[test]
    fn ui_audio_plays_while_game_audio_remains_paused() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut mixer = Mixer::new(rx);
        for (id, value, pausable) in [("game", 0.3, true), ("button", 0.1, false)] {
            tx.send(Command::Play {
                id: id.into(),
                pausable,
                policy: PlayPolicy::Retrigger,
                delay: 0.,
                fade_in: 0.,
                sample: Arc::new(Sample {
                    samples: vec![value; 4],
                    channels: 1,
                    rate: 48000,
                }),
                volume: 1.,
                rate: 1.,
                loop_start: None,
                offset: 0.,
                end: None,
            })
            .unwrap();
        }
        tx.send(Command::Paused(true)).unwrap();
        let mut output = [0.; 8];
        mixer.mix(&mut output);
        assert_eq!(output, [0.1; 8]);
        tx.send(Command::Paused(false)).unwrap();
        mixer.mix(&mut output);
        assert_eq!(output, [0.3; 8]);
    }
}
