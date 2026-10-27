#![allow(dead_code)]
//! Owned native Snowball diagnostic; production model/renderer, no web runtime.
#[path = "../src/assets.rs"]
mod assets;
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
    if args.get(3).is_some_and(|s| s == "input") {
        let started = get_time();
        let mut count = 0;
        while get_time() - started < 40. {
            let updating = game.begin_frame(
                get_frame_time() as f64,
                screen_width() as f64,
                screen_height() as f64,
            );
            view.draw(&assets, &game).unwrap();
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
    if args
        .get(3)
        .is_some_and(|s| s == "reference" || s == "reference-pointer")
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
            p.alive = row["alive"].as_bool().unwrap();
            p.gone = false;
            p.frozen = false;
            p.throwing = None;
            p.face = row["rotation"][1].as_f64().unwrap();
            p.anim = row["anim"].as_str().unwrap_or("elf_rig_idle").into();
            p.anim_start = game.time;
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
    }
    if args.get(3).is_some_and(|s| s == "reference-pointer") {
        let began = get_time();
        while get_time() - began < 9. {
            view.draw(&assets, &game).unwrap();
            view.input(&mut game, true);
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
        view.draw(&assets, &game).unwrap();
        next_frame().await
    }
    view.draw(&assets, &game).unwrap();
    get_screen_data().export_png(
        args.get(1)
            .map(String::as_str)
            .unwrap_or("/tmp/snowball-native-game.png"),
    );
    println!(
        "players {} alive {}, knockouts {}, map seed {}",
        game.players.len(),
        game.players.iter().filter(|p| p.alive).count(),
        game.knocked_out,
        game.seed
    );
}
