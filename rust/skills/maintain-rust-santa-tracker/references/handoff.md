# Native Santa Tracker handoff

Read this for onboarding or work on the coding scenes. Project paths below are relative to the repository root unless absolute. The live feedback file and coverage ledger remain authoritative; this reference explains how to continue without repeating discoveries.

## Snapshot and the user's intent

Snapshot: 24 October 2026, after public native commit `c26766c` and local reference commit `4498b64b9`. Refresh Git history/status before relying on these identifiers. The most recent implementation pass added playable native Code Lab and Code Boogie and corrected the Boogie chooser after the user's screenshot comparison. At that point all 67 tests passed (62 library, 5 binary), and a release build succeeded. That count is a baseline, not proof of the current checkout.

The user explicitly chose **Very similar** for Code Boogie, then reiterated that **1:1 remains the target**. Code Lab's README state was **Partially alike**. Preserve later user changes. Technical agent status remains partial: neither whole scene nor the whole site is certified 1:1. The root README already lists the source scenes in a two-column Name/State table; keep detail in the feedback/coverage files.

A subsequent stage-effects pass adds `codeboogie_stage.rs`: original light
selection/transitions, move cards, title/countdown spans, watcher transforms,
strict millisecond sprite clocks and beat/track scheduling. Its durable evidence
is 15 browser scenarios / 2,775 snapshots, 12 DOM states at six viewports, 59 CSS
phase samples and ten matched stills. A live two-block first lesson reaches
Nice moves. Read the stage follow-up section in SOURCE-COVERAGE.md for the
precise scope and remaining gaps; refresh current test/build results before work.

The next pass preserves Boogie View code loops/name allocation instead of
expanding actions. `codeboogie-user-code-reference.json` checks 98 complete
strings from the real bundled Blockly generator. Code is saved at Run time,
not regenerated after editing during playback. The next result-panel pass now matches 70 DOM configurations / seven sizes,
96 CSS phases including reversals, and 12 isolated images. Small raster
differences remain; textarea interaction, sharing and host game-over are still
unfinished. The README rating stays Very similar.

Do not confuse these three things: the user's README rating, their editable approval fields, and the scope of measured technical evidence. Changing one does not automatically change the others.

## Find the implementation and evidence

| Location | Purpose |
| --- | --- |
| `rust/src/coding.rs` | Typed block graph/interpreter, maze execution/results, dance status/queues, source viewport and chooser layout formulas, fixture tests. |
| `rust/src/codeboogie_result.rs`, `coding_result_view.rs` | Source result flex/typography/transitions and live renderer, with layout/phase/raster fixtures. |
| `rust/src/coding_user_code.rs` | Boogie educational code text, loop/name allocation and root ordering, compared with actual Blockly. |
| `rust/src/codeboogie_stage.rs` | Source beat/track state, light shuffling, timed card removal, title spans, sprite clocks and character/card geometry. |
| `rust/src/coding_view.rs` | Native editor and drag/drop, SVG block painting, puzzle/maze and dancer rendering, chooser/results, beat/track queues. |
| `rust/src/main.rs` | Routes, scene input eligibility, loading, resets, audio drains and capture controls. A missing input-eligibility branch previously made visible editors inert. |
| `rust/src/assets.rs` | Image decoding/filtering; Boogie dancer strips load lazily, chooser SVGs use higher resolution/mipmaps. |
| `rust/src/sound.rs` | Six original Boogie music slots and coding sound actions; full mixer/bus timing is still incomplete. |
| `rust/src/chrome.rs`, `navigation.rs` | Shared host HUD/Home/Menu/level counts and playable recommendation pool. Verify host configuration separately from scene markup. |
| `rust/assets/{codelab,codeboogie}/` | Native SVG exports of original symbols and Blockly-painted blocks, plus retained geometry. |
| `static/scenes/{codelab,codeboogie}/img/` | Original image assets in both checkouts. The reference checkout additionally has the original JS/HTML/SCSS. |
| `rust/data/coding-source-provenance.json` | Original source hashes and export/harness scope. |
| `rust/data/coding-function-inventory.json` | Function inventory; a listed/reviewed function is not necessarily fully implemented. Consult its status and the coverage ledger. |
| `rust/data/{codelab,codeboogie}-levels.json` | Source-exported levels and sequences. |

The coding section near the end of `rust/SOURCE-COVERAGE.md` maps source function groups to native work and explicit gaps. Own-scene source review does **not** mean the full third-party Blockly/Closure engines were reviewed or ported. The native editor is currently a replacement with known differences.

Durable test fixtures in `rust/data/`:

- `codelab-movement-reference.json`: 16,128 actual Chromium Player.move/jump cases, controlled starting cells/directions/lengths, including collision and animation durations. No movement/animation engine stub. This does not cover every rendered animation phase or sound.
- `codeboogie-evaluation-reference.json`: 54 original status/queue cases spanning nine dance classes. Original constructors/methods run in a VM with namespaces/inheritance/localized messages supplied; this does not exercise the Blockly engine or browser dancer renderer.
- `codelab-viewport-reference.json`: five settled original DOM viewports.
- `codeboogie-chooser-reference.json`: five original DOM viewports, including the requested 1846x966 and the actual native 1846x1014 window size.
- `codeboogie-chooser-raster-errors.json`: four matched idle captures below host chrome, with explicit region/limitations. Low idle error is not whole-mode certification.
- `codelab-initial-raster-error.json`: initial jigsaw comparison only.

Reference exporter/harness scripts may exist under `rust/tools/` in the development checkout, including `export-coding.cjs`, `export-coding-blocks.cjs`, `codelab-movement-reference.cjs`, `coding-layout-reference.cjs` and `coding-browser-reference.cjs`. Some are local/untracked and unavailable in a fresh clone. Inspect their scope and actual availability before invoking them; preserve them when present. They must not enter the Rust distribution. `/tmp` scripts/logs/screenshots from earlier sessions are disposable, not durable evidence.

## Corrected source details to preserve

- Boogie chooser uses `#3f49a3`, a Lobster title with source letter spacing/shadow, Roboto ExtraBold captions and regular descriptions. Its options deliberately have unequal artwork dimensions. The teacher SVG has a 397:371.2 viewBox; preserve containment inside its square background box.
- The original freestyle caption uses `choose-mode-info`, unlike the teacher's `choose-mode__info`. That source typo creates different normal-flow/absolute positioning. Do not "clean it up" or make the options symmetric.
- Chooser desktop breakpoints are 680/1025px; the shared dialog animation has its own 661px breakpoint. Background entrance is 500ms, dialog entrance 700ms with the first 20% held, artwork hover 200ms, Go transition 400ms after 200ms delay. Formulas are implemented; full phase/reversal/device validation remains unfinished.
- Lab movement duration is measured from real animation objects: walk 500ms, jump 1000ms, plus 200ms when turning. Declared wrapper duration alone was misleading. Jump checks its landing cell at length+1.
- Lab world origin is 0.3 tiles below its fixed 12.6-row viewport origin. Width-driven tiles can make that viewport taller than the actual window. Do not replace this with height-centered intuition.
- Block logical connection size differs from painted bounds: keep negative top notches and bottom shadows. Editable/movable/deletable are separate flags. Preserve the original grip marks on movable blocks.
- Boogie permits editing during playback; Lab locks it. Run uses the original orange button. Do not restore the invented Space-to-run shortcut or Boogie R-to-chooser reset; its source API restart handler is a no-op.
- Lab schedules an uncancelled one-shot portrait hide after three seconds per level. Older callbacks can survive a level change. Do not repeatedly close the tray every frame after the deadline.
- Boogie music variant changes are scheduled on the next bar and reset the music clock. Sample identity alone does not verify phase, gain, guards, fades or bus mixing.

Exporter pitfalls: export full/mini blocks through the actual original Blockly renderer, inline computed SVG presentation, embed relative image bytes, and escape XML nonbreaking spaces as `&#160;`. In the symbol exporter, skip source `block-*` symbols so they cannot overwrite the exported full block templates. Run the block export last if regenerating both. Preserve geometry and the source's nonmovable/nondeletable when_run.

## Remaining work when asked to continue fidelity

Prioritize the latest user correction. Otherwise choose a bounded unfinished source group and read its complete files and active dependencies before translating it. Useful next Boogie groups are exact result/tutorial/sharing flows, Blockly/editor behaviour, complete audio presentation, and additional active-render/lifecycle comparisons. Update the coverage ledger with the native equivalent and remaining branches rather than declaring the entire mode complete.

Known gaps include:

- Boogie stage logic and selected CSS phases now have source checks. Full active-gameplay raster/phase coverage, light/transform reversal combinations, image-load/colour retry races and audio-clock/route lifecycle still need work.
- Blockly-style nested layout, connection precedence/insertion, field/flyout/trash geometry, workspace panning, touch/keyboard/accessibility and responsive editor motion.
- Broader result button/focus/device/controller checks, final host game-over, tutorial queues, sharing/import/export and exact textarea layout/scrolling (Boogie loop formatting is source-matched for supported enabled graphs; Code Lab still expands actions).
- Lab pooled world/presents, world travel, precise rotation/sprite/pickup/loss ordering, highlights/rewind/cancellation, portrait transition/run delay. Its 100,000-action native limit is a documented intentional responsiveness difference.
- Precise audio scheduling, one-shot guards, cb_fail combinations, bus ducking and listening/device validation. Previous native UI tests were muted.
- Small chooser glyph spacing/raster differences, chooser exit and broader hover/entrance/reversal comparisons. Matching DOM boxes does not verify glyph pixels or interaction animation.

Christmas tracking is a separate unfinished UI task. `tracker.rs`/`tracker_control.rs` have route, date-gating and controller evidence; a standalone Santa preview is not an integrated tracker/map. The user does not currently have a confirmed Maps API key. Do not claim the map works or ask them to paste a key. Preserve the user-requested Dec 24 10:00 UTC–Dec 25 11:00 UTC visibility gate (end exclusive).

New stage harnesses are `rust/tools/codeboogie-stage-{reference,layout,phases}.cjs`
in the reference checkout. Their generated `rust/data/codeboogie-stage-*.json`
fixtures and `rust/examples/codeboogie_stage_capture.rs` are distributed. The
original Web Animations polyfill can hide native CSS animations from its
getAnimations wrapper: save the native Element/Document methods before the
polyfill loads when measuring CSS phases. Source start with `removeGuiNow=false`
exercises the real chooser close path; setting only hidden did not hide its
explicit display:block layer. Always inspect the captured reference image.

The stage oracle retains late-start/current-animation state, skips missed beats,
keeps the floor permutation across levels and preserves delayed remove-all
callbacks. Do not replace those behaviours with catchup loops or clear-on-restart
intuition. UI smoke needed a rendered-frame acknowledgement between synthetic
mouse down/move/up; the first unacknowledged attempt placed no blocks and was
discarded. The successful completion used periodic frame captures so occlusion
could not skip most beats during the test.

## Build and compare reliably

Run from the chosen checkout, checking local tool availability first:

```bash
export RUSTUP_HOME=/tmp/santa-rustup
export CARGO_HOME=/tmp/santa-cargo
export BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/clang/22/include
/tmp/santa-cargo/bin/cargo test --manifest-path rust/Cargo.toml --offline
/tmp/santa-cargo/bin/cargo build --release --manifest-path rust/Cargo.toml --offline
```

Omitting the Clang include override previously triggered a ThorVG rebuild and `stdbool.h` failure. Release builds may take about a minute; do independent work while they run. No Rust tests are needed solely for editing skill prose; validate skill structure/references instead.

Both coding views support the existing diagnostic `--advance`/`--frames` controls. For a settled chooser:

```bash
rust/target/release/santa-tracker --scene codeboogie --mute --width 1440 --height 800 --advance 2 --frames 3 --screenshot /tmp/boogie-native.png
```

Without advancing, a frozen initial chooser can correctly be offscreen during entrance. For gameplay, run live and use the real Run button. Own your test process and terminate only that process; leave the user's window alone. Confirm actual saved dimensions and match the reference to those dimensions. Compare the independent host chrome separately from the scene.

UI automation on this XWayland setup has sometimes coalesced input between rendered frames or ignored warps in background/occluded windows. An automation script finishing is not a successful drag/playthrough: verify the visible block graph, action/result state and intermediate frames. Focus the owned window, allow frames to render between actions and wait for a complete parseable PNG before reading/renaming it. Discard ineffective-input captures rather than changing game logic to satisfy them. Remove temporary per-frame diagnostics before publishing.

For original comparisons, inspect `serve.js` and existing scripts before starting a reference server; one prior invocation was `node serve.js --port 9022 --cache`, serving scene references on port 9102. Do not assume it is still running or that those ports are free. Execute original code only in development reference tools, never in the shipped app.

## Publishing and leaving a useful handoff

Fetch the distribution remote before an authorized push: the user edits its README directly. A previous pass had to preserve their new prose and Elf Jamband rating while merging the coding rows. Copy explicit native code/assets/fixtures and agent feedback, keeping root README images/prose/ratings and unknown user fields. The distribution's `rust/README.md` has native-only verification instructions; the reference copy can contain browser tooling, so do not blindly overwrite it either.

Regenerate `PARITY-REPORT.txt` from the correct checkout's JSON with the status_report example; stdout is just the saved path. Do not redirect stdout over the report. Shared build targets may reuse an example built in the other checkout, so check its embedded manifest root/input and output path before relying on it.

Inspect staged paths to exclude browser code and `rust/tools/`; preserve original asset bytes/licenses (some original SVGs contain CRLF/trailing whitespace). Commit and push only the clean distribution's own history to its public origin. A local reference commit is separate. Check clean distribution status, remote visibility and successful push. Update this reference only when durable continuation facts change; put detailed ongoing status in the existing feedback/coverage files.

## Result-panel continuation facts

Use `codeboogie_result_capture` (see native README) for the shared live renderer.
Its `flow` case is input-independent: actual intro/Run/Finish plus editing during
playback is asserted and rendered, but this does not replace pointer/keyboard
verification. The old failed synthetic input runs remain discarded.

Do not add an intuitive root fade: the original unitless time in `visibility 0
0.3s` invalidates the entire root transition declaration. Actual Chromium reports
zero duration (`transition: all`), so show/hide is immediate while the children
retain their delayed curves. The original a[hidden] View code link also remains
visible due to its inline-block CSS. Both quirks are covered by fixtures.

The separate missing-repeat symbol is not the full exported editor repeat block.
The font probe finds Noto Sans Mono on this Linux reference host; its original
installed font/OFL are retained for selected code-pane matches. No host sharing
or final shared game-over implementation has been added. Inspect current tests,
feedback and coverage; do not turn selected low image errors into whole-mode 1:1.
