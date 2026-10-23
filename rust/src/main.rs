mod assets;
mod capture;
mod chrome;
mod interlude;
mod jetpack_view;
mod loader_animation;
mod overlay;
mod render;
mod selfie_view;
mod sound;
mod ui_render;
mod village;

use macroquad::prelude::*;
use santa_tracker::runner::Runner;
use std::path::PathBuf;

struct Options {
    assets: PathBuf,
    seed: String,
    mute: bool,
    screenshot: Option<PathBuf>,
    frames: Option<u32>,
    advance: f64,
    scene: String,
    scroll: f32,
    beard_file: Option<PathBuf>,
    clock_ms: Option<u64>,
    width: i32,
    height: i32,
}
fn options() -> Result<Option<Options>, String> {
    let mut args = std::env::args().skip(1);
    let mut out = Options {
        assets: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"),
        seed: format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ),
        mute: false,
        screenshot: None,
        frames: None,
        advance: 0.,
        scene: "modvil".into(),
        scroll: 0.,
        beard_file: None,
        clock_ms: None,
        width: 1440,
        height: 800,
    };
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if let Some(dir) = exe.parent() {
        if dir.join("assets/scenes/runner/img").is_dir() {
            out.assets = dir.join("assets");
        }
    }
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("Santa Tracker — native Rust port (in development)\n\nAvailable: village, Runner, Elf Jetpack and Santa Selfie.\n\n--assets PATH       Original static assets directory\n--seed TEXT         Deterministic game seed\n--mute              Start without audio\n--scene NAME        modvil, runner, jetpack or santaselfie\n--scroll PIXELS     Initial village scroll position\n--beard-file PATH   Restore an original-format saved beard\n--advance SECONDS   Advance the selected scene before rendering\n--frames COUNT      Exit after COUNT frames (for native render checks)\n--screenshot PATH   Save the final rendered frame (or F12 live)\n--width PIXELS      Initial window width\n--height PIXELS     Initial window height\n--clock-ms EPOCH    Deterministic countdown clock for comparisons\n\nControls: Space/Up jump (twice), Down slide, P pause, R restart, M mute, Escape home");
                return Ok(None);
            }
            "--mute" => out.mute = true,
            "--clock-ms" => {
                out.clock_ms = Some(
                    args.next()
                        .ok_or("Missing clock milliseconds")?
                        .parse()
                        .map_err(|_| "Invalid clock milliseconds")?,
                )
            }
            "--width" | "--height" => {
                let value: i32 = args
                    .next()
                    .ok_or("Missing window size")?
                    .parse()
                    .map_err(|_| "Invalid window size")?;
                if !(320..=4096).contains(&value) {
                    return Err("Window size must be between 320 and 4096".into());
                }
                if arg == "--width" {
                    out.width = value;
                } else {
                    out.height = value;
                }
            }
            "--assets" | "--seed" | "--scene" | "--advance" | "--frames" | "--screenshot"
            | "--scroll" | "--beard-file" => {
                let value = args.next().ok_or_else(|| format!("{arg} needs a value"))?;
                match arg.as_str() {
                    "--assets" => out.assets = value.into(),
                    "--beard-file" => out.beard_file = Some(value.into()),
                    "--seed" => out.seed = value,
                    "--scene" => {
                        if !["runner", "modvil", "jetpack", "santaselfie"].contains(&value.as_str())
                        {
                            return Err(format!("Scene '{value}' has not been ported to Rust yet"));
                        }
                        out.scene = value;
                    }
                    "--scroll" => {
                        out.scroll = value.parse().map_err(|_| "Invalid scroll position")?;
                        if !out.scroll.is_finite() || out.scroll < 0. {
                            return Err("Scroll must be nonnegative and finite".into());
                        }
                    }
                    "--screenshot" => out.screenshot = Some(value.into()),
                    "--advance" => {
                        out.advance = value.parse().map_err(|_| "Invalid time")?;
                        if !out.advance.is_finite() || out.advance < 0. {
                            return Err("Time must be finite and nonnegative".into());
                        }
                    }
                    _ => {
                        let n = value.parse::<u32>().map_err(|_| "Invalid frame count")?;
                        if n == 0 {
                            return Err("Frame count must be positive".into());
                        }
                        out.frames = Some(n);
                    }
                }
            }
            _ => return Err(format!("Unknown option: {arg}")),
        }
    }
    if !out.assets.join("scenes/runner/img/hero.svg").is_file() {
        return Err("Cannot find original art; use --assets /path/to/static".into());
    }
    Ok(Some(out))
}
fn main() {
    match options() {
        Ok(Some(opts)) => macroquad::Window::from_config(
            Conf {
                window_title: "Santa Tracker — Rust port in development".into(),
                window_width: opts.width,
                window_height: opts.height,
                high_dpi: true,
                sample_count: 4,
                ..Default::default()
            },
            run(opts),
        ),
        Ok(None) => {}
        Err(e) => {
            eprintln!("Santa Tracker: {e}");
            std::process::exit(1);
        }
    }
}
async fn run(opts: Options) {
    use santa_tracker::{jetpack::Jetpack, selfie::Selfie};
    let mut assets = assets::Assets::new(opts.assets);
    let mut sound = sound::Sound::new(opts.mute);
    let engine = thorvg::Thorvg::init(0).expect("Native animation engine initialization");
    let mut village = village::Village::new(&engine).expect("Original village fonts and data");
    let mut chrome = chrome::Chrome::new(opts.clock_ms).expect("Original toolbar icons and fonts");
    let mut jet_view = jetpack_view::JetpackView::new().expect("Jetpack fonts");
    let mut selfie_view = selfie_view::SelfieView::new().expect("Selfie rendering materials");
    let mut runner = Runner::new(&opts.seed);
    let (width, height) = jetpack_view::JetpackView::dimensions();
    let mut jetpack = Jetpack::new(&opts.seed, width, height);
    let mut selfie = Selfie::new(&opts.seed);
    if let Some(path) = opts.beard_file {
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|code| selfie.restore(code.trim()))
        {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Cannot load beard {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    }
    let mut route = opts.scene;
    if route == "santaselfie" {
        sound.trigger("music_start_ingame", &assets.root);
    }
    village.scroll = opts.scroll;
    match route.as_str() {
        "runner" => runner.advance(opts.advance),
        "jetpack" => jetpack.advance(opts.advance, (0., 0.)),
        "santaselfie" => {
            for _ in 0..(opts.advance * 1000. / 16.) as u64 {
                selfie.step();
            }
        }
        _ => {}
    }
    let mut loaded = std::collections::HashSet::new();
    let mut unavailable: Option<String> = None;
    let mut accumulated = 0.;
    let mut frames = 0;
    let mut interlude = interlude::Interlude::new();
    let mut loader_animation = loader_animation::LoaderAnimation::new(assets.root.clone());
    let mut overlay = overlay::Overlay::new().expect("Original pause overlay");
    let mut scene_filter = overlay::SceneFilter::new().expect("Original disabled scene filter");
    loop {
        if let Some((scene, result)) = assets.poll_scene_load() {
            if let Err(error) = result {
                eprintln!("Scene {scene}: {error}");
                std::process::exit(1);
            }
            loaded.insert(scene);
        }
        if is_key_pressed(KeyCode::Escape) {
            if unavailable.is_some() {
                unavailable = None;
            } else if selfie_view.export_open {
                selfie_view.export_open = false;
            } else if village.menu {
                village.menu = false;
            } else if route != "modvil" {
                if interlude.begin("modvil".into()) {
                    sound.trigger("menu_transition_game_in", &assets.root);
                }
            } else {
                break;
            }
        }
        let mut just_loaded = false;
        if route != "modvil" && !loaded.contains(&route) {
            if let Err(e) = assets.load_scene(&route) {
                eprintln!("Could not load original {route} assets: {e}");
                std::process::exit(1);
            }
            loaded.insert(route.clone());
            just_loaded = true;
        }
        if is_key_pressed(KeyCode::M) {
            sound.mute();
        }
        let restart = is_key_pressed(KeyCode::R) && route != "modvil";
        if restart {
            sound.stop();
            match route.as_str() {
                "runner" => runner = Runner::new(&opts.seed),
                "jetpack" => {
                    let (w, h) = jetpack_view::JetpackView::dimensions();
                    jetpack = Jetpack::new(&opts.seed, w, h);
                }
                "santaselfie" => selfie.reset(),
                _ => {}
            }
            accumulated = 0.;
        }
        if is_key_pressed(KeyCode::P) {
            match route.as_str() {
                "runner" => {
                    runner.paused = !runner.paused;
                    sound.paused(runner.paused);
                }
                "jetpack" => {
                    jetpack.paused = !jetpack.paused;
                    sound.paused(jetpack.paused);
                }
                "santaselfie" => {
                    selfie.paused = !selfie.paused;
                    sound.paused(selfie.paused);
                }
                _ => {}
            }
        }
        let active = unavailable.is_none()
            && !interlude.busy()
            && match route.as_str() {
                "runner" => runner.playing && !runner.paused,
                "jetpack" => jetpack.playing && !jetpack.paused,
                "santaselfie" => !selfie.paused && !selfie_view.export_open,
                _ => false,
            };
        let mut jet_input = (0., 0.);
        if active {
            match route.as_str() {
                "runner" => {
                    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up) {
                        runner.jump();
                    }
                    if is_key_pressed(KeyCode::Down) {
                        runner.down();
                    }
                    if is_key_released(KeyCode::Down) {
                        runner.end_slide();
                    }
                }
                "jetpack" => {
                    jet_input = jet_view.input();
                    let (w, h) = jetpack_view::JetpackView::dimensions();
                    jetpack.resize(w, h);
                }
                "santaselfie" => selfie_view.input(&mut selfie, opts.frames.is_some()),
                _ => {}
            }
            if opts.frames.is_none() && !just_loaded {
                accumulated += get_frame_time() as f64;
            }
            let tick = if route == "santaselfie" {
                0.016
            } else {
                1. / 60.
            };
            while accumulated > tick {
                match route.as_str() {
                    "runner" => runner.step(tick),
                    "jetpack" => jetpack.step(tick, jet_input),
                    "santaselfie" => selfie.step(),
                    _ => {}
                }
                accumulated -= tick;
            }
        } else {
            accumulated = 0.;
        }
        clear_background(BLACK);
        let mut selected = None;
        match route.as_str() {
            "modvil" => {
                selected = village.draw(&mut assets);
                for event in village.sounds.drain(..) {
                    sound.trigger(event, &assets.root);
                }
            }
            "runner" => {
                for event in runner.sounds.drain(..) {
                    sound.trigger(event, &assets.root);
                }
                scene_filter.begin();
                render::View::new().runner(&assets, &runner);
                scene_filter.finish(runner.paused || !runner.playing);
            }
            "jetpack" => {
                for event in jetpack.sounds.drain(..) {
                    sound.trigger(event, &assets.root);
                }
                scene_filter.begin();
                jet_view.draw(&assets, &jetpack);
                scene_filter.finish(jetpack.paused || !jetpack.playing);
            }
            "santaselfie" => {
                for event in selfie.sounds.drain(..) {
                    sound.trigger(event, &assets.root);
                }
                selfie_view.draw(&assets, &selfie, opts.frames.is_some());
            }
            _ => {}
        }
        if route == "runner" || route == "jetpack" {
            let (paused, gameover) = if route == "runner" {
                (runner.paused, !runner.playing)
            } else {
                (jetpack.paused, !jetpack.playing)
            };
            if let Some(action) =
                overlay.draw(&mut assets, &chrome, &mut village, &route, paused, gameover)
            {
                match action {
                    overlay::Action::Resume => {
                        if route == "runner" {
                            runner.paused = false;
                        } else {
                            jetpack.paused = false;
                        }
                        sound.paused(false);
                    }
                    overlay::Action::Restart => {
                        sound.stop();
                        if route == "runner" {
                            runner = Runner::new(&opts.seed);
                        } else {
                            let (w, h) = jetpack_view::JetpackView::dimensions();
                            jetpack = Jetpack::new(&opts.seed, w, h);
                        }
                        accumulated = 0.;
                    }
                    overlay::Action::Home => selected = Some("modvil".into()),
                    overlay::Action::Route(target) => selected = Some(target),
                }
            }
            for event in overlay.sounds.drain(..) {
                sound.trigger(event, &assets.root);
            }
        }
        let hud = match route.as_str() {
            "runner" => Some(chrome::GameHud {
                playing: runner.playing,
                paused: runner.paused,
                score: runner.score as i64,
                countdown: runner.countdown.ceil(),
                level: None,
                badge: true,
                can_pause: true,
            }),
            "jetpack" => Some(chrome::GameHud {
                playing: jetpack.playing,
                paused: jetpack.paused,
                score: jetpack.score,
                countdown: jetpack.countdown.ceil(),
                level: Some(jetpack.level + 1),
                badge: true,
                can_pause: true,
            }),
            "santaselfie" => Some(chrome::GameHud {
                playing: true,
                paused: selfie.paused,
                score: 0,
                countdown: 0.,
                level: None,
                badge: false,
                can_pause: false,
            }),
            _ => None,
        };
        if let Some(action) = chrome.draw(hud, sound.is_muted()) {
            match action {
                chrome::Action::Home => selected = Some("modvil".into()),
                chrome::Action::Menu => village.menu = !village.menu,
                chrome::Action::Mute => sound.mute(),
                chrome::Action::Pause => match route.as_str() {
                    "runner" => {
                        runner.paused = !runner.paused;
                        sound.paused(runner.paused);
                    }
                    "jetpack" => {
                        jetpack.paused = !jetpack.paused;
                        sound.paused(jetpack.paused);
                    }
                    _ => {}
                },
                chrome::Action::Restart => {
                    sound.stop();
                    match route.as_str() {
                        "runner" => runner = Runner::new(&opts.seed),
                        "jetpack" => {
                            let (w, h) = jetpack_view::JetpackView::dimensions();
                            jetpack = Jetpack::new(&opts.seed, w, h);
                        }
                        _ => {}
                    }
                    accumulated = 0.;
                }
            }
        }
        for event in chrome.sounds.drain(..) {
            sound.trigger(event, &assets.root);
        }
        if let Some(target) = selected {
            if ["modvil", "runner", "jetpack", "santaselfie"].contains(&target.as_str()) {
                if !interlude.busy() {
                    if target != "modvil" && !loaded.contains(&target) {
                        if let Err(error) = assets.start_scene_load(&target) {
                            eprintln!("Scene {target}: {error}");
                            std::process::exit(1);
                        }
                    }
                    interlude.begin(target);
                    sound.trigger("menu_transition_game_in", &assets.root);
                }
            } else {
                unavailable = Some(
                    village
                        .data
                        .scene_names
                        .get(&target)
                        .cloned()
                        .unwrap_or(target),
                );
            }
        }
        let scene_ready = interlude
            .target()
            .is_some_and(|target| target == "modvil" || loaded.contains(target));
        if let Some(target) = if scene_ready { interlude.ready() } else { None } {
            interlude.hide();

            if ["modvil", "runner", "jetpack", "santaselfie"].contains(&target.as_str()) {
                sound.stop();
                accumulated = 0.;
                village.menu = false;
                match target.as_str() {
                    "modvil" => {
                        village
                            .sounds
                            .extend(["music_start_village", "village_start"]);
                    }
                    "runner" => runner = Runner::new(&opts.seed),
                    "jetpack" => {
                        let (w, h) = jetpack_view::JetpackView::dimensions();
                        jetpack = Jetpack::new(&opts.seed, w, h);
                    }
                    "santaselfie" => {
                        selfie = Selfie::new(&opts.seed);
                        sound.trigger("music_start_ingame", &assets.root);
                    }
                    _ => {}
                }
                route = target;
                sound.trigger("menu_transition_game_out", &assets.root);
            } else {
                unavailable = Some(
                    village
                        .data
                        .scene_names
                        .get(&target)
                        .cloned()
                        .unwrap_or(target),
                );
            }
        }
        if let Some(name) = &unavailable {
            draw_rectangle(
                0.,
                0.,
                screen_width(),
                screen_height(),
                Color::new(0., 0., 0., 0.6),
            );
            let r = Rect::new(
                screen_width() / 2. - 230.,
                screen_height() / 2. - 80.,
                460.,
                160.,
            );
            draw_rectangle(r.x, r.y, r.w, r.h, WHITE);
            for (i, line) in [
                name.as_str(),
                "This activity is not available yet.",
                "Press Escape to return.",
            ]
            .iter()
            .enumerate()
            {
                draw_text_ex(
                    line,
                    r.x + 24.,
                    r.y + 42. + i as f32 * 38.,
                    TextParams {
                        font: Some(&village.font),
                        font_size: 24,
                        color: DARKGRAY,
                        ..Default::default()
                    },
                );
            }
        }
        if let Some(layers) = interlude.layers() {
            for (y, color) in layers {
                if let Some(color) = color {
                    draw_rectangle(0., y, screen_width(), screen_height(), color);
                } else {
                    loader_animation.draw(y, interlude.animation_ratio(99. / 24.));
                }
            }
        }
        if is_key_pressed(KeyCode::F12) {
            if let Some(path) = &opts.screenshot {
                if let Err(error) = capture::save_png(&get_screen_data(), path) {
                    eprintln!("Screenshot: {error}");
                }
            }
        }
        frames += 1;
        if opts.frames.is_some_and(|n| frames >= n) {
            if let Some(path) = &opts.screenshot {
                if let Err(error) = capture::save_png(&get_screen_data(), path) {
                    eprintln!("Screenshot: {error}");
                }
            }
            break;
        }
        next_frame().await;
    }
}
