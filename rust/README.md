# Native Santa Tracker rewrite — in progress

The target is a faithful rewrite of the **entire original site and all 55 scenes in Rust**, with the original art and audio. **The current build does not meet that target yet.** It is not a completed 1:1 recreation.

The application runs native Rust window, rendering, input, simulation, Ogg decoding and audio mixing code. It does not launch Node.js, Chromium, a webview or the original JavaScript game engine. SVG and Lottie files remain artwork; native libraries render them. Original source folders remain intact as references.

## Current coverage

- Village: original four background layers, exported sprite geometry/keyframes, original fonts/logo, eased Play/Family Guide controls and positions, eleven scrolling sections, native playback of their Lottie files, activity cards, a navigation grid, and snow/cloud simulation. Village animation includes the glitch elf, water-spout clipping, sprite keyframes, card hover/reverse motion and occasional card animations. Module/card geometry is compared against the original browser at 1440, 768 and 375 pixels.
- Shared controls: original toolbar SVG paths and sizes, eased press/release shadows, hover/press audio, keyboard focus, home/pause/restart controls and a score/time badge with level/score transitions. The December 24 10:00 UTC countdown uses the original split arithmetic, responsive hiding, shaped ends, font weights and rolling/fading digit transitions.
- Shared pause/game-over: original elf/trophy artwork, resume/restart/home buttons, two recommended activities, entrance/exit motion and the disabled scene's greyscale, brightness, blur and scaling. The selector uses the repository's original navigation defaults, adds Elf Ski and Present Drop explicitly, and reveals its cards together. Home Play and overlay recommendations share the original biased choice/recent-games behavior, filtered to the six native activities; Play no longer always launches Runner. Opaque offscreen scenes are copied without reapplying sprite-edge alpha, preventing dark outlines.
- Scene changes: original four coloured interlude layers and Santa/Mrs Claus Lottie illustration. Illustration rendering and scene image decoding run on background threads; GPU uploads are bounded per frame to keep the window responsive.
- Runner: Rust physics, seeded random generation, entity spawning, platforms, collisions, scoring, time boosts, magnet attraction, timers, pause and restart. The renderer now follows the original layer origin, discrete responsive scales/portrait ceiling, stretched background canvas, original run/slide frames, rollers/wires, machinery, belts/gears, hit/pickup/magnet effects and Lobster speedometer with level stripes. Five original simulation scenarios, live level timers/canvas movement, and DOM layout at five viewport sizes are compared with the source. See [source coverage](SOURCE-COVERAGE.md).
- Elf Jetpack: original elf/item/cloud artwork, animated flames and sparks, sprite frames, sway/rotation, sky transitions with clouds/level numbers and the finish goal. Flame scaling follows the original eased transition, player/hit rotation uses the CSS transform origin, and transparent sprite RGB is extended to prevent dark filtering fringes. Native movement, collisions, pickups, clock bonuses, hazards, levels and spawning follow the original classes. Three seeded scenarios are checked against the original simulation.
- Santa Selfie: original SVG scene and tool artwork; the original 645-point beard, Verlet constraints, shaving, dragging, regrowth, eight paint colors, four decorations and cleaning. Native hair compositing uses the original Bézier shape. Eye tracking/blinks, facial expressions, tool animations, breathing and the barber pole are animated. Eight tool/physics steps are checked against the original classes. The original beard encoding can be saved and restored.
- Elf Ski: original glTF skier and texture, original Standard/Phong lighting shaders, tree atlas with trilinear filtering, procedural terrain, present geometry/palette/spin/bob/collection, tapered two-layer ski trail, camera movement, crash tumble/snowball, scoring, sound events, restart and game-over controls. Arrow-key directions and held mouse steering follow the original input formulas. Source-executed tests compare world generation, motion, rotation, trail and pickup/removal. See [the source coverage ledger](SOURCE-COVERAGE.md) for reviewed functions and remaining gaps.
- Gift Slingshot: playable native charge/release controls, boats and icebergs, original river/trees/crossbow assets, attached and sinking presents, repeat-hit scoring/time bonuses, 10 levels, pause, restart and resize. Seven source scenarios match at 1e-7, including the ten-level finish; the idle desktop capture is close. Tutorial prompts and a full visual/effect/viewport review remain missing.
- Present Drop: original factory/stage artwork, sled and rope elves, dropping animation, scrolling foreground/background themes, train hands/handles/wheels, rising score flags, original pooled entities, strict collision rules, gravity, levels, time/score bonuses, keyboard/mouse controls, pause/restart and game over. Three original-source scenarios cover idle progression, repeated drops and hits, pooled reuse, pause timers and restart.
- Audio: original Ogg files decoded by Rust; a Rust stereo mixer supports resampling, looping, pause/mute with UI sounds continuing while gameplay is paused, scheduled starts, fades and stopping individual sources. Jetpack/Selfie/Ski/Present Drop actions include their multiple-source starts and tool loop stops. Present Drop's sled loop also has the original clamped stereo panning and bus gain. Full process scheduling, bus effects, group behavior and musical transitions are **not** ported yet.

Most scenes still do not have native gameplay implementations. The six available scenes still have visual/interaction differences and have not passed full visual and behavioral parity. The original blur is approximated by a native Gaussian sampling shader; platform font rasterization, overlay detail, mobile tray layout and full website behavior still have differences. An unavailable activity reports that it is unavailable instead of silently opening another game. Maps/live tracking, videos, translations, accessibility, gamepad support, all remaining games and complete shared overlays still require work. Passing the current tests is not evidence of whole-site parity.

Gift Slingshot is now playable with `--scene boatload`: Up/Down move; hold Space/Right to charge, release to throw. Mouse press/drag/release follows the original touch-target behavior. Source simulation scenarios include collisions, level changes, restart, resize and overlapping refill timers. Its original queued tutorial is implemented; complete effect/viewport comparisons remain unfinished.

Christmas tracking has a tested Rust route engine and delivery-only gate; **the native tracking map/UI is not implemented yet**. `cargo run --manifest-path rust/Cargo.toml --example tracker_reference -- 1798110000000` checks a simulated Christmas route state. It uses the original configured endpoint's bundled 420-stop schedule, shifted to the selected year like the source. It does not yet refresh a live remote route.

## Feedback and continuing work

Read [PARITY-REPORT.txt](PARITY-REPORT.txt) for the per-mode functionality list, including missing scenes and tracking. Regenerate it with `cargo run --manifest-path rust/Cargo.toml --example status_report`.

Edit [PARITY-STATUS.json](PARITY-STATUS.json) to record your verdict for each of the 55 scenes: `user_happy` can be `unreviewed`, `yes`, `partly` or `no`; put details in `user_notes` and choose a `priority`. There are also overall happiness/notes fields. Codex preserves your feedback and updates the separate `agent_*` evidence/progress fields. Seven scenes currently have partial native implementations; 48 remain unported.

The project skill is [maintain-rust-santa-tracker](skills/maintain-rust-santa-tracker/SKILL.md), also installed locally under `~/.codex/skills/maintain-rust-santa-tracker`. It tells future work to read your feedback, compare original source/captures, preserve gameplay during optimization, and publish only the native distribution.

Recent rendering fixes preserve Lottie art proportions in recommendation/menu/village cards, retain premultiplied Selfie images through filtering, and stop discarding fractional animation updates. Selfie also skips repainting an unchanged beard surface and avoids repeated constraint-list allocations. Gameplay steps, solver order and RNG behavior are unchanged. These fixes do not establish complete visual parity.

For live frame diagnostics, run `rust/target/release/santa-tracker --scene santaselfie --mute --profile /tmp/selfie-frames.json --profile-frames 240`. The first 30 frames are warmup. The report separates main-thread update, scene, UI and total work from observed presentation intervals. Game actions are disabled for this diagnostic run; hover visuals can still respond to the pointer. Use an unobscured window with comparable desktop load. This is not an input-to-photon latency measurement. `--profile` cannot be combined with frozen `--frames` captures. First-use village decoding and Ogg decoding still need performance work.

## Build and run

```sh
./rust/build.sh
./rust/run.sh
./rust/run.sh --scene runner
./rust/run.sh --scene jetpack
./rust/run.sh --scene santaselfie
./rust/run.sh --scene elfski
./rust/run.sh --scene presentdrop
./rust/run.sh --scene boatload
```

Building requires Rust, a C/C++ compiler, and libclang with its builtin headers. `build.sh` handles header discovery on Linux installations that have libclang but no clang driver. Set `CARGO=/path/to/cargo` to select a toolchain. On this machine the isolated development toolchain is in `/tmp/santa-cargo` and `/tmp/santa-rustup`:

```sh
RUSTUP_HOME=/tmp/santa-rustup CARGO_HOME=/tmp/santa-cargo \
CARGO=/tmp/santa-cargo/bin/cargo ./rust/build.sh --offline
```

The executable is `rust/target/release/santa-tracker`. Runtime requirements on Linux are an OpenGL-capable desktop and SDL2 for audio. It uses original resources in `../static`; `--assets /path/to/static` overrides that location. The bundled fonts are compiled into the executable. Use `--help` for deterministic screenshot options. `--mute` avoids opening an audio device during render checks.

Elf Ski: hold Down to begin skiing; Left/Right steer, and holding Down with a side key gives a diagonal. You can also hold the mouse to steer. It has no pause action in the original scene config.

Present Drop: Left/Right move the sled; Space/Down drops a present. Hold/move the mouse and release to use the original touch targeting/drop behavior.

Runner controls: Space/Up jumps, Down slides. Jetpack: arrow keys, or hold and drag the mouse. Santa Selfie: select a tool in the tray, then hold/drag on Santa; release the growth spray to regrow hair. The reset button restores the beard. Share saves `santa-selfie.png` and `santa-selfie.beard` in the working directory. This native file export differs from the original site's URL sharing/photocard flow. Restore with `--scene santaselfie --beard-file santa-selfie.beard`.

P pauses, R restarts/resets, M toggles audio, and Escape returns to the village. Village supports mouse wheel/drag scrolling, activity cards and the toolbar menu. For live capture, pass `--screenshot /tmp/scene.png` and press F12; `--frames` captures a frozen deterministic frame after `--advance`.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml
cargo run --manifest-path rust/Cargo.toml --example status_report
cargo run --manifest-path rust/Cargo.toml --example opaque_surface_reference
```

The JavaScript reference exporters and browser capture tools live only in the separate development/reference checkout; they are deliberately omitted from this Rust-only distribution. They are not used by the native application. `reference.cjs` loads the original Runner classes, coordinator, scoreboard and random generator; its output is checked by Rust tests to a numeric tolerance of `1e-7`. `focus-reference.cjs` does the same for the original Jetpack classes/scoreboard and Selfie cloth/point/tool operations. Additional tests check beard encoding, invalid input rejection and tool sound transitions. UI reference tests cover 516 original countdown inputs, card-player reversals and measured responsive card geometry; a sprite-filtering test checks that alpha and visible source artwork remain unchanged. The native animation test checks that an original section animates rather than rendering a static poster and stays premultiplied through filtering. The audio tests check looping, pause/resume, channel output, delayed starts, fades and source stops. The countdown test checks the original UTC deadline. Runner also checks the live delayed level transitions and original canvas offsets, and its layout is checked at five viewport sizes.

Original browser captures go into `rust/target/`, with measured layout data in `data/*layout-reference.json` and Selfie expression measurements in `data/selfie-expression-reference.json`. Native screenshots can be produced using:

```sh
rust/target/release/santa-tracker --mute --frames 2 --screenshot /tmp/village.png
rust/target/release/santa-tracker --mute --scroll 1122 --frames 2 --screenshot /tmp/movies.png
rust/target/release/santa-tracker --scene jetpack --mute --seed test1234 --advance 3 --frames 2 --screenshot /tmp/jetpack.png
rust/target/release/santa-tracker --scene santaselfie --mute --seed test1234 --advance .5 --frames 2 --screenshot /tmp/selfie.png
```

## Attribution

The original Santa Tracker and artwork are by Google and its contributors. Adapted code retains the Apache-2.0 license; original images and audio retain their CC-BY licensing. See the repository's [LICENSE](../LICENSE) and [README](../README.md#license). Exported data tables, keyframes and SVG icon paths come directly from those sources.

Lobster was obtained from the [Google Fonts repository](https://github.com/google/fonts/tree/main/ofl/lobster); Google Sans was obtained from the same Google Fonts service referenced by the original site. Roboto and Open Sans use the same Google Fonts service for the declared UI/score families. Their OFL notices and embedded license metadata are in `assets/fonts/`. Native animation playback uses [ThorVG](https://docs.rs/thorvg/0.5.1/thorvg/); it does not embed a browser.

Elf Ski lighting shaders derive from THREE.js r110 (MIT); the license is included at `assets/shaders/THREE-LICENSE`. Its Perlin noise source is public domain, credited to Stefan Gustavson, Peter Eastman and Joseph Gentle.

The native Rust-only visual comparison harness can render Runner states without browser code: `cargo run --manifest-path rust/Cargo.toml --example runner_reference -- slide /tmp/runner-slide.png`. It draws the actual renderer after the same input/simulation actions as the browser reference.


Tutorial prompts now use the original queue/dismiss timing, input artwork and cloud clip in the five native action games. Elf Ski uses a frame atlas derived from the original video with its sample aspect ratio preserved. Desktop keyboard prompts are the tested input profile; full touch/pressure behavior remains incomplete.

Recent fidelity checks cover Selfie wall-clock blinking while paused, mobile toolbox wrapping, eye artwork and SVG background aspect fitting; shared pause/game-over geometry covers 18 original browser cases. Original/native captures still have small raster/font differences. The final opaque-window pass is GPU-checked to preserve every RGB byte while setting output alpha to opaque. This prevents applying surface transparency twice; it does not certify that every image edge matches the browser.
