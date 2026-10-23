use santa_tracker::audio::{Command, Mixer, Sample};
use serde_json::Value;
use std::{
    collections::HashMap,
    ffi::{c_char, c_int, c_void},
    path::Path,
    sync::{
        mpsc::{self, Sender},
        Arc,
    },
};

#[repr(C)]
struct AudioSpec {
    frequency: c_int,
    format: u16,
    channels: u8,
    silence: u8,
    samples: u16,
    padding: u16,
    size: u32,
    callback: Option<unsafe extern "C" fn(*mut c_void, *mut u8, c_int)>,
    userdata: *mut c_void,
}
struct Device {
    id: u32,
    close: unsafe extern "C" fn(u32),
    quit: unsafe extern "C" fn(u32),
    _mixer: Box<Mixer>,
    _library: libloading::Library,
}
unsafe extern "C" fn callback(userdata: *mut c_void, output: *mut u8, length: c_int) {
    if length <= 0 || userdata.is_null() || output.is_null() {
        return;
    }
    // SDL invokes this with the f32 stereo format requested below. The boxed
    // mixer stays at the same address until SDL_CloseAudioDevice has joined
    // the device thread. Only this callback ever accesses it after startup.
    let mixer = unsafe { &mut *(userdata as *mut Mixer) };
    let output =
        unsafe { std::slice::from_raw_parts_mut(output.cast::<f32>(), length as usize / 4) };
    mixer.mix(output);
}
impl Device {
    fn open(mixer: Mixer) -> Result<Self, String> {
        let library = [
            "libSDL2-2.0.so.0",
            "libSDL2.so",
            "libSDL2.dylib",
            "SDL2.dll",
        ]
        .iter()
        .find_map(|name| unsafe { libloading::Library::new(name).ok() })
        .ok_or("SDL2 audio device library is unavailable")?;
        unsafe {
            let init = *library
                .get::<unsafe extern "C" fn(u32) -> c_int>(b"SDL_InitSubSystem\0")
                .map_err(|e| e.to_string())?;
            let open = *library
                .get::<unsafe extern "C" fn(
                    *const c_char,
                    c_int,
                    *const AudioSpec,
                    *mut AudioSpec,
                    c_int,
                ) -> u32>(b"SDL_OpenAudioDevice\0")
                .map_err(|e| e.to_string())?;
            let pause = *library
                .get::<unsafe extern "C" fn(u32, c_int)>(b"SDL_PauseAudioDevice\0")
                .map_err(|e| e.to_string())?;
            let close = *library
                .get::<unsafe extern "C" fn(u32)>(b"SDL_CloseAudioDevice\0")
                .map_err(|e| e.to_string())?;
            let quit = *library
                .get::<unsafe extern "C" fn(u32)>(b"SDL_QuitSubSystem\0")
                .map_err(|e| e.to_string())?;
            if init(0x10) != 0 {
                return Err("Could not initialize native audio".into());
            }
            let mut mixer = Box::new(mixer);
            let format = if cfg!(target_endian = "little") {
                0x8120
            } else {
                0x9120
            };
            let spec = AudioSpec {
                frequency: 48000,
                format,
                channels: 2,
                silence: 0,
                samples: 1024,
                padding: 0,
                size: 0,
                callback: Some(callback),
                userdata: (&mut *mixer as *mut Mixer).cast(),
            };
            let id = open(std::ptr::null(), 0, &spec, std::ptr::null_mut(), 0);
            if id == 0 {
                quit(0x10);
                return Err("Could not open stereo audio playback device".into());
            }
            pause(id, 0);
            Ok(Self {
                id,
                close,
                quit,
                _mixer: mixer,
                _library: library,
            })
        }
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            (self.close)(self.id);
            (self.quit)(0x10);
        }
    }
}

pub struct Sound {
    device: Option<Device>,
    config: Value,
    jamband_voices: Vec<(usize, f64, f64, f64, Option<f64>)>,
    maker_switches: usize,
    samples: HashMap<String, Arc<Sample>>,
    commands: Sender<Command>,
    muted: bool,
}
impl Sound {
    pub fn new(muted: bool) -> Self {
        let (commands, rx) = mpsc::channel();
        let device = if muted {
            None
        } else {
            match Device::open(Mixer::new(rx)) {
                Ok(d) => Some(d),
                Err(e) => {
                    eprintln!("Audio device: {e}");
                    None
                }
            }
        };
        Self {
            device,
            config: serde_json::from_str(include_str!("../data/audio.json")).unwrap(),
            jamband_voices: vec![],
            maker_switches: 0,
            samples: HashMap::new(),
            commands,
            muted,
        }
    }
    fn source(&self, key: &str, depth: usize) -> Option<String> {
        if depth > 32 {
            return None;
        }
        let audio = &self.config["audio"][key];
        if audio["file_id"].is_string() {
            return Some(key.into());
        }
        let content = audio["content"].as_array()?;
        if content.is_empty() {
            return None;
        }
        let index = macroquad::rand::gen_range(0, content.len());
        self.source(content[index].as_str()?, depth + 1)
    }
    pub fn presentdrop_pan(&mut self, position: f64) {
        let pan = position.clamp(-0.5, 0.5) as f32;
        self.commands
            .send(Command::PanSource {
                id: "cwIe".into(),
                pan,
            })
            .ok();
    }
    fn preload_keys(&mut self, keys: &[&str], root: &Path) {
        if self.device.is_none() {
            return;
        }
        let mut pending = keys.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        while let Some(key) = pending.pop() {
            let a = &self.config["audio"][&key];
            if let Some(children) = a["content"].as_array() {
                pending.extend(children.iter().filter_map(Value::as_str).map(str::to_owned));
                continue;
            }
            if self.samples.contains_key(&key) {
                continue;
            }
            if let Some(file) = self.config["files"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["id"] == a["file_id"])
            {
                if let Some(url) = file["url"].as_str() {
                    match Sample::load(&root.join("audio").join(format!("{url}.ogg"))) {
                        Ok(s) => {
                            self.samples.insert(key, Arc::new(s));
                        }
                        Err(e) => eprintln!("Preload {url}: {e}"),
                    }
                }
            }
        }
    }
    pub fn maker_start(&mut self, root: &Path) {
        self.preload_keys(&["h54h", "diiL", "h9jl", "Cc2y", "HQfD", "OckO"], root);
        self.trigger("music_start_scene", root);
    }
    pub fn jamband_start(&mut self, root: &Path) {
        if self.device.is_none() {
            return;
        }
        self.preload_keys(
            &[
                "EPDs", "QcJi", "Du2o", "OSie", "Px11", "QLAo", "LdQ0", "uARB", "Jdmw", "SdSp",
                "ZBDz", "J9C5",
            ],
            root,
        );
        self.jamband_voices.clear();
        self.play_source("J9C5", root, 0., 1., true);
    }
    pub fn jamband_event(&mut self, event: santa_tracker::jamband::SoundEvent, root: &Path) {
        use santa_tracker::jamband::SoundEvent;
        if self.device.is_none() {
            return;
        }
        const GROUPS: [&str; 10] = [
            "EPDs", "QcJi", "Du2o", "OSie", "Px11", "QLAo", "LdQ0", "uARB", "Jdmw", "SdSp",
        ];
        match event {
            SoundEvent::Preview(i) => {
                let keys = self.config["audio"]["ZBDz"]["content"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>();
                for k in &keys {
                    self.commands
                        .send(Command::StopSource {
                            id: k.clone(),
                            fade: 1.,
                        })
                        .ok();
                }
                let p = [0, 3, 1, 2, 4, 5, 6, 7, 8, 9][i];
                self.play_source(&keys[p], root, 0., 0., true);
            }
            SoundEvent::Track {
                instrument,
                pattern,
                volume,
            } => {
                let now = macroquad::time::get_time();
                self.jamband_voices
                    .retain(|v| v.4.is_none_or(|end| end > now));
                let position = self
                    .jamband_voices
                    .iter()
                    .map(|v| (v.2 + now - v.1).rem_euclid(v.3))
                    .fold(0.0_f64, f64::max);
                for v in &mut self.jamband_voices {
                    if v.0 == instrument && v.4.is_none() {
                        v.4 = Some(now + 0.2);
                    }
                }
                let keys = self.config["audio"][GROUPS[instrument]]["content"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>();
                for k in &keys {
                    self.commands
                        .send(Command::StopSource {
                            id: k.clone(),
                            fade: 0.2,
                        })
                        .ok();
                }
                if pattern >= 0 {
                    let key = &keys[pattern as usize];
                    let old = self.config["audio"][key].clone();
                    let mut source = old.clone();
                    let duration = self.config["files"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|f| f["id"] == source["file_id"])
                        .and_then(|f| f["duration"].as_f64())
                        .unwrap_or(16.);
                    let end = source["loop_end"]
                        .as_f64()
                        .filter(|v| *v > 0.)
                        .unwrap_or(duration);
                    source["offset"] = serde_json::json!(position);
                    source["volume"] = serde_json::json!(volume);
                    source["retrig"] = serde_json::json!(true);
                    self.config["audio"][key] = source;
                    self.play_source(key, root, 0., 0., true);
                    self.config["audio"][key] = old;
                    self.jamband_voices
                        .push((instrument, now, position, end, None));
                }
            }
        }
    }
    pub fn trigger(&mut self, event: &str, root: &Path) {
        if event == "elfmaker_switch_item" {
            self.maker_switches += 1;
            self.play_source("diiL", root, 0., 0., true);
            if self.maker_switches % 8 == 0 {
                self.play_source("h9jl", root, 0., 0., true);
            }
            return;
        }

        if self.device.is_none() {
            return;
        }
        // Native translations of the original focused scenes' Klang actions.
        // An empty action stays empty; stop actions never restart their sources.
        let actions: Option<&[(&str, bool, f64, f64)]> = match event {
            "music_start_scene" => Some(&[("OckO", true, 0., 0.2)]),
            "elfmaker_switch_type" => Some(&[("h54h", true, 0., 0.)]),
            "elfmaker_switch_color" => Some(&[("Cc2y", true, 0., 0.)]),
            "elfmaker_photo" => Some(&[("HQfD", true, 0., 0.)]),
            "bl_game_start" => Some(&[("uVNy", true, 0., 0.)]),
            "bl_game_stop" => Some(&[("uVNy", false, 0., 0.3)]),
            "bl_shoot" => Some(&[("dSUn", true, 0., 0.), ("KRJ2", false, 0., 0.)]),
            "bl_hit_ice" => Some(&[("SbJ6", true, 0., 0.)]),
            "bl_hit_water" => Some(&[("BGt_", true, 0., 0.)]),
            "bl_score_yellow" => Some(&[("KRJ2", false, 0., 0.), ("oEVR", true, 0., 0.)]),
            "bl_score_red" => Some(&[("lfW0", true, 0., 0.)]),
            "bl_rope_stretch" => Some(&[("KRJ2", true, 0., 0.)]),
            "runner_hit" => Some(&[("dk0G", true, 0., 0.)]),
            "runner_jump_1" => Some(&[("s5Rs", true, 0., 0.)]),
            "runner_jump_2" => Some(&[("nMr5", true, 0., 0.)]),
            "runner_present" => Some(&[("NvN6", true, 0., 0.)]),
            "runner_game_over" => Some(&[("Kzah", false, 0., 0.3)]),
            "runner_start" | "runner_end" | "runner_hurry_up" | "runner_hurry_up_end" => Some(&[]),
            "pd_item_drop" => Some(&[("NuNw", true, 0., 0.)]),
            "pd_item_miss" => Some(&[("ZE1O", true, 0., 0.)]),
            "pd_player_level_up" => Some(&[("lUXp", true, 0., 0.)]),
            "pd_player_present_pickup" => Some(&[("p7Sm", true, 0., 0.)]),
            "pd_pull_start" => Some(&[("cwIe", true, 0., 0.1)]),
            "pd_pull_stop" | "pd_game_over" => Some(&[("cwIe", false, 0., 0.)]),
            "pd_end_game" => Some(&[("cwIe", false, 0., 0.4)]),
            "pd_start_game" | "pd_player_crash" | "pd_game_pause" | "pd_game_unpause"
            | "pd_landing_page" | "pd_sled_position" => Some(&[]),
            "elfski_loop_start" => Some(&[("jbIP", true, 0., 0.1)]),
            "elfski_loop_stop" => Some(&[("jbIP", false, 0., 0.1)]),
            "elfski_crash" => Some(&[("dk0G", true, 0., 0.), ("jbIP", false, 0., 0.1)]),
            "jetpack_clock" => Some(&[("OZ9t", true, 0., 0.)]),
            "jetpack_hit" => Some(&[("dk0G", true, 0., 0.)]),
            "jp_back_to_screen" => Some(&[("PBoz", true, 0., 0.)]),
            "jp_out_of_screen" => Some(&[("WuZk", true, 0., 0.)]),
            "jp_level_up" => Some(&[("lUXp", true, 0., 0.)]),
            "jp_pickup_item" => Some(&[("gPwn", true, 0., 0.)]),
            "jp_pickup_parachute_item" => Some(&[("TxLC", true, 0., 0.)]),
            "jp_start_game" => Some(&[("XgMa", true, 0., 0.3), ("Kzah", true, 0., 0.3)]),
            "jp_end_game" | "jp_game_over" => {
                Some(&[("XgMa", false, 0., 0.3), ("Kzah", false, 0., 0.3)])
            }
            "jp_hurry_up" | "jp_hurry_up_end" | "jp_game_pause" | "jp_game_unpause"
            | "jp_player_moving" | "jp_player_position" | "selfie_start" => Some(&[]),
            "selfie_click" => Some(&[("CdtB", true, 0., 0.)]),
            "selfie_color_start" => Some(&[("KWUj", true, 0., 0.), ("rmPs", true, 0., 0.)]),
            "selfie_color_stop" => Some(&[("KWUj", false, 0., 0.), ("V9u3", true, 0., 0.)]),
            "selfie_concerned" => Some(&[("_aVt", true, 0., 0.)]),
            "selfie_dryer_start" => Some(&[("bB_A", true, 0.5, 0.1), ("jUd0", true, 0., 0.)]),
            "selfie_dryer_stop" => Some(&[
                ("bB_A", false, 0., 0.),
                ("SKdc", true, 0., 0.),
                ("jUd0", false, 0., 0.1),
            ]),
            "selfie_end" => Some(&[
                ("KWUj", false, 0., 0.1),
                ("bB_A", false, 0., 0.1),
                ("sA5i", false, 0., 0.1),
                ("iSmY", false, 0., 0.1),
            ]),
            "selfie_happy" => Some(&[("nJDV", true, 0., 0.)]),
            "selfie_hoho" => Some(&[("hRRa", true, 0., 0.)]),
            "selfie_item" => Some(&[("LPnK", true, 0., 0.)]),
            "selfie_nose_off" => Some(&[("iWxS", true, 0., 0.)]),
            "selfie_nose_on" => Some(&[("b5sR", true, 0., 0.)]),
            "selfie_shave_cutting_start" => Some(&[("sA5i", true, 0., 0.)]),
            "selfie_shave_cutting_stop" => Some(&[("sA5i", false, 0., 0.)]),
            "selfie_shave_start" => Some(&[("iSmY", true, 0., 0.)]),
            "selfie_shave_stop" => Some(&[("iSmY", false, 0., 0.), ("QYm4", true, 0., 0.)]),
            "selfie_spray_big" => Some(&[("quwj", true, 0., 0.)]),
            "selfie_spray_small" => Some(&[("DCs_", true, 0., 0.)]),
            _ => None,
        };
        if let Some(actions) = actions {
            for &(id, play, delay, fade) in actions {
                if play {
                    self.play_source(id, root, delay, fade, true);
                } else {
                    self.commands
                        .send(Command::StopSource {
                            id: id.into(),
                            fade,
                        })
                        .ok();
                }
            }
            return;
        }
        // These original processes are empty, or only stop the prior
        // Jetpack ambience. They must not start their referenced samples.
        if matches!(
            event,
            "runner_start" | "runner_hurry_up" | "runner_game_over" | "menu_out"
        ) {
            return;
        }
        if event == "music_ingame_gameover" {
            self.commands.send(Command::Stop).ok();
        }
        let Some(process) = self.config["events"][event].as_str() else {
            return;
        };
        let Some(vars) = self.config["processes"][process]["vars"].as_array() else {
            return;
        };
        let Some(id) = vars
            .iter()
            .filter_map(Value::as_str)
            .find_map(|key| self.source(key, 0))
        else {
            return;
        };
        self.play_source(
            &id,
            root,
            0.,
            0.,
            !(event.starts_with("generic_")
                || event.starts_with("nav_")
                || event.starts_with("menu_")),
        );
    }
    fn play_source(&mut self, key: &str, root: &Path, delay: f64, fade_in: f64, pausable: bool) {
        let Some(id) = self.source(key, 0) else {
            return;
        };
        let source = &self.config["audio"][&id];
        if !self.samples.contains_key(&id) {
            let Some(file_id) = source["file_id"].as_str() else {
                return;
            };
            let Some(file) = self.config["files"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["id"] == file_id)
            else {
                return;
            };
            let Some(url) = file["url"].as_str() else {
                return;
            };
            let path = root.join("audio").join(format!("{url}.ogg"));
            match Sample::load(&path) {
                Ok(sample) => {
                    self.samples.insert(id.clone(), Arc::new(sample));
                }
                Err(e) => {
                    eprintln!("{}: {e}", path.display());
                    return;
                }
            }
        }
        self.commands
            .send(Command::Play {
                pausable,
                id: id.clone(),
                retrigger: source["retrig"].as_bool().unwrap_or(true),
                delay,
                fade_in,
                sample: Arc::clone(&self.samples[&id]),
                volume: (source["volume"].as_f64().unwrap_or(1.)
                    * if key == "cwIe" { 0.7 } else { 1. }) as f32,
                rate: source["playback_rate"].as_f64().unwrap_or(1.),
                loop_start: if source["loop"].as_bool().unwrap_or(false) {
                    Some(source["loop_start"].as_f64().unwrap_or(0.))
                } else {
                    None
                },
                offset: source["offset"].as_f64().unwrap_or(0.),
                end: source["loop_end"].as_f64().filter(|v| *v > 0.),
            })
            .ok();
    }
    pub fn mute(&mut self) {
        self.muted = !self.muted;
        if !self.muted && self.device.is_none() {
            let (commands, rx) = mpsc::channel();
            match Device::open(Mixer::new(rx)) {
                Ok(device) => {
                    self.device = Some(device);
                    self.commands = commands;
                }
                Err(error) => {
                    eprintln!("Audio device: {error}");
                    self.muted = true;
                }
            }
        }
        self.commands
            .send(Command::Gain(if self.muted { 0. } else { 1. }))
            .ok();
    }
    pub fn is_muted(&self) -> bool {
        self.muted
    }
    pub fn paused(&mut self, value: bool) {
        self.commands.send(Command::Paused(value)).ok();
    }
    pub fn stop(&mut self) {
        self.commands.send(Command::Stop).ok();
        self.commands.send(Command::Paused(false)).ok();
    }
}
