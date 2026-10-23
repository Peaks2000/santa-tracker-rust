//! entrypoint.js updatePlayNextCards: biased selection, recent-route buffer,
//! and a shared Play destination / two recommendation cards.
use crate::random::Random;
pub const PLAYABLE: [&str; 8] = [
    "santaselfie",
    "jetpack",
    "elfski",
    "presentdrop",
    "runner",
    "boatload",
    "elfmaker",
    "jamband",
];
pub struct NextGames {
    random: Random,
    recent: Vec<String>,
    last_activity: String,
    pub cards: [String; 2],
    pub play: String,
}
impl NextGames {
    pub fn new(seed: &str) -> Self {
        Self {
            random: Random::seeded(seed),
            recent: vec![],
            last_activity: String::new(),
            cards: ["santaselfie".into(), "jetpack".into()],
            play: "santaselfie".into(),
        }
    }
    pub fn on_route(&mut self, route: &str) {
        if route != "modvil" {
            self.last_activity = route.into();
        }
        if !self.recent.iter().any(|r| r == route) {
            self.recent.push(route.into());
        }
        while self.recent.len() >= 6 {
            self.recent.remove(0);
        }
        let mut nav: Vec<String> = PLAYABLE.iter().map(|s| (*s).into()).collect();
        let mut cards = Vec::new();
        let mut attempts = 0;
        while cards.len() < 2 && !nav.is_empty() {
            attempts += 1;
            let i = (self.random.next().powi(4) * nav.len() as f64) as usize;
            let choice = nav.remove(i);
            if (self.recent.contains(&choice) || cards.contains(&choice)) && attempts < 20 {
                continue;
            }
            cards.push(choice);
        }
        // The full site has enough games to keep drawing from its nav. With only
        // five native ports, the filtered list can empty before finding two unseen
        // games. Reuse an older playable entry instead of producing JS undefined.
        let last = self.last_activity.as_str();
        while cards.len() < 2 {
            let choices: Vec<_> = PLAYABLE
                .iter()
                .filter(|r| !cards.iter().any(|c| c == **r) && **r != route && last != **r)
                .collect();
            let i = (self.random.next().powi(4) * choices.len() as f64) as usize;
            cards.push((*choices[i]).to_owned());
        }
        self.cards = [cards.remove(0), cards.remove(0)];
        self.play = self.cards[0].clone();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_recommendations_drive_home_play() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/navigation-reference.json")).unwrap();
        let mut n = NextGames::new("test1234");
        for step in data.as_array().unwrap() {
            n.on_route(step["route"].as_str().unwrap());
            assert_eq!(serde_json::json!(n.cards), step["cards"]);
            assert_eq!(n.play, step["play"].as_str().unwrap());
        }
    }
    #[test]
    fn limited_native_pool_never_selects_missing_or_repeated_activity() {
        let mut n = NextGames::new("play-all-native");
        let mut played = std::collections::HashSet::new();
        let mut previous = String::new();
        for _ in 0..40 {
            n.on_route("modvil");
            let route = n.play.clone();
            assert!(PLAYABLE.contains(&route.as_str()));
            assert_ne!(route, previous);
            assert_ne!(n.cards[0], n.cards[1]);
            played.insert(route.clone());
            n.on_route(&route);
            previous = route;
        }
        assert_eq!(played.len(), PLAYABLE.len());
    }
}
