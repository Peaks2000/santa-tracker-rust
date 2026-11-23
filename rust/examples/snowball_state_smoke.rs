//! CPU-only local-match smoke. No window, graphics, assets, audio or input API.
use santa_tracker::snowball::{Snowball, UNDEFINED_TILE};
fn main() {
    let mut game = Snowball::new("spawn-late-match-smoke");
    game.start();
    assert_eq!(game.all_time, 0);
    assert_eq!(game.players.len(), 1);
    assert_eq!(game.drops.len(), 25);
    assert!(game.players.iter().all(|p| p.tile == -1));
    assert!(game.drops.iter().all(|d| d.tile == -1));
    for frame in 0..180 * 60 {
        game.step(1. / 60., 1440., 800.);
        if frame == 0 {
            assert_eq!(game.players.len(), 26);
            assert_eq!(game.all_time, 26);
        }
        game.sounds.clear();
    }
    assert!(game.all_time <= 100);
    assert!(game.players.iter().all(|p| !p.active || p.setup));
    assert!(game.drops.iter().all(|d| d.setup));
    let report = serde_json::json!({
        "scope":"CPU-only native regression, not source/full-game parity or GPU validation",
        "seconds":180,"frames":180*60,"all_time":game.all_time,
        "player_slots":game.players.len(),"active_players":game.players.iter().filter(|p|p.active).count(),
        "bot_controllers":game.bots.len(),"drops":game.drops.len(),
        "knocked_out":game.knocked_out,"gameover":game.gameover,
        "map_rings":game.map.rings.len(),
        "undefined_player_tiles":game.players.iter().filter(|p|p.tile==UNDEFINED_TILE).count(),
        "undefined_drop_tiles":game.drops.iter().filter(|d|d.tile==UNDEFINED_TILE).count(),
        "shots":game.shots.len(),"particle_slots":game.effects.trails.len()+game.effects.splats.len()
    });
    let output = std::env::args().nth(1);
    let text = serde_json::to_string_pretty(&report).unwrap() + "\n";
    if let Some(path) = output {
        std::fs::write(path, &text).unwrap();
    }
    print!("{text}");
}
