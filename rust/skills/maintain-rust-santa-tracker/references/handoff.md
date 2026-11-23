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
differences remain; textarea interaction, sharing and complete host presentation/lifecycle remain
unfinished. The README rating stays Very similar.

Do not confuse these three things: the user's README rating, their editable approval fields, and the scope of measured technical evidence. Changing one does not automatically change the others.

## Find the implementation and evidence

| Location | Purpose |
| --- | --- |
| `rust/src/coding.rs` | Typed block graph/interpreter, maze execution/results, dance status/queues, source viewport and chooser layout formulas, fixture tests. |
| `rust/src/coding_host.rs` | Teacher/Lab completion and host restart, checked against 29 real browser host states. |
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
- Broader result button/focus/device/controller checks, complete host game-over presentation/lifecycle, tutorial queues, sharing/import/export and exact textarea layout/scrolling (Boogie loop formatting is source-matched for supported enabled graphs; Code Lab still expands actions).
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
installed font/OFL are retained for selected code-pane matches. Host sharing is still missing. The next section connects shared game-over;
complete presentation/lifecycle matching remains unfinished. Inspect current tests,
feedback and coverage; do not turn selected low image errors into whole-mode 1:1.

## Coding completion continuation facts

The placeholder completion page is gone. `coding_host.rs` and main routing now
open the existing shared overlay over the retained last scene. Code Lab resets
to puzzle one; Code Boogie host restart is intentionally the original no-op,
clearing host game-over without resetting editor/player/clock. Do not replace it
with a chooser reset. Its following Continue reports stop 10 after stop 9;
Lab reports stop 15 after fourteen levels. Source host ignores stop payload score.
The header must offer restart at game-over even when can_pause is false.

`coding-host-reference.json` has 29 real browser scene/API/Unistore/component
states. Original entrypoint closures run unchanged; external Firebase/router/
audio/tutorial boot is scaffolded. `coding_completion_capture` invokes native
completion callbacks, asserts restart semantics and renders the real shared
composition. It does not solve levels or certify pointer input. `finish_live`
is required for coding: caching the fully disabled filtered image freezes live
Boogie actors/lights. The far-right native crop changes under game-over; Lab's
idle crop stays static. Tests currently total 75, but revalidate live state.

Reference boot needs original CSS/attribute polyfills before components, and Lit
updates/CSS transitions must settle before screenshots. Early hidden=false and
getAnimations finish alone produced near-invisible overlays; those captures were
discarded. Auto-focus rings, heavier title raster, shared blur approximation,
keyboard/orientation/route/input-frame/audio details remain unfinished. Broader
freestyle/custom/share/boot lifecycle still requires source work; no full parity
claim or README rating change follows from these completion checks.

## Jetpack level-up flash correction

The user reported a slight flash on level-up. Native 1.1px gradient strips on
1px strides double-blended at fractional positions (e.g. 0.45px), causing a
roughly 20-channel colour jump. `JetpackView::draw_sky` now draws a continuous
solid/fade mesh matching the source 200%-height background. The original JS
BACKGROUND_HEIGHT=150 constant is unused; do not copy it instead of actual CSS.

`jetpack-background-reference.json` has 12 real original gradient cases and RGB
column bytes. `jetpack_background_capture` checks the actual GPU sky renderer
(28,800 channel comparisons, tolerance 2/255), taking account of bottom-up GL
readback. Browser exporter is development-only. See coverage/error fixtures for
before/after evidence. The five-second progress/strict >1 completion and other
Jetpack art/logic are unchanged; full mode parity is still unproven. Preserve
this continuous draw rather than reintroducing overlapping scanline strips.

## Snowball Storm continuation

The 27 October 2026 pass adds the local `snowball` route, `snowball.rs`,
`snowball_collision.rs`, `snowball_model.rs` and `snowball_view.rs`. Read the
Snowball section in SOURCE-COVERAGE.md and its function inventory before
continuing: the rating is Very similar, target 1:1, and several lifecycle/mixing/
randomness/input/audio areas are explicitly partial. Do not mistake the initial
render/pose oracles for complete mode parity.

The actual index starts `local`, not the unused multiplayer/lobby path. Preserve
per-render-frame movement with wall-derived ticks, greedy path cost using the
source's from-index cost, source queue tie behavior, separate map/erosion seeds,
and nearby-tile selection's origin-vs-candidate bug. Tree statics are registered
after quad bounds creation and absent from its dynamic rebuild, but the outer
collidables iteration still queries them against moving objects. They DO hit
projectiles. The earlier inactive-tree conclusion was incorrect; see the
collision follow-up below and its unchanged-source oracle. Source camera lookAt followed
by rotation.z=0 produces a flipped-Y camera, not the intuitive lookAt basis.
Weather's supplied attributes are unused and it has no active attribute, so the
splash has no invented weather. No pause, R shortcut or swapping click is added.

Live rendering separates begin_frame (Level/camera/erosion) and finish_frame
(systems), with draw/input in between. Map picks queue for the next Level pass;
elf targeting is direct. The original keeps simulating behind game-over. Native
batching is necessary to avoid Macroquad silently clamping large map meshes.
GLTFLoader r110 binds only JOINTS_0/WEIGHTS_0; extra sets in this asset are ignored.
The source texture cache keys only major/minor colours despite hair/ponytail
arguments. Native per-fragment side tones/alpha must not be replaced with white
top vertex interpolation (that visibly changes raised edges).

`snowball-reference.json` checks four seeded maps/erosion, grid/path/shot/powerup
formulas and forty real source quad queries. `snowball-skin-reference.json`
checks 30 original Model/AnimationMixer poses, all 2746 vertex positions; it does
not certify interrupted action mixing/normals. `snowball_capture` uses the real
native renderer. Its `reference` case imports measured original state/textures,
omits presents/projectiles and resets animation phase; never present its low
whole-image error as a full gameplay match. Development browser exporters stay
in the original checkout's rust/tools only. Preserve unknown/user feedback and
the public README's two-column table on publication.

## Snowball action-state follow-up — 22 November 2026

`src/snowball_animation.rs` now drives each player's persistent Model state;
throw/fall timing must not return to timestamp-based rendering. New fixture
`snowball-animation-reference.json` has 650 actual Model/THREE states, 42 actual
ready-model Elf.update states and fourteen scaled skeleton vertex/normal samples
per Model state. Consult its provenance and coverage scope; this is not full-mode
certification. CPU skinning now applies scale five to the glTF root inside the
hierarchy, not outside its translation. Keep the client marker on the unscaled
dolly. `snowball_capture ... action-N` renders a first-sequence animation state.

Preserve cached once/clamp/timeScale flags, same-name play no-op, retained throw
callbacks after interruption, mutable action-array iteration during finished
callbacks and the unfiltered Elf fall listener. Original property poses are saved
when active bindings go from zero to one; stopAll does not restore the pose.
Fade clocks/results are Float32 even though action and property arithmetic is
Float64. All original scene tracks are LINEAR TRS; unused engine warp/scheduled/
ping-pong paths are explicitly out of this port's checked scope.

The integration harness exposed that Elf.onAllocated retains position and facing.
Those incoming transforms are now recorded/imported for identical starting
conditions in the isolated update test. It does not fix/certify native object
pooling. Original model/actor reuse, LOD/readiness, collision registration and
full-match random/lifecycle parity remain work. Installed cargo is currently
available at /home/finn/.cargo/bin/cargo; the old /tmp toolchain was absent.
Continue to check availability rather than assuming either path exists.


### Snowball collision follow-up (22 November 2026)

`data/snowball-collision-reference.json` records eight actual original tree/quad
collision cases, eight actual SnowballSystem queue cases and four shared-limit
frames. Its provenance describes the actor/effect scaffolds. Tree colliders
(radius 14, grid y -17) participate as primary colliders even though not inserted
into the local dynamic quad. Erosion does not unregister them. Never infer an
inactive branch solely from quad insertion; follow both sides of notification.
Native contacts now include trees; multiple hits happen before removal.
Source newSnowballs.pop() reverses setup order. Splice + for-loop increment skips
the shifted next shot for both collision and expiry; freshly set-up skipped
shots are collidable before tickWhenThrown initializes. `Shot.setup` tracks that
separately. Collision uses the shared limit's previous LodSystem camera center;
resize/MainLevel.measure recreates that limit at the origin.
Full dynamic Set/handler ordering/deduplication, persistent quad nodes, shot/shot
contacts, pickups/arrivals and original pools remain partial. Do not claim these
narrow cases certify the whole match. No rating/user-feedback changes follow.


### Snowball ordered-contact follow-up (22 November 2026)

Production now uses `snowball_collision::Collision`: ordered registration,
persistent Quad (including source clearDynamic empty-node retention), exact
asymmetric measuredCollisions branches and directed notifications. Stable
Shot.id/hit survive Vec splices; player setup/arrived handler flags and drop
landing/colliding-player state keep subscription/registration separate from
health. Ordinary frames must not call reset_collisions; it is for fresh-map or
explicit diagnostic state import. Bots query the same retained frame quad.
Shot/shot contacts, every hit sound, thrower exclusion, incoming facing -PI/2,
repeated fall callbacks with first exit timestamp, first-notified Elf pickup
(including simultaneous death), next-pass spin deregistration and one local
pickup sound per frame now have actual concrete-source checks.

Evidence: `snowball-collision-lifecycle-reference.json` (five original persistent
collision sequences, 22 frames) and `snowball-contact-actions-reference.json`
(five actual actor/system sequences, 20 frames), with provenance explaining
scaffolds. The latter starts actors arrived with ready models, imported measured
poses/scales, map held at state 1, no bot/parachute/LOD/input/effect integration,
and controlled fall selection. Do not certify full-match random/pooling/arrival
behavior from it. Earlier dynamic-contact gaps are superseded within this scope;
full spawn/landing/freezing/restart phases, generic collision subscription/debug
and mid-pass XY/registry mutation remain unverified. Normal active callbacks do
not mutate XY/registry, so ordered measured-pair dispatch is valid for this path.

Reference server imports rewrite to ?module. Harness imports of Elf/Snowball
must use that same canonical URL: mixing direct .js and .js?module creates two
class identities and silently breaks instanceof-based pickup/death handlers.
An initial contact run was discarded after debugger confirmation; final fixtures
use canonical imports. Do not infer inactive handlers from a duplicate-module
harness. Existing formula/shape tests do not depend on those instance checks.


### Snowball carry/landing follow-up (22 November 2026)

`src/snowball_parachute.rs` owns FIFO undropped/dropping/removed state and stored
canopy transforms. Arrival.dropped starts Infinity, then actual ParachuteSystem
carry sets it. DropSystem queues before parachute update; PlayerSystem queues
after it. Never replace elf carry with allocationTick+1: frames can skip/hold
clock ticks. `Player.landed` / `Drop.landed` distinguish map attachment from
arrival. The present is detached for one render frame after arrival and is
registered/reparented by next DropSystem. Elves reparent in the same PlayerSystem
stage. Landing precedes queued destinations/targets; bot target actor identity
must survive reparenting before the shot reads its position. `Drop.setup` also
keeps pre-setup model state separate from its parachute-ready z32 pose.

Render stored system state, not descent recalculated with the new render clock.
`SnowballView::carried_matrix` is T(canopy)*Ry(sway)*T(0,0,-84); flattening its
translation loses the rotated offset. Draw canopies independently of carried
actor visibility. Hide unparented actors, detached presents and the local marker
before arrival. Normal parent composition is checked; freezing/teleport parent
reparenting, LOD/model readiness and full actor/resource pooling remain partial.

Evidence: four actual original parachute cases/21 updates (including source
teardown mutation/deferred detach), three actual player/drop/parachute collision
stage cases/22 frames, 210 actual THREE matrices and four isolated present/canopy
pixel comparisons. Read the provenance/scaffolds: explicit spawn tiles, passable
map, controlled bot throw selection and imported drop scales do not certify a
full seeded match. The `snowball_capture` example accepts `arrival-0`,
`arrival-30`, `arrival-130`, `arrival-299` as third argument after output/seconds
(use seconds 0); imports measured canopy state, clears map/players and renders
only this isolated asset comparison. Those images have small edge differences;
no whole-scene 1:1 claim follows.

Source texture readiness must be checked before an isolated loaded-asset capture.
When external requestAnimationFrame is held by a harness, Playwright's default
RAF polling can stall readiness waits; use an explicit interval for that wait.
A first incomplete canopy capture was discarded before final comparisons.

The distribution rust/README.md has a deliberately native-only verification section that differs from the reference checkout. Sync specific diagnostic paragraphs there; replacing the whole file would introduce commands for reference tools that are not distributed.


### Snowball removal/frozen-parent follow-up (22 November 2026)

`src/snowball_removal.rs` owns Presence, Removal queues, pooled Ice transforms
and Parent (Detached/Map/Canopy/Ice). Do not infer a transform or disappearance
from `.frozen`: requests mark exiting now, next removal update attaches LIFO and
sets local (0,0,-16). Frozen presents stay visible. A parachute can later reparent
a frozen elf back to the map while its now-empty iceberg continues drifting.
`landed` records that the arrival queue was processed; actual Parent determines
render membership. Iceberg/free pooled state and strict bounds use source
formulas; indexToPosition resets z, so reused setup starts at +32, not old-z+32.
Drift is one normalized 1/3 step per update, never time-accelerated.

Presence and Visibility/material opacity differ. Teleport holds 1500ms then
moves local z and visibility for 700ms. At completion exiting is false before
Elf.update, so it retains the preceding material opacity; local client mesh
stays parented, NPC cleanup detaches its normal map parent. `active` separately
tracks system-array membership and prevents repeated knockout counting. Client
records first observed dead time before PlayerSystem; gone or strict >2500ms
then triggers game over. Full Client arrival-marker/camera/input/HUD paths still
need matching; don't mistake just its verified death decision for full parity.

Evidence: four actual original removal cases/26 updates; three actual concrete
system cases/19 frames with ready models; ten THREE body/model matrices; five
isolated frozen-present/iceberg comparisons. Read provenance for explicit tiles,
map cells/clock/pool reset and selected fall randomness. No full-match/scene
restart/actor-model pool/LOD readiness certification follows. Diagnostic third
argument `removal-6`, `removal-7`, `removal-43`, `removal-81`, `removal-126` after
output/seconds (seconds 0) imports only measured body/present state on blank map.

The above direct-window captures were initially misread as an application MSAA
mismatch. That was wrong: main already renders through a single-sample SceneFilter.
The later compositor follow-up below corrects that conclusion and its diagnostic.
Do not change the application window MSAA based on those old diagnostic edges.
Ice opacity now uses Float32 vertex data instead of byte-quantized tint alpha.


### Compositor/density follow-up (22 November 2026)

Before diagnosing pixels, trace main.rs -> SceneFilter.begin -> scene draw ->
finish/finish_live. `snowball_capture` formerly bypassed that path and rendered
directly into the 4x-MSAA default window. The application already used a
single-sample scene target. Historical direct-window metrics are labeled; use
`snowball-compositor-raster-errors.json` for corrected application-path evidence.
The diagnostic now includes the production compositor and high_dpi window flag.

The real shared issue was logical-sized framebuffer allocation on high-DPI
windows. SceneFilter now allocates logical size * native dpi_scale and keeps
camera/presentation in logical units, including disabled texture copies. Density
or size changes invalidate cached filtered surfaces. `surface::scissor` converts
24 existing clip calls by active render-pass texture dimensions (or physical
window dimensions), with unchanged logical expressions. At 1x it preserves the
rectangles; empty clips must remain empty at fractional density. Do not scale
layout or input to fix framebuffer units, or use the window density for a
separately sized render target.

Evidence: actual GPU density/clipping/cached-presentation diagnostic and 27
isolated original/native present/canopy/iceberg captures at 1, 1.25 and 2 pixel
ratios. The nine selected normal-density captures are unchanged by the allocation
fix. Source uses actual browser DPR, native diagnostic uses explicit target
ratio; actual OS DPI remains 1, so monitor events/all font and UI density are not
certified. `snowball_capture output.png 0 removal-81 2` exports opaque scene RGB
at 2880x1600 from a 1440x800 logical viewport. A final explicit ratio is for an
enabled scene diagnostic, not a disabled host screenshot. `scene_filter_density`
exercises the actual shared GPU path and cache, without changing user display
settings. Residual shader/filter edges and whole-scene/match scope remain open.


### Client arrival marker/camera follow-up (22 November 2026)

There are TWO PlayerMarkers. `client_arrival` stores ClientSystem's direct
playerLayer marker, independently of the ring under the elf's dolly. Attach
only if alive, not arrived, tile > -1 and no parent. Store its transform once;
subsequent tile/dolly changes don't move it. Camera snaps to tile.x/tile.y*-0.75
without the old native dolly-y offset. Landing removes it. Dead/no-player early
returns preserve the attached marker, including death during descent. Start
clears this state and begins with the source level camera at the origin.
MainLevel's living camera also requires !presence.exiting and !presence.gone.
The tracker retains the source browser `length`=0 quirk and per-update friction.

`data/snowball-client-reference.json`/provenance contains four cases / 28 frames
from actual Client/MainLevel/Tracker/THREE/grid. The native test invokes actual
Snowball.begin_frame camera gates. Player/UI/event sinks and clock are explicit
stubs; full target/destination/network/HUD/assignment/removal are not checked.
Native local spawn still chooses missing tiles earlier than original PlayerSystem;
do not call these explicit-tile tests a whole initialization check. Pooled shared
elf-marker visibility, model/LOD loading and transparent ordering remain partial.

Both ring paths use exported original 50-vertex ShapeBufferGeometry and exact
indices with float diffuse/opacity. Arrival depth testing is disabled. Reuse
existing clip and particle pipelines: the full app eagerly loads all scenes and
Macroquad 0.4.16 has a fixed 32-pipeline limit. A first two-material attempt
passed isolated capture but failed actual app startup, and was replaced. A
negative per-vertex data.z tag selects the unlit ring branch; ordinary clip
geometry uses zero and particle random is nonnegative. Preserve their existing
shader paths. `snowball_capture output.png 0 marker-attached` (or retained,
landed, dead) isolates the measured arrival marker, not an entire elf/match.


Current user constraint: they are playing and asked to stop interrupting their
game. No more app/browser windows, focus changes or input injection during that
session, even for an owned smoke process. Source reads, saved-image review,
compiles and headless/unit work can continue. Offline texture audit confirms
seven original PNGs are byte-identical in distribution; their reported unclear
textures are still unresolved. Inspect sampling/render scale and generated
textures before proposing any sharpening. Last actual live app smoke showed
descent and later game over, not a source-matched complete match. --advance on
an untouched Splash does not start Snowball; earlier time-only app screenshots
were still the splash. A first XTest attempt failed to capture; injected owned
X11 window events later produced the recorded descent/game-over images. Do not
repeat that input test while the user is playing.


### Offline intro texture-density pass (22 November 2026)

No window/input tests while the user is playing. `snowball_texture_audit` is a
normal-main CPU-only example: it invokes the actual Assets decoder, parses
source SVG bounds and records sample coverage for existing CSS/draw sizes.
The cloud was 118x71 enlarged to 426.546x256; scale 8 now decodes 944x568.
The elf sheet was 512x512 with a 256x256 selected quarter, so 160x160 at 2x
was enlarged. Scale 2 now yields 1024x1024 with a 512x512 selected crop.
Only raster density changed; logical bounds/crops/destinations/timing remain.
See `snowball-intro-density-audit.json` for before/after/source hashes and scope.
The first 90x90 elf claim was an unmeasured mistake, corrected before the change;
actual CPU decode measured 512x512. Do not copy that initial claim.

The offline bilinear preview demonstrates the cloud's former enlarged-bitmap
edge, not Chromium or native GPU rendering. All 97 tests pass; no GPU/window
check was run after this density change. Two RGBA decodes add about 5 MiB plus
mipmaps; cold-load timing is unprofiled. General unclear world textures remain
open. Existing reference elf texture PNGs can locate the binary-circle eye-AA
difference without opening a window; source PNG byte identity does not prove
sampling or generated-texture parity. CPU Image.export_png flips rows because
it targets GPU readbacks: use png::Encoder on decoded top-down bytes instead.


### Elf map/sampler offline follow-up (22 November 2026)

Keep the user's no-window/no-input restriction. World trilinear filtering is
source behavior. A confirmed mismatch was wrapping: glTF empty sampler defaults
to Repeat/Repeat in the actual loader, then Elf clones that map before replacing
its image. Texture.copy retains these flags and flipY=false. Macroquad upload
was Clamp. assets::configure_texture now sets Repeat on generated elf.png and
its imported reference.png only; the other Snowball maps stay clamped. Actual
UV accessor data includes two U values below 0. See world-texture provenance
for source/backend hashes and scope. No GL state or full runtime-render check
was performed. Don't generalize this to all scene/CSS textures.

`snowball_texture::elf_rgba` replays source-order rectangle/path paints; the View
still caches by major/minor, deliberately ignoring hair/pony like source. Eye
circles now use native AA coverage instead of integer pixel-center tests.
Canvas's repeated retained contours have the same nonzero-winding region;
readiness/global cache population/seed order are still not certified. Against
28 existing original browser PNGs, all non-eye pixels match, error decreases
88.47–88.75%, and remaining 284–290 differing pixels have max channel error
20–36. This is NOT exact Canvas raster parity. Use the ordinary-main, CPU-only
`snowball_elf_texture_audit` and its errors JSON, not a live app, while the user
plays. Hair/pony are inferred only from constant non-eye reference regions;
no original eye image is injected. The immutable old loop lives only in this
diagnostic to reproduce the comparison, never as the application generator.
The 28-image regression checks non-eye identity and bounded improvement, not
a false assertion of full equality. All 98 tests pass and the release builds;
GPU sampling/lighting/composition/world clarity remain open.


### Command-stage/destination-marker pass (22 November 2026)

No windows/input during the user's game. Source oracles can run without a
browser: `node --experimental-vm-modules rust/tools/snowball-command-reference.cjs`
executes unchanged source modules with actual bundled THREE. This development
CJS stays out of distribution; JSON fixtures and native tests are distributed.
Canonical VM module identity matters for instanceof/static pools. See provenance
for inert Canvas/image/DOM, low LOD, empty pools and all physics/UI/network stubs.
It is not a full scene/engine/browser run.

Native now stores map pick, Client destination/target and Player command queues
separately. Input enemy targets copy position, whereas bot targets retain a player
index resolved after arrivals are positioned. MainLevel feeds Client; living
Client forwards, dead Client retains. Player drops dead/unarrived destinations,
then handles all targets. Failed local paths enqueue Client for NEXT frame;
failed NPC paths do nothing. Last target per player wins without changing first
insertion order. Shots enter at Player stage, so their Snowball setup/birth is
next update, still LIFO. Charges are chosen/decremented at dispatch. A target
assigned to a dead player retains its animation flag until a living update.
Living MainLevel clears an old dead-camera target without resetting velocity.

MapSystem destination-marker state is stored before commands/movement, not read
live from path while rendering. Destination shape is actual 12-vertex original
geometry; rotate z=PI/4, world z=cellSize*.5/y=destination-20. Reuse the existing no-depth
marker shader to stay under Macroquad's 32-pipeline limit. Camera matrix helper
is extracted unchanged; three supplied viewport/cell-size clip cases use the
production destination/viewport helpers and match THREE. GPU rendering
remains untested. The reference-pointer diagnostic now dispatches one system
frame after release; do not run it while the user plays or call it verified.

14 original cases / 64 frames / 192 native snapshots compare render, input and
post-Player stages. Source afterClient data is retained but not directly hooked
in native tests. Bot commands are injected before native finish in these cases
(they commute with early stubbed stages and avoid conflicting same-player Client/
bot writes); actual AI, collisions, loaded models, RNG/pools, async I/O, network,
raw mouse/octree/touch and entire scene lifecycle remain outside this evidence.
Tests cover powerup swap/depletion and reached-marker lag as well as commands.
Do not claim complete mode/input/animation parity from these comparisons.

This command/marker pass passes all 100 native tests (89 library + 11 binary);
release application/capture builds pass. No window or raw input was used.


### Elf root/ring visibility pass (22 November 2026)

No window/input tests while the user plays. New `snowball_lod::ElfLod` records
HIGH/LOW, dirty, root_visible and model_requested (not model ready). Defaults
LOW/dirty/visible/unrequested; dead assignments are ignored. Dirty is consumed
only in active Elf.update: high+alive requests/shows, otherwise hides. A dirty
HIGH consumed while dead does not retry just because health revives. Native
renderer now skips a hidden root, while animation clocks keep updating.

LOD follows collision using CURRENT camera and inclusive 15x45 local collider
containment. Canopy Point setters register AFTER the elf and forward LOD even
if another parent has taken the carried object; keep the override and both
intermediate dirty transitions. Do not hide canopy art or gifts for these
assignments. Generic duplicate registry/redrop/pool identity is still unverified.

`client_ring` is the shared marker under a dolly, separate from client_arrival.
Setup owns it; arrived sets mesh visibility. Dead+exiting UPDATE detaches only
that owner's marker. Freeze happens after Elf.update, so the ring survives that
next render. A time jump skipping exiting updates can leave it attached after
gone; preserve the source quirk. Teardown alone does not detach. Renderer reads
owner/mesh/root state, not health. start maps an old dolly outside the cleared
actor set to no current owner and retains visible; raw old-parent/pool lifecycle
is not implemented by this abstraction.

CPU VM `snowball-lod-reference.cjs` runs real source core systems/THREE without a
browser; original GLTFLoader FileLoader IO is held pending. It measures requested
flags/root/ring, not actual model completion or loaded-material opacity/actions.
Eight cases / 46 frames / 92 native snapshots plus 13 ring-only operations cover
limits/resize/death/dirty/canopy/landing/freeze/skipped exits/reassignment/teardown.
Source afterLod snapshots are stored but not directly hooked in native tests.
sceneVisible means hierarchy eligibility, never pixel/frustum verification.
Native uses actual finish_frame with collision and LocalLevel/AI isolated out.

Legacy frozen raster capture imports poses/textures, not LOD/readiness flags;
it explicitly keeps root-visible isolation and restores its known living,
arrived local ring. It was compiled, not run. Full loading/pools/materials/input
and GPU appearance remain open. All 102 tests pass; do not infer mode 1:1.


### Gift/canopy alpha pass (22 November 2026)

No windows/input while the user plays. Complete Drop/Parachute/Obstacles source
review and actual CPU bundled-THREE defaults show gift/canopy alphaTest=0;
tree atlas alone explicitly cuts alpha below .1. The shared native clip shader
now distinguishes these through data.z=-2 and float opacity in data.w. Gift
color stays white to avoid u8 opacity quantization. Canopy uses 1. Marker -1
branch and ordinary tree cutoff remain. No extra pipeline. All shared present
geometry scalars/indices match Drop arrays. See basic-alpha provenance.
This fixes known fringes/late fade; overall texture fuzziness is still unresolved
and GPU rendering/transparent sorting/loading/pools remain unverified.

Release application build passes. Shader runtime/pixel comparison was not run;
prior 102 state tests were not rerun for this render-only change.


### Snowball encoding pass (22 November 2026)

Snowball previously inherited Ski gammaOutput=true via ski-standard.frag.
Original Snowball RenderSystem uses false renderer default; glTF texture remains
sRGB through Elf map.clone. Production snowball_material::elf_fragment now
changes ONLY output helper to LinearToLinear and adds existing USE_COLOR.
Elf Ski shader stays gamma=2. Source CPU oracle snowball-encoding-reference.cjs
executes actual WebGLPrograms.getParameters/encoding helpers via internal-export
instrumentation, with original default assignments and explicit source-reviewed
sRGB map. No WebGLRenderer/browser/GPU constructed. JSON records exact helpers
and hashes; native regression verifies both production Snowball and untouched
Ski helpers. Main ambient/material factors already match. Configuration test
is not GPU or full glTF/loading/lighting validation. Overall fuzziness remains
unresolved pending visual verification; don't claim full mode 1:1.

All 103 tests pass (92 library + 11 binary). Release application and capture
diagnostic build; neither was executed. No window, GPU context or input used.


### Raw particle raster pass (22 November 2026)

No windows/input. Both original effects are RawShaderMaterial Points, not
PointsMaterial, so their explicit gl_PointSize is physical pixels without DPR
uniform scaling. Native quads now divide by surface::framebuffer_size (extracted
from existing scissor lookup). Trail layer draws before impact layer, confirmed
by CPU actual effect constructors/THREE transparent sort. Vertex elapsed/random
math now uses f32 source attribute inputs, trail PI=3.14159 and fragment birth
cutoffs. See particle-raster provenance and reference-only VM script.
This does NOT implement fixed 500/2000 slot rings, persistent random slots, source
wrap bug, pooled registration/restart lifetime or precise effect-stage updates.
GPU point-size hardware clamp/coverage/rounding, texture orientation and complete
scene sorting remain unverified. All 103 tests pass; no windows were launched.

Release application and snowball_capture diagnostic build. Neither was run;
GPU appearance remains unverified while the user plays.


### Fixed effect buffers and stored clock (22 November 2026)

Production now uses snowball_effects::Effects: fixed 500/2000 buffers, constructor
random-per-slot floats, ordered duplicate trail IDs, absent remove pops last,
LIFO stable Splat component keys, float32 writes and source old-cursor wrap bug.
Effects flush after Parachute before Map/Client/Bot/Player; render uses stored
Effects.time from the previous update, not new frame time. Snapshot traverses
slot indices and reuses Vec capacity. Trail colors are carried to the shader.

Important: effect teardown itself is a no-op, BUT index.js restart destroys the
Game and constructs a new one. Native start recreates buffers/randoms; don't
preserve old ring data on website restart. Splash placeholders consume no
gameplay randomness. Production stages test setup/removal/restart and no impact
reroll. New source VM exports 14 steps/all 2500 slots (35000 records/280000
fields) plus cursors/registrations/time. Imports original random attributes for
test initialization; full UUID/pool/RNG order is NOT certified. No GPU/browser/
input: TextureLoader image I/O returns an empty handle in source VM.

Source keys model stable component identities through queued mutations; complete
pooled Shot/Splat aliasing and actual image readiness remain incomplete. Match
clock/within-frame getter/host-pause timing, GPU point coverage/clamp/texture
orientation and whole-mode 1:1 remain unverified. All 105 tests pass. Earlier
notes saying fixed buffers were unimplemented describe the prior state.

Release application and snowball_capture diagnostic build successfully. Neither
was launched; user game/window/input remained untouched.


### Projectile material-state pass (22 November 2026)

Shot.shader_time is stored per material/update; projectile batch receives it in
normal.y and no longer uses new-frame global Time. Normal/big shot_visuals pools
retain random/time LIFO, also across start's new Game; teardown frees pending
before active. Full actor UUID/collider/event/Splat identity is NOT pooled by
this representation. Fresh random consumes once, reuse only consumes skew.

Important source quirk: Vector3.copy(Vector2 trajectory.origin) makes z undefined
until advance. Native throw uses NaN; renderer eligibility is setup&&hit.none,
not birth>=0. Newly parented, splice-skipped shots may have birth=-1/old time and
nonfinite coordinates. Native passes these through; GPU clipping is unverified.

Actual source SnowballSystem/Snowball/BigSnowball/Allocatable/THREE fixture covers
13 frames/26 snapshots/13 throws (5 fresh, 8 reuse). Native actual finish_frame
with collision/AI isolated compares state and free material order. Import random
only on verified fresh allocations; never on reuse. Source case restart replaces
SnowballSystem with supplied clock, not full Game. All 106 tests pass; no browser/
GPU/input. Complete object pool aliases, image readiness, full RNG/UUID/clock and
GPU output remain unverified. See projectile reference/provenance and coverage.

Release application and snowball_capture diagnostic build. No window or GPU
check was run; user game/focus/input stayed untouched.


### Map performance timestamps and float fade pass (22 November 2026)

GameMap timestamps and HexMap/Obstacles uniforms use performance.now, distinct
from reset-on-new-Game ClockSystem time. Native Map::new_at/set_state store
f32 timestamps; runtime passes wall, not game time, for creation/erosion. Stored
map_shader_time/obstacle_shader_time reset0 and update at Map stage after render.
Deformation casts both inputs to f32 before subtraction. Preserve negative elapsed
when a new state precedes the next uniform update; don't clamp it away.

Tile fade now uses float UV.x instead of u8 alpha; tree fade float normal.w/tag
-3 applies before original .1 cutoff. Basic material tag -2/marker -1 unchanged.
No pipeline/filter change. Source active helpers are shake/sink/fade; exported
erode helper is unused here. GPU fade/deformation/tone/pixels remain unverified.

Actual original MapSystem/GameMap/THREE clock fixture compares 4096 state/time
pairs over ten frames (203 deltas), erosion RNG and stored uniforms through
actual native finish_frame, including fractional/long perf times and new Map
reconstruction. Controlled constant-per-stage perf and image/input/collision
adapters, not real document/host timing. Native wall's exact absolute origin,
loading/background/pause and within-frame getter timing remain open. Full
reference capture imports known exporter map clocks at6000; compiled only.
All 107 tests pass; no window/browser/GPU/input. Whole mode/site 1:1 unfinished.

Release application and snowball_capture diagnostic build. Neither was launched;
GPU appearance remains unverified and user game/focus/input stayed untouched.


### Material transparency and winding pass (22 November 2026)

Snowball elf now supplies source transparent pipeline instead of Ski's opaque
standard config. Ski lit wrapper preserves former pipeline(phong); new
lit_configured accepts Snowball params. snowball_material::pipeline covers 12
kinds: FrontSide elf/tile/tree/gift/canopy/rings; DoubleSide destination/projectile/
iceberg/points. Ice additive alpha now SrcAlpha/One like RGB. Other alpha normal.

Generic quad/tree triangles now 0,3,1,3,2,1 for native TL/TR/BR/BL storage, matching
actual THREE plane topology before culling. Front-only clip/lit/tile pipelines
cull Back/CCW. Arrival ring uses gl_FrontFacing discard in existing no-depth
particle material (tag -1), destination tag -4 remains double-sided. No extra
pipeline. Effective depth writes off when depth test disabled.

Source CPU fixture: actual constructors/setup, GLTFParser.loadMaterial with image
I/O adapter + manually applied source-reviewed Elf clone/transparent override.
No real Elf callback/model readiness. Extracted original backend setMaterial/
setBlending/setFlipSided bodies run against recording adapters, caches reset
unknown per kind. Not real GL/WebGLState constructor or call counts. 12 source
configurations and actual plane triangles compare to production policy/helpers.
All 109 tests pass; release app/capture build, not run. Positive determinants
only; arbitrary reflections, GPU winding/coverage/pixels, full sorting/readiness
and whole-mode/site 1:1 remain unverified. See material-state reference/provenance.


### Iceberg priority/order pass (23 November 2026)

Shells moved out of effects() to after arrival/destination markers, before HUD: custom
source graphic renderOrder=1 outranks all ordinary world objects (order/group0).
snowball_render_order comparator mirrors source group/render/depth/ID branch
order; production uses it only for shell pass, NOT whole global sorting. Shell
depth uses default local camera/gimbal f64 origin, preserving source sort namespace.
At fixed cellSize=64/radius=32, body creation slots preserve relative graphic IDs
through reuse; radius-driven graphic replacement remains unsupported by this
ID equivalence. Other normal objects still use fixed draw groups: unfinished.

Actual source constructors/pool reuse + THREE matrices/frustum/WebGLRenderList
fixture covers six controlled scenes, exact slot order and depth within 2e-15.
Visible/default-layer/group0 traversal is fixture code, not renderer execution.
Generic source-key tests prove priority/ID and pairwise NaN behavior; arbitrary
nonfinite sort sequences remain unverified. No GPU/pixels/window/input. All 111
tests pass. Full global ties/model readiness/geometry/routing still unfinished.

Release application and snowball_capture diagnostic build successfully. Neither
was launched; user game/focus/input stayed untouched.


### Deferred spawn setup pass (23 November 2026)

add_player/add_drop no longer pick tiles on creation. Source arrivals hold -1;
Drop stage selects FIFO first, Player stage later selects FIFO before setup.
Cumulative all_time increments only at player setup/queue, matching connected
record timing. Source fresh initial 26p/25d cohort now gets the same per-actor
tiles/map RNG under a supplied map seed; don't revive eager tile selection.

UNDEFINED_TILE=i32::MIN tags failed/undefined indices separately from request
-1. Its position is NaN; NaN position lookup remains invalid rather than tile0.
Existing undefined components skip fallback. Not full JS coercion/serialization.
Native sparse/missing-ring defensive skips still differ from potential source
exceptions, while empty-ring undefined return is checked.

Original GameMap/Player/Drop/Parachute/Client/LOD/State CPU oracle, fresh pools,
held model IO/adapted Canvas/image and inert collision/input/network/events:
five cases/26 frames plus creation =57 snapshots, 70 actors. Actual native
finish_frame checks tile/count/arrival/parent/queues/transforms/RNG/camera. No
LocalLevel/AI/whole pool identity/global RNG/graphics certification. All 112 tests
pass. CPU-only example snowball_state_smoke ran 180s/10800 steps and reached
allTime100 without windows; results stored in spawn-smoke.json, not parity proof.
Pooled position/facing/model reuse, sparse exceptions, AI/global RNG and GPU
still unfinished. Earlier handoff notes about eager tile choice are superseded.

Release application/capture builds pass; CPU-only state-smoke also builds/runs.
No app window, GPU context, focus change, audio playback or input was used.

### CSS HUD image alpha and priority pass (23 November 2026)

Snowball's six CSS images now decode premultiplied and draw through the existing
premultiplied shader: item-frame, powerup-1, sleigh, parachute-button, loading-cloud,
elves-parachuting. Do not send these through straight/default sprite blending.
3 world maps and generated elf/gifts stay straight RGBA. No asset bytes/dimensions,
source crop/alpha/layout/density/minification changes. Original PNG transparent
RGB is black; straight interpolation darkened edges before source-over.

ui_render::premultiplied_material returns Rc<Material>, weakly cached like Shapes.
Eight prior callers plus Snowball retain shared handles; last owner releases GPU
pipeline. Shader strings/blend factors unchanged; no custom uniforms/textures.
No added pipeline slot; cache/runtime GPU and cross-scene behavior unverified.
Powerup inactive status paints first, active second, per source immediate z1/z2
swap priority, independent of ongoing transforms. Full HUD phase parity open.

All114 tests pass (101lib+13bin). CPU snowball_hud_alpha_audit compares actual
PNG decoder with analytic premultiplied bilinear/source-over: fractional-sample
max59--63.75/255 old vs <.5 new. NOT browser/GPU pixels or mipmap proof. Two SVG
associated-channel/density checks, 4PNG once-premultiply and3world byte controls.
No window/input while user plays. World clarity, eye-AA, GPU/CSS filtering and
whole-scene/site fidelity remain unfinished; do not label the user report solved.

Release application and snowball_capture diagnostic build successfully. Neither
was launched; no GPU/window/focus/input or audio test was used.

### Bot decision pass (23 November 2026)

snowball_bot::update is now the production decision helper. Source Bot.update
only returns for unarrived, not dead/gone; native gone guard removed. Vector3
root z now participates in origin/range checks (old native used XY only).
Preserve >.985 empty-path wander, nearby lookup before query, origin-not-candidate
map quirk, cached target bypassing filters, random-before-distance, strict >40
cooldown, and cooldown advance on failed >.5 coin. Retained collision quad query
still comes before Player stage. Dead Player blocks projectile but pending flag
remains; concrete finish_frame test covers inactive/dead/gone owner.

35 actual original CPU-VM decision cases compare303draws/16move/10target requests
and state/query count, including15 nearby searches/edge/corner/states1,2,3.
Explicit nearby lists and random tape after real source setup; plain actor
allocation boundary, no real quad in this new fixture or Player execution.
Canonical z0 move adapter matches source generated Vector3; Elf movement is XY.
All116 tests pass (103lib+13bin); CPU180s smoke reaches100 connected/70KO.
No window/input/GPU/audio; smoke is not source whole-match parity.

Next ownership gaps are material: bots still embed controller state in native
Player records and create players early. Source BotSystem queues actual player
allocation late and old Bot controllers alias recycled Elf instances. Do not
claim pool/global-RNG/whole-match parity from the decision oracle. Source
removeBot has no local caller; oldBots uses this.bits.splice typo/error, remains
unported/unverified. General nonfinite/invalid-ID behavior also uncertified.

Release application/capture builds pass. Neither was launched; user game,
focus, input and audio stayed untouched.

### Queued bot creation pass (23 November 2026)

Startup now has1 player/25 drops/25 queued bots. Late requests enqueue too.
setup_queued_bots drains only after Client, before Bot decisions and Player;
new player palette/lastThrow initialize then, allTime increments at Player setup.
Restart replaces pending count. Do NOT restore start-time26-player allocation.
The earlier bot-decision note about early creation is now superseded.

Actual original LocalLevel/MainLevel plus Bot/Player/Drop/Parachute/Map/Client/
LOD/State CPU oracle checks61 states over12 frames (creation + Level/Drop/Client/
Bot/Player), strict16/17,128/129,257/258,625/626 gates, pending/active bot state,
map RNG, actor tile/setup/arrival/root positions/parent, queues and camera.
Source map construction seed imported; constructor/global RNG/appearance NOT
matched. Fake DOM/image/input/collision/network/audio; models held pending;
no collision targets and explicit nonempty paths isolate wandering. Not full
SnowballGame/host/render or whole-match parity. Source methods unchanged.

Production finish_frame uses same driver with empty observer; tests observe
boundaries without substituting work. Old isolated fixtures disable pending
bots or explicitly supply two non-bot actors when their source oracle omits bots.
All118 tests pass (105lib+13bin). CPU smoke now asserts initial1 and26 connected
after first step. No window/GPU/input/audio while user plays. Separate Bot object
ownership and old controllers aliasing recycled Elf instances remain unfinished;
queued count only represents source's identical no-argument local requests.

Updated CPU-only180s/10800-step smoke passes: initial1 player,26 connected
after first step, final allTime100/knockedOut70. Retained native records are
not source active-array lengths or original whole-match parity.

Release application and snowball_capture diagnostic build successfully. Neither
was launched; no window/focus/input/audio or GPU test was used.

### Within-match Elf reuse and controllers (23 November 2026)

Player vector now contains PHYSICAL ELF SLOTS, not all historical spawns or source
active array order. player_order() returns active insertion order using per-life
order. Free NPC slots LIFO; client0 is not freed inside a match. Controller lives
in public bots Vec with .player slot/.state.last_target/.state.last_throw;
Player now has .speed instead of .bot and no controller fields. Old controllers
persist and alias reused slots alongside new controllers, per source behavior.

On reuse reset components in place; retain root position/z/parent, facing, LOD/
model-request and animator state. No animator cloning/recreation. Teardown clears
setup/pick handler and collision registration for freed NPCs. Setup/landing/
updates/free use generation order. Bot commands enter queues immediately during
Bot stage, with target position references resolved after Player landing.
Palette first-pair history (max24) preserves hair/pony cache choice despite slot
overwrites; renderer preloads it. Filtering/art/shaders/density unchanged.

Actual source allocator/Elf/Player/Bot and named arrival/LOD systems compare41
CPU snapshots,4 bodies/8 connected lives/7 controllers over two LIFO reuse cycles,
reset and retained fields/registrations/queues/throw boundary. Source decision
RNG controlled at Bot method entry using actual shared Random; constructor LCG,
UUID/palette/global RNG excluded. GLTF held pending; collision query supplied
elf2, native sole collider avoids contact pairs; source throws recorded rather
than executing projectile system. Snapshot logs copied, not mutable aliases.
All119 tests pass (106lib+13bin), all-target check passes; CPU180s smoke ends with
100 connections/99 controllers/50 slots/26 active/74KO. Not original match stats,
GPU appearance or latency proof. No window/input/audio.

NEXT: start still clears all elf slots/free list (cross-restart pool missing),
loaded model/animator/material/async and global marker ownership not certified,
old PlayerMap IDs aliasing recycled elves/network paths remain incomplete.
Numeric source discrepancy found: source movement stops against delta.length()
after normalization, native compares configured step; V::norm zero/NaN behavior
also needs exact source boundaries. Keep these explicit; no whole-mode 1:1.
Earlier notes about controller fields in Player and no within-match reuse are
superseded. GPU checks remain prohibited while user is playing.

Release application/capture builds pass; neither launched. User game/focus/input
and audio stayed untouched. Also audit distinct wall/game clocks next: source
Client death wait uses performance.now while native currently stores game time;
existing equal-clock fixtures cannot establish those divergent-clock branches.

### Vector endpoints and Client clock pass (23 November 2026)

V::norm now uses source length||1: preserve signed zero/subnormals, NaN length
fallback doesn't poison other components. Player movement compares distance to
computed normalized vector length, not configured speed; no epsilon/hypot.
Actual original THREE Vector2/Elf setup-update CPU oracle:225 normalizations,
621 steps (384endpoint+225vector+12speed), exact component bits/waypoint count,
NaN class only, atan2 face tolerance1e-12. Native real Player stage tested.

Client dead wait uses wall (source performance.now), keeping strict>2500ms,
gone bypass, zero sentinel and no reset on revive. Actual ClientSystem six cases/
25 frames test split/frozen/jumping game clocks and timestamp/stop/HUD values,
including missing actor. Native actor inactive to isolate Client branch.
All three new tests fail before correction and pass after. Older removal fixture
now sets wall=time=source input.at; expected data unchanged. All122 tests pass
(109lib+13bin). No window/GPU/input/audio. Previous numeric/Client mismatch notes
are superseded, not a claim of all invalid-number or clock-domain equivalence.

Remaining: cross-restart Elf pool/class/marker/model lifecycle, generic old-ID
routing, global RNG and loaded async/graphics; Health/Presence/Removal/animation
clock domains and actual app pause/background/absolute wall origin not certified.
User texture clarity is still open; source-based CPU work continues without
interrupting their game. Keep whole-site goal and conservative README ratings.

All three new tests also pass optimized release mode. Application/capture release
builds pass; neither launched. CPU180s smoke completes (100 connected,99
controllers,50 slots,26 active,74KO), not source match/graphics/latency proof.

### Jetpack image alpha pass (23 November 2026)

Jetpack CSS image decoder now premultiplies PNG once and retains resvg associated
SVG bytes. JetpackView uses shared premult material for all images, defaults for
text/sky/end; same-pipeline sprite batching retained. Original artwork/filter/
density/crops/layout unchanged. CPU nine-PNG audit includes OLD edge bleeding:
items/hazards/player/goal/touchpad worst analytic errors now <.5/255. Clouds
unchanged; fire incurs <=.448 byte rounding versus previously exact samples.
No browser/GPU verification or whole-scene clarity claim. New test covers nine
PNGs and three active SVGs. 123 tests pass; release app/capture compile only.

Full own-scene source read, branch scope/unfinished work in SOURCE-COVERAGE.
NEXT clear source defects: goal CSS contain-fit (412x452 image in 452x412 box),
outside scene scale and before scene paint; independent 3s CSS/3.5s wall timer
currently pauses incorrectly. Levelcloud/number opacity belongs to one group,
not two separate .3 source-over draws. Mobile touchpad presentation incomplete.
Do not launch tests/windows or inject input/audio while user is playing. Texture
clarity across other modes, display density/minification remain open.

### Jetpack goal geometry pass (23 November 2026)

Supersedes preceding goal geometry/stacking/resize defects: native uses original
contain/top-right 412x452 image inside capped 452x412 viewport box, draws goal
before scene sprites and captures target once on level10 (resize retains it).
CPU original Goal constructor/reset/transition plus actual Game resize callback:
30 cases +30 resizes, source hashes in jetpack-goal-reference.json. CSS fit is
analytic, NOT browser computed layout. Source target arithmetic uses *10/100;
*.1 first failed exact double comparison. All124 tests pass (110lib+14app).

NEXT goal clock/reset lifecycle still incomplete: native timer freezes on pause;
source setTimeout doesn't, can survive restart/dispose. Do not infer entrance
animation from declared transition alone: source removes display:none and assigns
transform in same task, actual browser transition might not run. Existing native
interpolation retained pending evidence. Levelcloud/number group opacity and
mobile touchpad remain. No windows/GPU/input/audio; don't interrupt user game.

Release application and Jetpack capture build pass; compiled only, not launched.
