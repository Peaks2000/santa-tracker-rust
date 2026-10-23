//! maputils.js DataManager, route skew and spherical helpers, translated to Rust.
//! The bundled route is the original configured endpoint's schedule, not GPS data.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}
pub const NORTH_POLE: Location = Location {
    lat: 84.6,
    lng: 168.,
};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stop {
    pub id: String,
    pub arrival: f64,
    pub departure: f64,
    #[serde(rename = "presentsDelivered")]
    pub presents: f64,
    pub city: String,
    pub region: String,
    pub location: Location,
    pub details: StopDetails,
    #[serde(default)]
    pub distance: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StopDetails {
    pub timezone: f64,
}
#[derive(Debug, Serialize)]
pub struct Details<'a> {
    pub raw: &'a Stop,
    #[serde(rename = "visibleTo")]
    pub visible_to: usize,
    pub location: Location,
    pub presents: f64,
    pub distance: f64,
    pub heading: f64,
    pub stop: bool,
    pub home: bool,
    pub next: f64,
}
pub struct Route {
    pub stops: Vec<Stop>,
    year: u64,
    index: usize,
    time: f64,
}
pub fn leap(year: u64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
pub fn year_start(seconds: u64) -> (u64, u64) {
    let (mut year, mut start) = (1970, 0);
    while start + (if leap(year) { 366 } else { 365 }) * 86400 <= seconds {
        start += (if leap(year) { 366 } else { 365 }) * 86400;
        year += 1;
    }
    (year, start)
}
pub fn takeoff(year: u64) -> u64 {
    let mut start = 0;
    for y in 1970..year {
        start += (if leap(y) { 366 } else { 365 }) * 86400;
    }
    start + (334 + u64::from(leap(year)) + 23) * 86400 + 10 * 3600
}
/// User-requested visibility: the 25-hour delivery flight only. The web source
/// additionally has a Dec 1-25 season guard AND a remotely configured show flag.
pub fn flight_window(now_ms: u64) -> (u64, u64) {
    let (year, _) = year_start(now_ms / 1000);
    let from = takeoff(year) * 1000;
    (from, from + 25 * 3600 * 1000)
}
pub fn visible(now_ms: u64) -> bool {
    let (from, to) = flight_window(now_ms);
    now_ms >= from && now_ms < to
}
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
fn ratio(a: f64, b: f64, t: f64) -> f64 {
    (t - a) / (b - a)
}
pub fn angle(a: Location, b: Location) -> f64 {
    let (al, bl) = (a.lat.to_radians(), b.lat.to_radians());
    2. * (((al - bl) / 2.).sin().powi(2)
        + al.cos()
            * bl.cos()
            * ((a.lng.to_radians() - b.lng.to_radians()) / 2.)
                .sin()
                .powi(2))
    .sqrt()
    .asin()
}
pub fn distance(a: Location, b: Location) -> f64 {
    angle(a, b) * 6378137.
}
pub fn heading(a: Location, b: Location) -> f64 {
    let (al, bl, dl) = (
        a.lat.to_radians(),
        b.lat.to_radians(),
        b.lng.to_radians() - a.lng.to_radians(),
    );
    let h = (dl.sin() * bl.cos()).atan2(al.cos() * bl.sin() - al.sin() * bl.cos() * dl.cos())
        / (std::f64::consts::PI / 180.);
    ((h + 180.) % 360. + 360.) % 360. - 180.
}
pub fn interpolate(a: Location, b: Location, f: f64) -> Location {
    let (al, ag, bl, bg) = (
        a.lat.to_radians(),
        a.lng.to_radians(),
        b.lat.to_radians(),
        b.lng.to_radians(),
    );
    let theta = angle(a, b);
    let s = theta.sin();
    if s < 1e-6 {
        return a;
    }
    let (x, y) = (((1. - f) * theta).sin() / s, (f * theta).sin() / s);
    let vx = x * al.cos() * ag.cos() + y * bl.cos() * bg.cos();
    let vy = x * al.cos() * ag.sin() + y * bl.cos() * bg.sin();
    let vz = x * al.sin() + y * bl.sin();
    Location {
        lat: vz.atan2((vx * vx + vy * vy).sqrt()) / (std::f64::consts::PI / 180.),
        lng: vy.atan2(vx) / (std::f64::consts::PI / 180.),
    }
}
impl Route {
    /// UI entry point: never expose the delivery tracker outside its flight.
    pub fn visible_details(&mut self, now_ms: u64) -> Option<Details<'_>> {
        if !visible(now_ms) {
            return None;
        }
        let (year, _) = year_start(now_ms / 1000);
        if year != self.year {
            let shift = takeoff(year) as f64 * 1000. - self.stops[0].departure;
            for stop in &mut self.stops {
                if stop.arrival != 0. {
                    stop.arrival += shift;
                }
                if stop.departure != 0. {
                    stop.departure += shift;
                }
            }
            self.year = year;
            self.index = 0;
            self.time = 0.;
        }
        self.set_now(now_ms as f64);
        Some(self.details())
    }
    pub fn range(&self) -> (f64, f64) {
        let from = self.stops[0].departure;
        (
            from,
            if self.stops.len() == 1 {
                from
            } else {
                self.stops.last().unwrap().arrival
            },
        )
    }
    /// Source timezone fallback takes local Dec 25 midnight and the user's UTC
    /// offset in seconds. Supplying these keeps locale/calendar policy outside
    /// the route calculations, as it is in the browser's Date implementation.
    pub fn closest_arrival_by_timezone(&self, offset_seconds: f64, midnight_ms: f64) -> f64 {
        self.stops
            .iter()
            .min_by(|a, b| {
                (offset_seconds - a.details.timezone)
                    .abs()
                    .total_cmp(&(offset_seconds - b.details.timezone).abs())
                    .then_with(|| {
                        (midnight_ms - a.arrival)
                            .abs()
                            .total_cmp(&(midnight_ms - b.arrival).abs())
                    })
            })
            .unwrap()
            .arrival
    }
    pub fn bundled(year: u64) -> Result<Self, String> {
        Self::from_json(include_str!("../data/tracker-route.json"), year)
    }
    pub fn from_json(json: &str, year: u64) -> Result<Self, String> {
        #[derive(Deserialize)]
        struct Data {
            destinations: Vec<Stop>,
        }
        let mut stops: Vec<Stop> = serde_json::from_str::<Data>(json)
            .map_err(|e| e.to_string())?
            .destinations;
        if stops.is_empty() {
            stops.push(Stop {
                id: "takeoff".into(),
                arrival: 0.,
                departure: takeoff(year) as f64 * 1000.,
                presents: 0.,
                city: "Santa's Village".into(),
                region: "North Pole".into(),
                location: NORTH_POLE,
                details: StopDetails { timezone: 0. },
                distance: 0.,
            });
        } else {
            let actual_from = stops[0].departure;
            let actual_to = stops.last().unwrap().arrival;
            if !actual_from.is_finite() || !actual_to.is_finite() || actual_to <= actual_from {
                return Err("Invalid route time range".into());
            }
            let from = takeoff(year) as f64 * 1000.;
            let to = from + 25. * 3600. * 1000.;
            let skew = |v: f64| {
                if v == 0. {
                    0.
                } else {
                    lerp(from, to, ratio(actual_from, actual_to, v))
                }
            };
            let mut previous = stops[0].location;
            let mut total = 0.;
            for s in &mut stops {
                if ![
                    s.arrival,
                    s.departure,
                    s.presents,
                    s.location.lat,
                    s.location.lng,
                    s.details.timezone,
                ]
                .iter()
                .all(|v| v.is_finite())
                    || s.location.lat.abs() > 90.
                    || s.location.lng.abs() > 180.
                {
                    return Err("Invalid route destination".into());
                }
                s.arrival = skew(s.arrival);
                s.departure = skew(s.departure);
                total += distance(previous, s.location);
                s.distance = total;
                previous = s.location;
            }
        }
        Ok(Self {
            stops,
            year,
            index: 0,
            time: 0.,
        })
    }
    pub fn set_now(&mut self, time: f64) {
        let time = if time.is_finite() { time } else { 0. };
        if time == self.time {
            return;
        }
        if time < self.time {
            self.index = 0;
        }
        self.time = time;
        let mut i = self.index;
        while i < self.stops.len() {
            if time <= self.stops[i].departure {
                self.index = i;
                break;
            }
            i += 1;
        }
        if i == self.stops.len() {
            self.index = self.stops.len() - 1;
        }
    }
    pub fn details(&self) -> Details<'_> {
        let curr = &self.stops[self.index];
        if self.index == 0 {
            return Details {
                raw: curr,
                visible_to: 0,
                location: NORTH_POLE,
                presents: 0.,
                distance: 0.,
                heading: 0.,
                stop: true,
                home: true,
                next: (curr.departure - self.time).max(0.),
            };
        }
        let prev = &self.stops[self.index - 1];
        if self.time >= curr.arrival {
            let r = 0.3 + ratio(curr.arrival, curr.departure, self.time) * 0.7;
            Details {
                raw: curr,
                visible_to: self.index,
                location: curr.location,
                presents: (lerp(prev.presents, curr.presents, r) + 0.5).floor(),
                distance: curr.distance,
                heading: heading(prev.location, curr.location),
                stop: true,
                home: self.index + 1 >= self.stops.len(),
                next: curr.departure - self.time,
            }
        } else {
            let f = ratio(prev.departure, curr.arrival, self.time);
            let loc = interpolate(prev.location, curr.location, f);
            Details {
                raw: curr,
                visible_to: self.index - 1,
                location: loc,
                presents: (lerp(prev.presents, curr.presents, f * 0.3) + 0.5).floor(),
                distance: prev.distance + distance(prev.location, loc),
                heading: heading(prev.location, curr.location),
                stop: false,
                home: false,
                next: curr.arrival - self.time,
            }
        }
    }
    /// Source closestArrival geographic branch; the home stop is skipped.
    pub fn closest_arrival(&self, location: Location) -> f64 {
        let mut best = 100.;
        let mut index = 0;
        for (i, s) in self.stops.iter().enumerate().skip(1) {
            let a = angle(location, s.location);
            if a < best {
                best = a;
                index = i;
            }
        }
        self.stops[index].arrival
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delivery_visibility_boundaries_and_years() {
        for year in [2024, 2026, 2027, 2100] {
            let from = takeoff(year) * 1000;
            assert!(!visible(from - 1));
            assert!(visible(from));
            assert!(visible(from + 25 * 3600 * 1000 - 1));
            assert!(!visible(from + 25 * 3600 * 1000));
            assert!(!visible(from - 23 * 86400 * 1000));
        }
    }
    #[test]
    fn public_tracking_entry_is_hidden_off_season_and_rebases_next_year() {
        let mut route = Route::bundled(2026).unwrap();
        assert!(route.visible_details(takeoff(2026) * 1000 - 1).is_none());
        assert!(route.visible_details(takeoff(2026) * 1000 + 1000).is_some());
        assert!(route
            .visible_details((takeoff(2026) + 25 * 3600) * 1000)
            .is_none());
        let now = takeoff(2027) * 1000 + 3600000;
        let actual = serde_json::to_value(route.visible_details(now).unwrap()).unwrap();
        let mut fresh = Route::bundled(2027).unwrap();
        assert_eq!(
            actual,
            serde_json::to_value(fresh.visible_details(now).unwrap()).unwrap()
        );
    }
    #[test]
    fn original_route_matches_javascript_details() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../data/tracker-reference.json")).unwrap();
        let mut route = Route::bundled(2026).unwrap();
        for case in cases.as_array().unwrap() {
            route.set_now(case["now"].as_f64().unwrap());
            let actual = serde_json::to_value(route.details()).unwrap();
            let expected = &case["details"];
            for key in ["visibleTo", "stop", "home"] {
                assert_eq!(actual[key], expected[key], "{key}");
            }
            assert_eq!(actual["presents"].as_f64(), expected["presents"].as_f64());
            for key in ["distance", "heading", "next"] {
                assert!(
                    (actual[key].as_f64().unwrap() - expected[key].as_f64().unwrap()).abs() < 1e-6,
                    "{key} at {}: {} vs {}",
                    case["now"],
                    actual[key],
                    expected[key]
                );
            }
            for key in ["lat", "lng"] {
                assert!(
                    (actual["location"][key].as_f64().unwrap()
                        - expected["location"][key].as_f64().unwrap())
                    .abs()
                        < 1e-9
                );
            }
            assert_eq!(actual["raw"]["id"], expected["raw"]["id"]);
        }
    }
}
