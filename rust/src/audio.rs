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
pub enum Command {
    Play {
        pausable: bool,
        id: String,
        retrigger: bool,
        delay: f64,
        fade_in: f64,
        sample: Arc<Sample>,
        volume: f32,
        rate: f64,
        loop_start: Option<f64>,
        offset: f64,
        end: Option<f64>,
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
    delay: usize,
    age: usize,
    fade_in: usize,
    fade_out: Option<(usize, usize)>,
    sample: Arc<Sample>,
    position: f64,
    step: f64,
    volume: f32,
    loop_start: Option<f64>,
    end: f64,
}
pub struct Mixer {
    commands: Receiver<Command>,
    voices: Vec<Voice>,
    paused: bool,
    gain: f32,
}
impl Mixer {
    pub fn new(commands: Receiver<Command>) -> Self {
        Self {
            commands,
            voices: Vec::with_capacity(64),
            paused: false,
            gain: 1.,
        }
    }
    pub fn mix(&mut self, output: &mut [f32]) {
        for command in self.commands.try_iter() {
            match command {
                Command::Stop => self.voices.clear(),
                Command::StopSource { id, fade } => {
                    for voice in self.voices.iter_mut().filter(|v| v.id == id) {
                        let frames = (fade.max(0.) * 48000.) as usize;
                        voice.fade_out = Some((frames, frames));
                    }
                }
                Command::Paused(p) => self.paused = p,
                Command::Gain(g) => self.gain = g,
                Command::Play {
                    pausable,
                    id,
                    retrigger,
                    delay,
                    fade_in,
                    sample,
                    volume,
                    rate,
                    loop_start,
                    offset,
                    end,
                } => {
                    if !retrigger
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
                        id,
                        delay: (delay.max(0.) * 48000.) as usize,
                        age: 0,
                        fade_in: (fade_in.max(0.) * 48000.) as usize,
                        fade_out: None,
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
            for voice in &mut self.voices {
                if self.paused && voice.pausable {
                    continue;
                }
                if voice.delay > 0 {
                    voice.delay -= 1;
                    continue;
                }
                let mut envelope = if voice.fade_in > 0 {
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
                for (channel, out) in frame.iter_mut().enumerate() {
                    let channel = channel.min(voice.sample.channels - 1);
                    let a = voice.sample.samples[index * voice.sample.channels + channel];
                    let b = voice.sample.samples[next * voice.sample.channels + channel];
                    *out += (a + (b - a) * fraction) * voice.volume * self.gain * envelope;
                }
                voice.position += voice.step;
            }
        }
        self.voices.retain(|v| {
            !matches!(v.fade_out, Some((0, _))) && (v.loop_start.is_some() || v.position < v.end)
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
            retrigger: true,
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
            retrigger: false,
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
                retrigger: true,
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
