//! Headless seasonal/route check; accepts Unix milliseconds, defaults to now.
//! This exercises route logic, not the unfinished native map UI.
fn main() -> Result<(), String> {
    let now = std::env::args()
        .nth(1)
        .map(|s| s.parse::<u64>().map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        });
    let (year, _) = santa_tracker::tracker::year_start(now / 1000);
    let mut route = santa_tracker::tracker::Route::bundled(year)?;
    let range = route.range();
    let details = route.visible_details(now);
    println!(
        "{}",
        serde_json::json!({"visible":details.is_some(),"window_ms":range,"details":details})
    );
    Ok(())
}
