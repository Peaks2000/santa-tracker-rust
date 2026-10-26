//! Coding-scene level/host completion state from original Game.bumpLevel and
//! scene API restart handlers. Original: Google LLC, Apache-2.0.
use serde::Serialize;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Next {
    Level(usize),
    Teacher,
    Stop,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodingHost {
    #[serde(skip)]
    pub dance: bool,
    #[serde(skip)]
    pub freestyle: bool,
    pub level_number: usize,
    pub game_level: Option<usize>,
    pub editor_level: usize,
    pub score_level: usize,
    pub max_level: usize,
    pub gameover: bool,
    pub stop_level: Option<usize>,
}
impl CodingHost {
    pub fn new(dance: bool, count: usize) -> Self {
        Self {
            dance,
            freestyle: false,
            level_number: 0,
            game_level: Some(0),
            editor_level: 0,
            score_level: 1,
            max_level: count,
            gameover: false,
            stop_level: None,
        }
    }
    pub fn load(&mut self, index: usize, count: usize, freestyle: bool) {
        self.level_number = index;
        self.editor_level = index;
        self.game_level = Some(index);
        self.freestyle = freestyle;
        self.max_level = count;
        self.gameover = false;
        self.stop_level = None;
        if !freestyle {
            self.score_level = index + 1;
        }
    }
    pub fn advance(&mut self) -> Next {
        self.level_number += 1;
        if self.level_number < self.max_level {
            self.game_level = Some(self.level_number);
            self.editor_level = self.level_number;
            if !self.freestyle {
                self.score_level = self.level_number + 1;
            }
            Next::Level(self.level_number)
        } else if self.freestyle {
            Next::Teacher
        } else {
            self.game_level = None;
            self.gameover = true;
            self.stop_level = Some(self.level_number + usize::from(!self.dance));
            Next::Stop
        }
    }
    /// Returns true when the scene actually handles restart. Code Boogie's
    /// handler is a no-op even though the host dismisses its game-over overlay.
    pub fn restart_from_host(&mut self) -> bool {
        self.gameover = false;
        if !self.dance {
            let count = self.max_level;
            self.load(0, count, false);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_and_restart_match_original_browser_host() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/coding-host-reference.json")).unwrap();
        for case in data["cases"].as_array().unwrap() {
            let dance = case["scene"] == "codeboogie";
            let mut model = CodingHost::new(dance, case["count"].as_u64().unwrap() as usize);
            for row in case["actions"].as_array().unwrap() {
                match row["action"].as_str().unwrap() {
                    "next" => {
                        model.advance();
                    }
                    "restart" => {
                        model.restart_from_host();
                    }
                    "initial" => {}
                    _ => unreachable!(),
                };
                let state = serde_json::to_value(&model).unwrap();
                assert_eq!(state, row["state"], "{} {}", case["scene"], row["action"]);
                assert_eq!(
                    model.gameover,
                    row["host"]["overlayVisible"].as_bool().unwrap()
                );
                assert_eq!(model.gameover, row["host"]["disabled"].as_bool().unwrap());
                assert_eq!(
                    if model.gameover {
                        Some("restart")
                    } else {
                        None
                    },
                    row["host"]["action"].as_str()
                );
            }
        }
    }
}
