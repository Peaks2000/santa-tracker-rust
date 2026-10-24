---
name: maintain-rust-santa-tracker
description: Maintain this user's native Rust Santa Tracker rewrite, using their editable parity feedback, original source comparisons, and Rust-only GitHub distribution. Use for work on this project and its modes, rendering, input, audio, performance, or progress tracking.
---

# Native Santa Tracker maintenance

The product is a native Rust recreation of the entire original Google Santa Tracker. The user wants original behavior, art, animations, proportions and sounds. A browser/webview wrapper or using the original JS engine is not an acceptable implementation. Original SVG, Lottie JSON, images, models and Ogg files are assets; native libraries and GLSL shaders are already used.

## Find the project and feedback

- Development/reference checkout: `/home/finn/rustsantatracker`, native crate `rust/`. Original source is in `static/` and `static/entrypoint.js` here.
- Clean distribution checkout: `/home/finn/santa-tracker-rust`, public GitHub `Peaks2000/santa-tracker-rust`. The development checkout's origin is the original web repository; **never push that history to the Rust repository**.
- In another checkout, locate `rust/Cargo.toml`, `rust/PARITY-STATUS.json` and `rust/SOURCE-COVERAGE.md`; use paths relative to that checkout. Find/fetch original source separately if needed, keeping the distributed repository free of browser code.

Read `PARITY-STATUS.json` before choosing work. The user edits `user_overall_happy`, `user_overall_notes`, each mode's `user_happy`, `user_notes`, and `priority`. Preserve these and unknown user-added fields exactly in meaning. Only the user can approve a mode; tests or agent judgment do not turn an unreviewed/no/partly value into yes. Read the latest file again before updating agent fields so concurrent user edits survive. Keep feedback in one maintained file, not duplicate trackers. Regenerate the human-readable `PARITY-REPORT.txt` with `cargo run --manifest-path rust/Cargo.toml --example status_report` after updating progress; that report is derived, and user feedback is edited in the JSON file.

Read `SOURCE-COVERAGE.md` for existing checks and parity gaps. Work on explicit user corrections first, then their priority/notes, then shared bugs and remaining native modes. Record concrete progress, unresolved issues and evidence in the mode's `agent_*` fields. Keep incomplete modes marked partial/not_started as appropriate.

## No glossing over source code

For each selected scene, read its complete JavaScript, HTML and styles **before implementing the corresponding behavior**, not merely before reporting completion. Use bounded reads and check file lengths so truncated tool output is not mistaken for a complete review. Search snippets locate code; they do not replace reading it. Account for every function and branch in `SOURCE-COVERAGE.md`: native equivalent, verified inactive/browser-only behavior, or an explicit unfinished item. Do not silently skip code because the first playable result looks plausible.

Trace callbacks, event registration/removal, initialization order, defaults, units, source assets and active shared/engine methods. Establish the actual physics integration order, body/anchor transforms, collision rules, camera rounding and update frequency from the bundled engine. Trace the host shell's route/configuration separately from the scene HUD: Home/Menu choice, pause availability, scoreboard and navigation can be independent. Do not infer any of these from intuition, another port or what a game usually does.

Preserve source quirks unless the user explicitly requests a behavior change. Check random consumption across initialization and full restart, pooled state, delayed callbacks surviving restart, cancellation, paused wall timers, hidden animations and the old/new scene during transitions. Translate original formulas and timing directly; do not add level caps, replace acceleration with a preferred model, smooth away a tick delay, invent controls or substitute artwork/icons. Record source bugs and the handling of them explicitly.

Validate behavior against the original code and, when relevant, its actual bundled engine. A mock of engine behavior cannot establish engine parity. Document reference-harness seed/clock control and stubs; do not patch the behavior under comparison to make it match Rust. Compare matching actions, seed, viewport and time, including intermediate animation phases, pressed/released/hover/focus states, pause/death/restart overlap, collisions and resize. Visually inspect original/native captures and difference images; a low whole-image error or an idle screenshot cannot certify every moving object, animation or interaction. Check sound selection, looping, gain, fades, scheduling and pause behavior as well as sample identity. If something cannot be checked, mark it unverified and keep working on the parts that can be checked.

## README faithfulness categories

Keep the GitHub root README scene list as exactly two columns, **Name** and **State**, with concise category values: **1:1**, **Almost 1:1**, **Very similar**, **Partially alike**, **Unplayable**. Keep detailed evidence/issues in the existing feedback file and coverage ledger, not table prose.

- **1:1**: every active behavior, animation, image/proportion, sound and applicable input path has been matched to the original, with no known material differences. Selected passing tests alone never justify this claim.
- **Almost 1:1**: the main experience closely matches, with only small, identified remaining differences.
- **Very similar**: recognizable and playable, with material differences or substantial unverified areas.
- **Partially alike**: only part of the experience is implemented or major flows differ.
- **Unplayable**: the scene is missing or its main flow cannot be played.

Preserve ratings the user has assigned in the README unless they request reassessment. Treat those as the user's ratings; do not silently reinterpret them as comprehensive technical certification or clear unresolved agent evidence. Do not change `user_happy` or other user feedback on the user's behalf. Assign agent-written ratings conservatively and distinguish implementation from verification.

## Source fidelity and performance

Read every function and line of the selected scene's JS, HTML, styles and active shared dependencies before calling its port complete. Track the review in the coverage ledger. Execute the original code only in separate development reference tools; compare native state and browser/native captures at matching seed, dimensions, time and actions. Inspect sprites, transform origins, crop/stacking, timers, pool reuse, keyframe phases and sound actions. Record unused functions and source quirks rather than silently assuming behavior.

The user wants optimizations to preserve gameplay, appearance and animation timing. Profile first; distinguish startup/loading, steady frames, simulation, input sampling, rendering, GPU presentation and audio decoding. Preserve fixed simulation rules unless evidence from the source warrants a correction. Never present frame-time measurements as end-to-end input latency. Keep original asset edges/shadows; fix alpha/filtering/compositing errors instead of removing outlines from the art. Check recommendation-card aspect ratios as well as card box dimensions.

Do not close or replace a window the user is testing. Smoke checks should own and terminate their own process. Screenshots of the original and native app are authorized for this project.

Christmas tracking must only be exposed during the delivery flight (Dec 24 10:00 UTC to Dec 25 11:00 UTC, end exclusive), per the user. The original web season helper additionally permits Dec 1-25 subject to remote config; distinguish that from the user-requested delivery visibility. Route math/gating is currently in `tracker.rs`; its passing oracle tests do not mean the unfinished native map/UI works. Preserve that distinction in reports.

## Build, check and publish

On this machine the isolated toolchain is `/tmp/santa-cargo` and `/tmp/santa-rustup`. Use `RUSTUP_HOME=/tmp/santa-rustup CARGO_HOME=/tmp/santa-cargo CARGO=/tmp/santa-cargo/bin/cargo ./rust/build.sh --offline`. Tests use that cargo with `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/clang/22/include` and `--manifest-path rust/Cargo.toml`. Native screenshot controls and comparison tooling are documented in `rust/README.md`; `--frames` freezes simulation, while `--advance` pre-simulates.

The clean distribution contains native Rust, fixtures, fonts, shader code, licenses and required original assets. Development `rust/tools/*.cjs` and original JS/HTML/CSS/SCSS stay out. Preserve the clean repository's user-edited root README. Before an authorized push, verify staged file names/history and keep GitHub public. The user explicitly corrected the earlier private preference: do not make the repository private again unless they request it. Include this skill's vendored copy and the feedback file in the Rust-only repository; installed skill updates belong in `~/.codex/skills/maintain-rust-santa-tracker` too.

Report tested changes and remaining limits candidly. Passing current snapshots does not establish whole-site or whole-mode 1:1 parity.
