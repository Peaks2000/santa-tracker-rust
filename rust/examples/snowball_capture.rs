#![allow(dead_code)]
//! Owned native Snowball diagnostic; production model/renderer, no web runtime.
#[path = "../src/assets.rs"]
mod assets;
#[path = "../src/scene_filter.rs"]
mod scene_filter;
#[path = "../src/ski_view.rs"]
mod ski_view;
#[path = "../src/snowball_model.rs"]
mod snowball_model;
#[path = "../src/snowball_view.rs"]
mod snowball_view;
#[path = "../src/ui_render.rs"]
mod ui_render;
mod chrome {
    pub fn over_controls(_: macroquad::prelude::Vec2, _: usize) -> bool {
        false
    }
}
use macroquad::prelude::*;
fn main() {
    macroquad::Window::from_config(
        Conf {
            window_title: "Snowball diagnostic".into(),
            window_width: 1440,
            window_height: 800,
            sample_count: 4,
            high_dpi: true,
            ..Default::default()
        },
        run(),
    )
}
async fn run() {
    let args: Vec<_> = std::env::args().collect();
    let seconds = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(6.);
    let mut assets =
        assets::Assets::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../static"));
    assets.load_scene("snowball").unwrap();
    let mut view = snowball_view::SnowballView::new().unwrap();
    let mut game = santa_tracker::snowball::Snowball::new("snowball-test");
    let mut filter = scene_filter::SceneFilter::new().unwrap();
    let pixel_ratio = args
        .get(4)
        .map(|s| s.parse::<f32>().expect("capture pixel ratio"));
    if args.get(3).is_some_and(|s| s == "input") {
        let started = get_time();
        let mut count = 0;
        while get_time() - started < 40. {
            let updating = game.begin_frame(
                get_frame_time() as f64,
                screen_width() as f64,
                screen_height() as f64,
            );
            filter.begin();
            view.draw(&assets, &game).unwrap();
            filter.finish_live(game.gameover);
            view.input(&mut game, true);
            if updating {
                game.finish_frame(screen_width() as f64, screen_height() as f64);
            }
            if game.sounds.contains(&"snowball_throw") {
                use std::io::Write;
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/snowball-input-events.jsonl")
                    .unwrap();
                writeln!(f,"{}",serde_json::json!({"event":"local_throw","time":game.time,"shots":game.shots.iter().filter(|s|s.thrower==0).count()})).unwrap();
            }
            if count % 30 == 0 {
                let player = game.players.first();
                let vp = snowball_view::SnowballView::vp(&game);
                let target=game.players.iter().enumerate().filter(|(_,p)|p.arrived&&p.alive).find(|(i,p)|*i>0&&{let at=vp.transform_point3(vec3(p.pos.x as f32,p.pos.y as f32,18.));at.x.abs()<0.8&&at.y.abs()<0.7}).map(|(i,p)|{let at=vp.transform_point3(vec3(p.pos.x as f32,p.pos.y as f32,18.));serde_json::json!({"index":i,"x":(at.x+1.)*screen_width()/2.,"y":(1.-at.y)*screen_height()/2.})});
                let report = serde_json::json!({"mouse":mouse_position(),"wall":game.wall,"down":is_mouse_button_down(MouseButton::Left),"pressed":view.pressed,"time":game.time,"phase":if game.phase==santa_tracker::snowball::Phase::Playing{"playing"}else if game.phase==santa_tracker::snowball::Phase::Intro{"intro"}else{"splash"},"player":player.map(|p|serde_json::json!({"pos":p.pos,"arrived":p.arrived,"alive":p.alive,"path":p.path.len()})),"shots":game.shots.len(),"sounds":game.sounds,"target":target});
                std::fs::write("/tmp/snowball-input-state.json", report.to_string()).unwrap();
            }
            if count % 60 == 0 {
                get_screen_data().export_png(args.get(1).map(String::as_str).unwrap());
            }
            count += 1;
            game.sounds.clear();
            next_frame().await;
        }
        return;
    }
    game.start();
    game.advance(seconds, 1440., 800.);
    if let Some(name) = args.get(3).and_then(|s| s.strip_prefix("marker-")) {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-client-reference.json")).unwrap();
        let row = data["renders"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == name)
            .expect("marker phase");
        game.players.clear();
        game.drops.clear();
        game.shots.clear();
        game.particles.clear();
        game.map.states.fill(0);
        game.map.obstacles.fill(-1);
        game.splash_until = 0.;
        game.camera = santa_tracker::snowball::V::new(
            row["camera"][0].as_f64().unwrap(),
            row["camera"][1].as_f64().unwrap(),
        );
        if !row["marker"].is_null() {
            let marker = &row["marker"];
            game.client_arrival.marker = Some(santa_tracker::snowball_client::ArrivalMarker {
                position: std::array::from_fn(|i| marker["position"][i].as_f64().unwrap()),
                rotation: marker["rotation"].as_f64().unwrap(),
            });
        }
    }
    if let Some(tick) = args
        .get(3)
        .and_then(|s| s.strip_prefix("removal-"))
        .and_then(|s| s.parse::<u64>().ok())
    {
        use santa_tracker::{
            snowball::V,
            snowball_removal::{Ice, Parent, Presence},
        };
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../data/snowball-removal-render-reference.json"
        ))
        .unwrap();
        let r = data["frames"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["tick"].as_u64() == Some(tick))
            .unwrap();
        game.players.clear();
        game.drops.truncate(1);
        game.shots.clear();
        game.particles.clear();
        game.parachutes = Default::default();
        game.removal = Default::default();
        game.map.states.fill(0);
        game.map.obstacles.fill(-1);
        game.splash_until = 0.;
        game.time = tick as f64 / 0.06;
        game.camera = V::new(
            data["camera"][0].as_f64().unwrap(),
            data["camera"][1].as_f64().unwrap(),
        );
        let d = &mut game.drops[0];
        d.setup = true;
        d.arrived = true;
        d.landed = true;
        d.active = true;
        d.gone = false;
        d.frozen = true;
        d.parent = Parent::Ice(0);
        d.presence = Presence {
            present: true,
            exit_time: 100.,
        };
        d.pos = V::new(
            r["entity"][0].as_f64().unwrap(),
            r["entity"][1].as_f64().unwrap(),
        );
        d.entity_z = r["entity"][2].as_f64().unwrap();
        d.z = r["modelZ"].as_f64().unwrap();
        d.rotation = r["modelY"].as_f64().unwrap();
        d.tilt = r["modelX"].as_f64().unwrap();
        d.opacity = 1.;
        d.spin = None;
        d.color = 0;
        d.scale = std::array::from_fn(|i| data["scale"][i].as_f64().unwrap());
        game.removal.bodies.push(Ice {
            pos: V::new(r["pos"][0].as_f64().unwrap(), r["pos"][1].as_f64().unwrap()),
            z: r["pos"][2].as_f64().unwrap(),
            rotation: r["rotation"].as_f64().unwrap(),
            opacity: r["opacity"].as_f64().unwrap(),
            start_tick: r["start"].as_f64().unwrap(),
            radius: Some(r["radius"].as_f64().unwrap()),
        });
        game.removal.layer.push(0);
    }
    if let Some(tick) = args
        .get(3)
        .and_then(|s| s.strip_prefix("arrival-"))
        .and_then(|s| s.parse::<u64>().ok())
    {
        use santa_tracker::{
            snowball::V,
            snowball_collision::Id,
            snowball_parachute::{Canopy, Parachutes},
        };
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../data/snowball-arrival-render-reference.json"
        ))
        .unwrap();
        let row = data["frames"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["tick"].as_u64() == Some(tick))
            .unwrap();
        game.players.clear();
        game.shots.clear();
        game.particles.clear();
        game.drops.truncate(1);
        game.map.states.fill(0);
        game.map.obstacles.fill(-1);
        game.splash_until = 0.;
        game.time = tick as f64 / 0.06;
        game.camera = V::new(
            data["camera"][0].as_f64().unwrap(),
            data["camera"][1].as_f64().unwrap(),
        );
        let d = &mut game.drops[0];
        d.setup = true;
        d.landed = false;
        d.arrived = false;
        d.gone = false;
        d.frozen = false;
        d.parent = santa_tracker::snowball_removal::Parent::Canopy;
        d.tilt = std::f64::consts::PI / 2.5;
        d.color = 0;
        d.pos = V::default();
        d.z = 32.;
        d.rotation = 0.;
        d.spin = None;
        d.scale = std::array::from_fn(|i| data["scale"][i].as_f64().unwrap());
        game.parachutes = Parachutes::default();
        game.parachutes.canopies.push(Canopy {
            entity: Id::Drop(0),
            pos: V::new(
                row["pos"][0].as_f64().unwrap(),
                row["pos"][1].as_f64().unwrap(),
            ),
            z: row["pos"][2].as_f64().unwrap(),
            rotation: row["rotation"].as_f64().unwrap(),
        });
    }
    if args
        .get(3)
        .is_some_and(|s| s == "reference" || s == "reference-pointer" || s.starts_with("action-"))
    {
        use santa_tracker::snowball::{Grid, Map, V};
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-render-reference.json")).unwrap();
        game.map = Map::new(Grid::default(), 1);
        game.time = data["time"].as_f64().unwrap();
        game.splash_until = 0.;
        game.gameover = false;
        game.camera = V::new(
            data["camera"][0].as_f64().unwrap(),
            data["camera"][1].as_f64().unwrap(),
        );
        for i in 0..4096 {
            game.map.states[i] = data["states"][i * 2].as_u64().unwrap() as u8;
            game.map.times[i] = data["states"][i * 2 + 1].as_f64().unwrap();
            game.map.obstacles[i] = data["obstacles"][i].as_i64().unwrap() as i8;
        }
        let rows = data["players"].as_array().unwrap();
        let template = game.players[0].clone();
        game.players.resize(rows.len(), template);
        for (i, (p, row)) in game.players.iter_mut().zip(rows).enumerate() {
            p.pos = V::new(
                row["position"][0].as_f64().unwrap(),
                row["position"][1].as_f64().unwrap(),
            );
            p.tile = row["tile"].as_i64().unwrap() as i32;
            p.arrived = row["arrived"].as_bool().unwrap();
            p.landed = p.arrived;
            p.parent = if p.landed {
                santa_tracker::snowball_removal::Parent::Map
            } else {
                santa_tracker::snowball_removal::Parent::Detached
            };
            p.alive = row["alive"].as_bool().unwrap();
            // This legacy raster fixture imports poses/textures, not source
            // loading/LOD flags. Keep its existing ready-to-draw isolation.
            p.lod.root_visible = true;
            p.gone = false;
            p.frozen = false;
            p.face = row["rotation"][1].as_f64().unwrap();
            p.anim = row["anim"].as_str().unwrap_or("elf_rig_idle").into();
            p.animator = santa_tracker::snowball_animation::Animator::new();
            p.animator.play(&p.anim, game.time);
            p.animator.update(game.time);
            p.color = [
                "#84488F", "#D55040", "#4DB1E9", "#55AB57", "#56BBCA", "#BC3EB4",
            ]
            .iter()
            .position(|s| *s == row["color"].as_str().unwrap())
            .unwrap();
            p.skin = ["#E7C241", "#936644", "#B8906D", "#DBBC99"]
                .iter()
                .position(|s| *s == row["skin"].as_str().unwrap())
                .unwrap();
            if row["texture"].is_string() {
                let bytes = std::fs::read(format!(
                    "{}/assets/snowball/reference-textures/{i}.png",
                    env!("CARGO_MANIFEST_DIR")
                ))
                .unwrap();
                let t = Texture2D::from_file_with_format(&bytes, None);
                assets::configure_texture("scenes/snowball/reference.png", &t);
                view.set_texture(p.color, p.skin, t);
            }
        }
        game.drops.clear();
        game.shots.clear();
        game.particles.clear();
        // This captured source local is alive and arrived after real setup.
        // Its shared ring belongs to the local dolly, unlike a fresh start's
        // not-yet-registered actors. Do not infer this for an arbitrary corpse.
        assert!(game.players[0].alive && game.players[0].arrived);
        game.client_ring.setup(0);
        game.client_ring.visible = true;
    }
    if let Some(step) = args
        .get(3)
        .and_then(|s| s.strip_prefix("action-"))
        .and_then(|s| s.parse::<usize>().ok())
    {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../data/snowball-animation-reference.json"))
                .unwrap();
        let mut animator = santa_tracker::snowball_animation::Animator::new();
        for state in reference["cases"][0]["states"]
            .as_array()
            .unwrap()
            .iter()
            .take(step + 1)
        {
            let op = &state["op"];
            let now = op["at"].as_f64().unwrap();
            match op["op"].as_str().unwrap() {
                "play" => {
                    animator.play(op["name"].as_str().unwrap(), now);
                }
                "mix" => animator.mix_once(op["name"].as_str().unwrap()),
                "update" => animator.update(now),
                _ => unreachable!(),
            }
        }
        game.players[0].animator = animator;
    }
    if args.get(3).is_some_and(|s| s == "reference-pointer") {
        let began = get_time();
        while get_time() - began < 9. {
            filter.begin();
            view.draw(&assets, &game).unwrap();
            filter.finish_live(game.gameover);
            view.input(&mut game, true);
            // Target requests now follow the real Client -> Player stage. Hold
            // the imported pose until an input release, then dispatch one frame.
            if is_mouse_button_released(MouseButton::Left) {
                game.reset_collisions();
                game.finish_frame(screen_width() as f64, screen_height() as f64);
            }
            let vp = snowball_view::SnowballView::vp(&game);
            let target=game.players.iter().enumerate().filter(|(i,p)|*i>0&&p.arrived&&p.alive).find_map(|(i,p)|{let at=vp.transform_point3(vec3(p.pos.x as f32,p.pos.y as f32,18.));(at.x.abs()<0.75&&at.y.abs()<0.7).then_some(serde_json::json!({"index":i,"x":(at.x+1.)*screen_width()/2.,"y":(1.-at.y)*screen_height()/2.}))});
            std::fs::write("/tmp/snowball-input-state.json",serde_json::json!({"mouse":mouse_position(),"target":target,"local_shots":game.shots.iter().filter(|s|s.thrower==0).count(),"local_throw_event":game.sounds.contains(&"snowball_throw")}).to_string()).unwrap();
            next_frame().await;
        }
        assert!(
            game.shots.iter().any(|s| s.thrower == 0),
            "isolated pointer case must hit an actual elf"
        );
        return;
    }
    for _ in 0..2 {
        if let Some(ratio) = pixel_ratio {
            filter.begin_with_pixel_ratio(ratio);
        } else {
            filter.begin();
        }
        view.draw(&assets, &game).unwrap();
        filter.finish_live(game.gameover);
        next_frame().await
    }
    if let Some(ratio) = pixel_ratio {
        filter.begin_with_pixel_ratio(ratio);
    } else {
        filter.begin();
    }
    view.draw(&assets, &game).unwrap();
    filter.finish_live(game.gameover);
    let output = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("/tmp/snowball-native-game.png");
    if pixel_ratio.is_some() {
        assert!(
            !game.gameover,
            "density diagnostic captures the enabled scene"
        );
        let mut pixels = filter.scene_texture().get_texture_data();
        // The production opaque-copy shader presents exactly this RGB with A=1.
        for rgba in pixels.bytes.chunks_exact_mut(4) {
            rgba[3] = 255;
        }
        pixels.export_png(output);
    } else {
        get_screen_data().export_png(output);
    }
    println!(
        "players {} alive {}, knockouts {}, map seed {}",
        game.players.len(),
        game.players.iter().filter(|p| p.alive).count(),
        game.knocked_out,
        game.seed
    );
}
