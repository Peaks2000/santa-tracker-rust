//! Native state/commands from modvil-tracker.js and maputils.js StopManager.
//! A map renderer consumes these commands; this module does not provide map tiles.
use crate::tracker::{Details, Location, Route, Stop};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Trail {
    pub opacity: f64,
    pub path: Vec<Location>,
}
#[derive(Debug, Serialize)]
pub struct Markers {
    pub visible: Vec<bool>,
    pub trails: Vec<Trail>,
    pub active: Vec<Location>,
    visible_to: usize,
}
impl Markers {
    pub fn new(route: &Route) -> Self {
        let mut visible = vec![false; route.stops.len()];
        visible[0] = true;
        Self {
            visible,
            trails: (0..20)
                .map(|i| Trail {
                    opacity: 1. - i as f64 / 20.,
                    path: vec![],
                })
                .collect(),
            active: vec![],
            visible_to: 0,
        }
    }
    pub fn update(&mut self, route: &Route) {
        let d = route.details();
        if d.visible_to < self.visible_to {
            self.visible[d.visible_to + 1..=self.visible_to].fill(false);
        } else {
            self.visible[self.visible_to..=d.visible_to].fill(true);
        }
        // Source only rebuilds the faded trail when the visible stop changes.
        if d.visible_to != self.visible_to {
            let usable = if d.home { 0 } else { d.visible_to.min(20) };
            for (i, trail) in self.trails.iter_mut().enumerate() {
                trail.path = if i < usable {
                    vec![
                        route.stops[d.visible_to - i].location,
                        route.stops[d.visible_to - i - 1].location,
                    ]
                } else {
                    vec![]
                };
            }
        }
        self.active = if d.stop {
            vec![]
        } else {
            vec![route.stops[d.visible_to].location, d.location]
        };
        self.visible_to = d.visible_to;
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub width: f64,
    pub height: f64,
    pub info_top: f64,
    pub photos_open: bool,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Camera {
    pub location: Location,
    pub zoom: f64,
    pub pan_y: f64,
}
impl Layout {
    pub fn camera(&self, location: Location, offset: f64) -> Camera {
        let usable = self.info_top - if self.photos_open { 100. } else { 0. }
            + if self.width > 600. { 40. } else { 0. };
        let min = (self.width.min(1024.) + usable) / 2.;
        Camera {
            location,
            zoom: ((min / 160.).round() + offset).clamp(2., 6.),
            pan_y: (self.height - usable) / 2.,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ClockAdjustment {
    pub offset_ms: f64,
    pub jitter_ratio: f64,
    pub jitter_seconds: f64,
}
impl ClockAdjustment {
    pub fn now(&self, actual_ms: f64) -> f64 {
        actual_ms + self.offset_ms + self.jitter_ratio * self.jitter_seconds * 1000.
    }
}
#[derive(Debug, Serialize)]
pub struct Frame<'a> {
    pub details: Details<'a>,
    pub markers: &'a Markers,
    pub focus_on_santa: bool,
    pub camera: Camera,
    pub destination: &'a Stop,
    pub stop: &'a Stop,
    pub feed_stops: Vec<&'a str>,
    pub santa_hidden: bool,
    pub gesture_handling: &'static str,
    pub zoom_control: bool,
    pub arrival_ms: f64,
    pub stats: Stats,
}

pub struct Tracker {
    pub route: Route,
    pub markers: Markers,
    pub adjustment: ClockAdjustment,
    pub layout: Layout,
    pub user_location: Option<Location>,
    /// Date-derived timezone at local Christmas midnight, in source seconds.
    pub timezone_seconds: f64,
    pub local_christmas_midnight_ms: f64,
    focus: bool,
    refocus_at: Option<f64>,
    temporary: Option<usize>,
    camera: Camera,
    next_sample: Option<f64>,
    exposed: bool,
}
impl Tracker {
    pub fn new(route: Route, layout: Layout) -> Self {
        let markers = Markers::new(&route);
        let midnight = route.range().0 + 14. * 3600000.;
        Self {
            route,
            markers,
            adjustment: ClockAdjustment::default(),
            layout,
            user_location: None,
            timezone_seconds: 0.,
            local_christmas_midnight_ms: midnight,
            focus: true,
            refocus_at: None,
            temporary: None,
            camera: layout.camera(crate::tracker::NORTH_POLE, -10.),
            next_sample: None,
            exposed: false,
        }
    }
    fn arrival(&self) -> f64 {
        if let Some(location) = self.user_location {
            self.route.closest_arrival(location)
        } else {
            self.route.closest_arrival_by_timezone(
                self.timezone_seconds,
                self.local_christmas_midnight_ms,
            )
        }
    }
    fn update_camera(&mut self) {
        if self.focus {
            let d = self.route.details();
            self.camera = self.layout.camera(
                d.location,
                if d.home {
                    -10.
                } else if d.stop {
                    1.
                } else {
                    0.
                },
            );
        }
    }
    /// wall_ms is monotonic; actual_ms is the real calendar clock. Visibility is
    /// deliberately gated by the unadjusted calendar, never the jitter/debug offset.
    pub fn tick(&mut self, actual_ms: u64, wall_ms: f64) -> Option<Frame<'_>> {
        self.exposed = crate::tracker::visible(actual_ms);
        if !self.exposed {
            self.next_sample = None;
            self.temporary = None;
            self.focus = true;
            self.refocus_at = None;
            return None;
        }
        let (year, _) = crate::tracker::year_start(actual_ms / 1000);
        let previous_takeoff = self.route.range().0;
        let rebased = self.route.rebase_year(year);
        if rebased {
            self.local_christmas_midnight_ms += self.route.range().0 - previous_takeoff;
            self.markers = Markers::new(&self.route);
            self.next_sample = None;
            self.temporary = None;
        }
        if self.next_sample.is_none_or(|next| wall_ms >= next) {
            self.route.set_now(self.adjustment.now(actual_ms as f64));
            self.markers.update(&self.route);
            self.next_sample = Some(
                self.next_sample
                    .map(|next| next + ((wall_ms - next) / 1000.).floor() * 1000. + 1000.)
                    .unwrap_or(wall_ms + 1000.),
            );
            self.update_camera();
        }
        if self.refocus_at.is_some_and(|due| wall_ms >= due) {
            self.focus_on_santa();
        }
        self.frame()
    }
    pub fn resize(&mut self, layout: Layout) {
        self.layout = layout;
        if layout.width <= 600. {
            self.refocus_at = None;
            self.focus_on_santa();
        }
    }
    /// User pan/zoom events only. Programmatic focus/resize must not call this.
    pub fn map_changed(&mut self, wall_ms: f64) {
        if self.focus {
            self.focus = false;
            self.refocus_at = Some(wall_ms + 20000.);
        }
    }
    pub fn focus_on_santa(&mut self) {
        if self.focus {
            return;
        }
        self.focus = true;
        self.refocus_at = None;
        self.temporary = None;
        self.update_camera();
    }
    pub fn marker_click(&mut self, id: &str, wall_ms: f64) {
        if !self.exposed {
            return;
        }
        // Original checks this.width (undefined), not this._width, so its
        // mobile early-return is ineffective. Keep that source quirk here.
        let Some(index) = self.route.stops.iter().rposition(|s| s.id == id) else {
            return;
        };
        if index == self.route.details().visible_to {
            self.focus_on_santa();
            return;
        }
        self.map_changed(wall_ms);
        self.temporary = Some(index);
        self.camera = self.layout.camera(self.route.stops[index].location, 2.);
    }
    pub fn frame(&self) -> Option<Frame<'_>> {
        if !self.exposed {
            return None;
        }
        let details = self.route.details();
        let arrival_ms = self.arrival() - self.route.now();
        let stats = Stats::new(&details, arrival_ms);
        let stop = &self.route.stops[details.visible_to];
        Some(Frame {
            destination: self
                .temporary
                .map(|i| &self.route.stops[i])
                .unwrap_or(details.raw),
            stop,
            feed_stops: self.route.feed_stops(details.visible_to),
            santa_hidden: details.home,
            markers: &self.markers,
            focus_on_santa: self.focus,
            camera: self.camera,
            gesture_handling: if self.layout.width <= 600. {
                "none"
            } else {
                "auto"
            },
            zoom_control: self.layout.width > 600.,
            arrival_ms,
            stats,
            details,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Stats {
    pub show_arrival_hours: bool,
    pub arrival: String,
    pub distance_km: String,
    pub gifts: String,
    pub current_stop: bool,
    pub city: String,
    pub region: String,
    pub countdown: String,
}
impl Stats {
    pub fn new(d: &Details<'_>, arrival_ms: f64) -> Self {
        Self {
            show_arrival_hours: arrival_ms > 600000.,
            arrival: format_arrival(arrival_ms),
            distance_km: format_number((d.distance / 1000.).round(), "km"),
            gifts: format_number(d.presents, ""),
            current_stop: d.stop,
            city: d.raw.city.clone(),
            region: d.raw.region.clone(),
            countdown: format_duration(d.next),
        }
    }
}
pub fn format_duration(ms: f64) -> String {
    if !(ms >= 0.) || !ms.is_finite() {
        return "?".into();
    }
    let s = (ms / 1000.).floor();
    let days = (s / 86400.).floor();
    if days > 365. {
        "?".into()
    } else if days > 0. {
        format!("{days}d")
    } else if s >= 3600. {
        format!(
            "{:02}:{:02}:{:02}",
            (s / 3600.) as u64 % 24,
            (s / 60.) as u64 % 60,
            s as u64 % 60
        )
    } else {
        format!("{}:{:02}", (s / 60.) as u64 % 60, s as u64 % 60)
    }
}
/// English text is exported from the original messages; other locales need their own formatter.
pub fn format_arrival(ms: f64) -> String {
    let hours = ms / 3600000.;
    if (hours + 0.5).floor() <= 1. {
        return "Santa will arrive soon".into();
    }
    let mut whole = hours.floor();
    let fraction = hours % 1.;
    let mut suffix = "";
    if fraction > 0.75 {
        whole += 1.;
    } else if fraction > 0.25 {
        suffix = "½";
    }
    format!("Santa will arrive in about {whole}{suffix} hours")
}
pub fn format_number(v: f64, suffix: &str) -> String {
    if !(v >= 0.) || !v.is_finite() {
        return "?".into();
    }
    let formatted = format!("{v:.3}");
    let formatted = formatted.trim_end_matches('0').trim_end_matches('.');
    let (whole, fraction) = formatted.split_once('.').unwrap_or((formatted, ""));
    let mut result = String::new();
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    if !fraction.is_empty() {
        result.push('.');
        result.push_str(fraction);
    }
    result.push_str(suffix);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    fn reference() -> Value {
        serde_json::from_str(include_str!("../data/tracker-control-reference.json")).unwrap()
    }
    fn compare(a: &Value, b: &Value, path: &str) {
        match (a, b) {
            (Value::Object(a), Value::Object(b)) => {
                for (k, v) in b {
                    compare(&a[k], v, &format!("{path}.{k}"));
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len(), "{path}");
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}[{i}]"));
                }
            }
            (Value::Number(a), Value::Number(b)) => assert!(
                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-7,
                "{path}: {a} vs {b}"
            ),
            _ => assert_eq!(a, b, "{path}"),
        }
    }
    #[test]
    fn all_original_stop_manager_updates() {
        let mut route = Route::bundled(2026).unwrap();
        let mut markers = Markers::new(&route);
        for case in reference()["markerCases"].as_array().unwrap() {
            route.set_now(case["now"].as_f64().unwrap());
            markers.update(&route);
            let visible: String = markers
                .visible
                .iter()
                .map(|v| if *v { '1' } else { '0' })
                .collect();
            assert_eq!(
                visible,
                case["visible"].as_str().unwrap(),
                "at {}",
                case["now"]
            );
            let trails: Vec<Vec<usize>> = markers
                .trails
                .iter()
                .map(|t| {
                    t.path
                        .iter()
                        .map(|p| {
                            route
                                .stops
                                .iter()
                                .rposition(|s| s.location.lat == p.lat && s.location.lng == p.lng)
                                .unwrap()
                        })
                        .collect()
                })
                .collect();
            assert_eq!(json!(trails), case["trails"], "at {}", case["now"]);
            compare(&json!(markers.active), &case["active"], "active");
            for (i, t) in markers.trails.iter().enumerate() {
                assert_eq!(t.opacity, 1. - i as f64 / 20.);
            }
        }
    }
    #[test]
    fn original_tracker_formatting_and_arrival_estimates() {
        let r = reference();
        for case in r["clocks"].as_array().unwrap() {
            let clock = ClockAdjustment {
                offset_ms: case["offset"].as_f64().unwrap(),
                jitter_ratio: case["ratio"].as_f64().unwrap(),
                jitter_seconds: case["jitter"].as_f64().unwrap(),
            };
            assert_eq!(
                clock.now(case["actual"].as_f64().unwrap()),
                case["now"].as_f64().unwrap()
            );
        }

        for case in r["formatting"].as_array().unwrap() {
            let ms = case["ms"].as_f64().unwrap();
            assert_eq!(
                format_duration(ms),
                case["duration"].as_str().unwrap(),
                "{ms}"
            );
            assert_eq!(
                format_arrival(ms),
                case["arrival"].as_str().unwrap(),
                "{ms}"
            );
        }
        for case in r["numbers"].as_array().unwrap() {
            assert_eq!(
                format_number(case["value"].as_f64().unwrap(), "km"),
                case["result"].as_str().unwrap()
            );
        }
        let route = Route::bundled(2026).unwrap();
        for case in r["arrivals"].as_array().unwrap() {
            let actual = if case["location"].is_object() {
                route.closest_arrival(serde_json::from_value(case["location"].clone()).unwrap())
            } else {
                route.closest_arrival_by_timezone(
                    case["offset"].as_f64().unwrap(),
                    case["midnight"].as_f64().unwrap(),
                )
            };
            assert!((actual - case["arrival"].as_f64().unwrap()).abs() < 1e-6);
        }
    }
    fn state(t: &Tracker) -> Value {
        let f = t.frame().unwrap();
        json!({"focus":f.focus_on_santa,"temporary":t.temporary.map(|i|&t.route.stops[i].id),"destination":f.destination.id,"stop":f.stop.id,"feed":f.feed_stops,"camera":f.camera,"hidden":f.santa_hidden,"gesture":f.gesture_handling,"zoomControl":f.zoom_control})
    }
    #[test]
    fn original_controller_focus_click_zoom_and_resize() {
        let mut t = Tracker::new(
            Route::bundled(2026).unwrap(),
            Layout {
                width: 1440.,
                height: 800.,
                info_top: 600.,
                photos_open: false,
            },
        );
        let mut wall = 0.;
        let mut now = crate::tracker::takeoff(2026) * 1000;
        t.tick(now, wall);
        for (i, row) in reference()["controller"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let a = &row["action"];
            if let Some(w) = a["wall"].as_f64() {
                wall = w;
            }
            match a["type"].as_str().unwrap() {
                "initial" => {}
                "now" => {
                    now = a["now"].as_f64().unwrap() as u64;
                    t.route.set_now(now as f64);
                    t.markers.update(&t.route);
                    t.update_camera();
                }
                "click" => t.marker_click(a["id"].as_str().unwrap(), wall),
                "pan" => t.map_changed(wall),
                "focus" => t.focus_on_santa(),
                "resize" => t.resize(Layout {
                    width: a["width"].as_f64().unwrap(),
                    ..t.layout
                }),
                "photos" => {
                    t.layout.photos_open = a["open"].as_bool().unwrap();
                    t.update_camera();
                }
                "wait" => {
                    if t.refocus_at.is_some_and(|due| wall >= due) {
                        t.focus_on_santa();
                    }
                }
                x => panic!("unexpected {x}"),
            }
            compare(&state(&t), &row["state"], &format!("controller {i} {a}"));
        }
    }
    #[test]
    fn christmas_clock_gate_sampling_and_new_year() {
        let from = crate::tracker::takeoff(2026) * 1000;
        let mut t = Tracker::new(
            Route::bundled(2026).unwrap(),
            Layout {
                width: 1440.,
                height: 800.,
                info_top: 600.,
                photos_open: false,
            },
        );
        t.adjustment = ClockAdjustment {
            offset_ms: 1234.,
            jitter_ratio: -0.25,
            jitter_seconds: 20.,
        };
        assert!(t.tick(from - 1, 0.).is_none());
        assert!(t.frame().is_none());
        assert!(t.tick(from, 10.).is_some());
        let first = t.route.now();
        assert_eq!(first, from as f64 + 1234. - 5000.);
        t.tick(from + 999, 1009.);
        assert_eq!(t.route.now(), first);
        t.tick(from + 1000, 1010.);
        assert_eq!(t.route.now(), first + 1000.);
        // Timer cadence remains anchored after a late native frame.
        t.tick(from + 2100, 2110.);
        let sample = t.route.now();
        t.tick(from + 2999, 3009.);
        assert_eq!(t.route.now(), sample);
        t.tick(from + 3000, 3010.);
        assert_eq!(t.route.now(), first + 3000.);
        assert!(t.tick(from + 25 * 3600000 - 1, 90000000.).is_some());
        assert!(t.tick(from + 25 * 3600000, 90000001.).is_none());
        assert!(t.frame().is_none());
        let next = crate::tracker::takeoff(2027) * 1000 + 3600000;
        t.tick(next, 91000000.);
        let mut fresh = Route::bundled(2027).unwrap();
        fresh.set_now(t.adjustment.now(next as f64));
        compare(
            &json!(t.route.details()),
            &json!(fresh.details()),
            "new year",
        );
        assert_eq!(
            t.local_christmas_midnight_ms,
            crate::tracker::takeoff(2027) as f64 * 1000. + 14. * 3600000.
        );
    }
    #[test]
    fn no_route_fallback_and_photo_metadata() {
        let mut t = Tracker::new(
            Route::from_json("{\"destinations\":[]}", 2026).unwrap(),
            Layout {
                width: 375.,
                height: 800.,
                info_top: 450.,
                photos_open: false,
            },
        );
        let f = t.tick(crate::tracker::takeoff(2026) * 1000, 0.).unwrap();
        assert!(f.feed_stops.is_empty());
        assert!(f.santa_hidden);
        assert_eq!(f.details.raw.id, "takeoff");
        let r = Route::bundled(2026).unwrap();
        assert!(
            r.stops[0].details.metadata["photos"]
                .as_array()
                .unwrap()
                .len()
                > 0
        );
    }
}
