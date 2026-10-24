//! santa-tutorial.js ordered queue, filter, dismissal and delayed display.
use std::collections::HashSet;
#[derive(Debug)]
pub struct Tutorial {
    pub filter: String,
    queue: Vec<String>,
    dismissed: HashSet<String>,
    pub active: Option<String>,
    pub selected_at: f64,
    pub loaded: bool,
    display_at: f64,
}
impl Tutorial {
    pub fn new(filter: &str) -> Self {
        Self {
            filter: filter.into(),
            queue: vec![],
            dismissed: HashSet::new(),
            active: None,
            selected_at: 0.,
            loaded: false,
            display_at: 0.,
        }
    }
    pub fn reset(&mut self) {
        self.queue.clear();
        self.dismissed.clear();
        self.active = None;
        self.loaded = false;
    }
    fn valid(&self, cand: &str) -> bool {
        !self.dismissed.contains(cand)
            && (cand.contains('.')
                || cand.split('-').next() == Some(self.filter.as_str())
                || (cand.starts_with("device-") && self.filter == "touch"))
    }
    fn refresh(&mut self, now: f64) {
        if self
            .active
            .as_ref()
            .is_some_and(|s| !self.dismissed.contains(s))
        {
            return;
        }
        let found = self.queue.iter().find(|s| self.valid(s)).cloned();
        self.active = found;
        self.selected_at = now;
        self.display_at = now + 1.5;
        self.loaded = false;
    }
    pub fn queue(&mut self, names: &[&str], now: f64) {
        let mut changed = false;
        for name in names {
            if !self.queue.iter().any(|s| s == name) {
                self.queue.push((*name).into());
                changed = true;
            }
        }
        if changed {
            self.refresh(now);
        }
    }
    pub fn dismiss(&mut self, names: &[&str], now: f64) {
        let mut changed = false;
        for name in names {
            changed |= self.dismissed.insert((*name).into());
        }
        if changed {
            self.refresh(now);
        }
    }
    pub fn asset_result(&mut self, name: &str, ok: bool, now: f64) {
        if self.active.as_deref() != Some(name) {
            return;
        }
        if ok {
            self.loaded = true;
            self.display_at = (self.selected_at + 1.5).max(now);
        } else {
            self.dismiss(&[name], now);
        }
    }
    pub fn visible(&self, now: f64) -> bool {
        self.active.is_some() && self.loaded && now >= self.display_at
    }
    pub fn animation_time(&self, now: f64) -> f64 {
        (now - self.display_at).max(0.)
    }
    pub fn for_scene(&mut self, scene: &str, now: f64) {
        self.reset();
        let names: &[&str] = match scene {
            "penguindash" => &["device-tilt", "keys-arrows", "spacenav-arrows"],
            "glider" => &[
                "device-tilt",
                "keys-arrows",
                "keys-space",
                "spacenav-arrows",
                "spacenav-space",
            ],
            "runner" => &["touch-updown", "keys-updown", "spacenav-updown"],
            "jetpack" => &[
                "touch-updown",
                "keys-leftright",
                "keys-updown",
                "spacenav-leftright",
                "spacenav-updown",
            ],
            "presentdrop" => &[
                "touch-leftright",
                "keys-space",
                "keys-leftright",
                "spacenav-space",
                "spacenav-leftright",
            ],
            "boatload" => &["touch-updown", "keys-space", "keys-updown"],
            "elfski" => &["elfski.mp4"],
            _ => &[],
        };
        self.queue(names, now);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_original_tutorial_queue_scenarios() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/tutorial-reference.json")).unwrap();
        for case in data.as_array().unwrap() {
            let mut t = Tutorial::new("mouse");
            for step in case["steps"].as_array().unwrap() {
                let now = step["now"].as_f64().unwrap();
                let names: Vec<&str> = step["names"]
                    .as_array()
                    .map(|v| v.iter().map(|s| s.as_str().unwrap()).collect())
                    .unwrap_or_default();
                match step["op"].as_str().unwrap() {
                    "filter" => t.filter = step["value"].as_str().unwrap().into(),
                    "queue" => t.queue(&names, now),
                    "dismiss" => t.dismiss(&names, now),
                    "load" => {
                        let active = t.active.clone().unwrap();
                        t.asset_result(&active, step["ok"].as_bool().unwrap(), now)
                    }
                    "reset" => t.reset(),
                    _ => {}
                }
                assert_eq!(t.active.as_deref(), step["active"].as_str());
                assert_eq!(
                    t.visible(now),
                    step["visible"].as_bool().unwrap(),
                    "{} {:?}",
                    case["name"],
                    step
                );
            }
        }
    }
}
