//! Inspect the native Christmas tracker controller without a map renderer.
//! Accepts Unix milliseconds (default: real time) and a UTC offset in seconds.
use santa_tracker::{
    tracker::{self, Route},
    tracker_control::{Layout, Tracker},
};
fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let now = args
        .next()
        .map(|s| s.parse::<u64>().map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        });
    let timezone = args
        .next()
        .map(|s| s.parse::<f64>().map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(0.);
    if !timezone.is_finite() || timezone.abs() > 86400. {
        return Err("Invalid UTC offset in seconds".into());
    }
    let (year, _) = tracker::year_start(now / 1000);
    let route = Route::bundled(year)?;
    let range = route.range();
    let mut tracker = Tracker::new(
        route,
        Layout {
            width: 1440.,
            height: 800.,
            info_top: 600.,
            photos_open: false,
        },
    );
    tracker.timezone_seconds = timezone;
    tracker.local_christmas_midnight_ms = range.0 + 14. * 3600000. - timezone * 1000.;
    let frame = tracker.tick(now, 0.);
    println!(
        "{}",
        serde_json::json!({"visible":frame.is_some(),"window_ms":range,"controller":frame})
    );
    Ok(())
}
