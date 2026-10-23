//! Regenerate a readable report; feedback is edited only in PARITY-STATUS.json.
use std::{fmt::Write, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("PARITY-STATUS.json"))?)?;
    let modes = data["modes"].as_array().ok_or("Missing modes")?;
    let text = |v: &serde_json::Value| v.as_str().unwrap_or("").to_owned();
    let mut out=String::from("SANTA TRACKER RUST — FUNCTIONALITY AND PARITY STATUS\n\nGenerated from PARITY-STATUS.json. Edit that file for your verdicts and notes.\nThis report is evidence/progress, not a claim of complete 1:1 accuracy.\n\n");
    let playable = modes
        .iter()
        .filter(|m| m["agent_status"] == "partial")
        .count();
    writeln!(
        out,
        "Native scenes playable, with parity gaps: {playable}/{}",
        modes.len()
    )?;
    writeln!(out, "Whole modes certified 1:1: 0")?;
    writeln!(
        out,
        "Your overall verdict: {}",
        text(&data["user_overall_happy"])
    )?;
    if !text(&data["user_overall_notes"]).is_empty() {
        writeln!(
            out,
            "Your overall notes: {}",
            text(&data["user_overall_notes"])
        )?;
    }
    out.push_str("\nPLAYABLE — STILL PARTIAL\n\n");
    for m in modes.iter().filter(|m| m["agent_status"] == "partial") {
        writeln!(
            out,
            "{} [{}] — your verdict: {}",
            text(&m["name"]),
            text(&m["id"]),
            text(&m["user_happy"])
        )?;
        writeln!(out, "  Progress: {}", text(&m["agent_progress"]))?;
        if let Some(issues) = m["agent_known_issues"].as_array() {
            for issue in issues {
                writeln!(out, "  Remaining: {}", text(issue))?;
            }
        }
        if !text(&m["user_notes"]).is_empty() {
            writeln!(out, "  Your notes: {}", text(&m["user_notes"]))?;
        }
        out.push('\n');
    }
    out.push_str("CHRISTMAS TRACKING — CORE TESTED; MAP/UI NOT IMPLEMENTED\n\n420 original scheduled stops; 3,020 source-JavaScript state comparisons pass.\nPosition, heading, gifts, distance, stop/flight timing and yearly route skew\nare implemented. Visibility is restricted to Dec 24 10:00 UTC through\nDec 25 11:00 UTC (end exclusive), as requested. Outside this window the\nroute entry point returns no tracking details. This is currently a library\nand headless diagnostic, not a working tracking screen in the application.\nGoogle Maps, stop markers/trails, feed/photos, focus/zoom and online refresh\nare still missing. The bundled data is the original configured endpoint's\n2019 schedule shifted to the selected year, as in the original fetchRoute.\n\nNOT IMPLEMENTED AS NATIVE SCENES\n\n");
    for m in modes.iter().filter(|m| m["agent_status"] == "not_started") {
        writeln!(
            out,
            "- {} [{}] — no native scene; your verdict: {}",
            text(&m["name"]),
            text(&m["id"]),
            text(&m["user_happy"])
        )?;
        if !text(&m["user_notes"]).is_empty() {
            writeln!(out, "  Your notes: {}", text(&m["user_notes"]))?;
        }
    }
    out.push_str("\nSHARED LIMITS\n\nSubtle image/font edges, overlay details and responsive layouts still need\nreview. Cold village image/Lottie loads and first-use Ogg decoding remain\nsynchronous. Frame profiling does not establish input-to-photon latency.\nSource comparison details and outstanding gaps: SOURCE-COVERAGE.md.\n");
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("PARITY-REPORT.txt"));
    std::fs::write(&path, out)?;
    println!("{}", path.display());
    Ok(())
}
