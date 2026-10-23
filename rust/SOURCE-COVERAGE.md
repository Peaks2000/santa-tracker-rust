# Source review and native coverage

This is a review ledger, not a claim of complete site parity. JavaScript tools
are development references only; the application executes native Rust.

## Elf Ski

Read in full: `static/scenes/elfski/index.html`, `index.scss`, all scene-specific game
JavaScript files and all four `js/three/` helpers, the imported noise source,
and the relevant THREE r110 / MeshLine rendering routines. The constants module
contains only its Closure namespace declaration.

| Source | Functions and behavior accounted for | Native location / evidence |
| --- | --- | --- |
| `physics.js` | Constructor/reset state, angle/speed getters, angle vector, finite input clamp, angle interpolation, negative acceleration tangent, speed limits, displacement. `stopBar` is unused. The unused `crash()` contains a source `math.sign` ReferenceError; the running game never calls it. | `ski::Character`; source execution comparisons. Native restart reconstructs state. |
| `vec.js` | Reviewed every vector function: inline normalization, unitVec, angleBetween, add, mult, multVec, lerp, dist, slerp. Runtime Ski uses multVec and dist; those calculations are translated at their call sites. Unused slerp has a source `vecB.y === vecB.y` typo; no native game path uses it. | `ski.rs`; remaining unused helpers are not exposed as a native vector library. |
| `noise.js` | Grad/dot2, fixed gradients/permutation, seed(0), fade, lerp, exported Perlin function. No runtime reseeding is called by this scene. | `ski::noise`; world snapshots. |
| `game-three.js` GameThree | Constructor, async model/texture readiness, measure, cameraFocus, render viewport, tick, internalTick, getters, prepareModel. Dispose is empty in source. Original glTF vertices/normals/UV/indices, scale 5, orthographic camera and 40px overscan, spotlight/ambient, tree collision threshold, score, delayed crash report, snowball and local quaternion tumbling. | `ski.rs`, `ski_view.rs`, `assets.rs`; seeded source model/world/rotation snapshots and browser captures. |
| `game-three.js` SceneDecorator | Constructor offsets, update row growth/freeing, treeType, treesNear, presentsNear, tree/present cell formulas, tick, decorate. `_prensets` misspelling is preserved in observable lifetime behavior: removed present objects remain in the collection until row cleanup. | `ski.rs`; source snapshots include reusable tree IDs, float32 positions, presents and rows. |
| `three/points.js` | Constructor uniforms/attributes, next/free/alloc, update range/DPI handling, tree atlas shader, 128px points at height 72, alpha discard and depth. | `ski_view.rs` expands points into depth-preserving quads, including centers outside the physical viewport. Full native buffers replace JS partial attribute update ranges. Trilinear mipmaps match source. |
| `three/present.js` | Original Blender geometry, palette, generateDropTexture/cache, randomColorCombo, constructor RNG order, collected/pickup and tick spin/bob/shrink/remove. | `ski.rs`, `ski_view.rs`, `data/ski-present-geometry.json`; explicit pickup/removal oracle plus movement scenarios. |
| `three/skitrail.js` | Constructor, widthFunc, push and vecFor; 80 points, replacement below 5px, gray offset/white raised layers. MeshLine previous/next normalization, width scale 1.8, taper and indices reviewed. | `ski.rs`, `ski_view.rs`; source trail snapshots and native/source captures. Projection expansion is native CPU math. |
| `three/loader.js` | Shared loader construction, cross-origin options, gltf/texture/immediateTexture entry points. ImmediateTexture is unused by this scene. | Native filesystem decoding; background asset preloading replaces browser promises and network CORS. |
| `index.html` LocalGame | Constructor/restart, skiingNoises transitions, render, resize, pointer capture/pressure/release, key tracking/averaging, targetAngle, tick/score, three-second crash-to-gameover and music/score/crash events. | `main.rs`, `ski_view.rs`, `ski.rs`, `sound.rs`. Arrow keys and held mouse supported. Native mouse does not expose touch pressure or browser pointer capture. |
| `index.html` ready/config | Audio preload/config, 60Hz simulation, independent draw loop, resize, pointer/mouse/touch registration, tutorial queue/dismiss. | Native scene routing/loading, frame loop, controls and shared overlay. **The tutorial MP4, touch fallback details and website API messaging remain unimplemented.** |
| `index.scss` | Body overflow/touch behavior, background, fixed mountain 40px overscan, canvas positioning. | Native window, original background, projection crop and pointer coordinates. |
| Klang processes | `elfski_loop_start`, `elfski_loop_stop`, `elfski_crash`, `generic_score`, music events read from original actions, including the 0.1s loop fades and crash stopping the loop. | `sound.rs`, original Ogg samples. Full shared music/bus/effect behavior remains incomplete. |

The cloned transparent model material in `_prepareModel` is deliberately **not**
used, matching the commented-out assignment in the source. The source creates
`_spottarget` but never assigns it to `SpotLight.target`; native lighting keeps
the original world-origin target. These are checked source behaviors, not fixes.

### Checks performed

- Original JS executed in a Node VM for down/steering/idle scenarios and explicit
  collection/removal; native values checked to 1e-7. Tree positions are compared
  after float32 conversion, as in the source GPU buffers.
- Original THREE runs in Chromium for deterministic idle/moving/crash captures;
  model lighting uses its assembled Standard/Phong GLSL with native attribute and
  matrix-uniform adapters. GLSL is shader code, not a JavaScript game runtime.
- Native arrow-key input, steering, delayed crash overlay, restart and return
  home exercised in a separate smoke window. Audio still needs listening checks;
  simulation snapshots alone do not cover rendering/input paths.
- Platform font rasterization, shared overlay blur, tutorial, touch pressure,
  high-DPI/multisample details and complete shared music remain parity gaps.

## Shared image edges

Opaque offscreen scenes are presented with an opaque RGB copy. The default
Macroquad alpha blend left alpha below one at antialiased sprite edges; blending
the scene texture again darkened those edges. The copy avoids that second alpha
application. Shared 2D asset decoding also extends edge RGB beneath fully
transparent texels, preserving every alpha and every visible source texel. The
3D Elf Ski maps retain the source THREE mipmap behavior.

## Present Drop

Read in full: `index.html`, `presentdrop-scene.scss`, all five SCSS partials,
all seven gameplay JS files and imported shared `pools.js`, `utils.js` and
`scoreboard.js`.

| Source | Functions and behavior accounted for | Evidence / native location |
| --- | --- | --- |
| `constants.js` | All values reviewed; TIME_PER_CHIMNEY is unused (hitChimney adds one second), tutorial/legacy countdown constants identified. | `presentdrop.rs`; source oracle. |
| `game.js` | Constructor/preloadPools, start/restart, onFrame update and insertion order, scale/resize, chimney spawning, createPresent/hitChimney, forEachActiveChimney (includes hit/dead chimneys still in the list), freeze/unfreeze, nextLevel, gameover, pause/resume, dispose. | Native model/main/view; pooled reset and timer scenarios. DOM/network analytics/listener disposal become native object lifetime. URL debug `?level=` and tutorial are not implemented. |
| `player.js` | Constructor/reset, onFrame strict overshoot, pull loop changes, position updates every third moving frame, setX clamps/elf-facing, keyboard methods, touchEnded, dropPresent/cooldown/newpresent class. | `presentdrop.rs` and view; wall-clock cooldown continues during pause, including callbacks surviving restart. |
| `present.js` | Constructor/pool/onInit/remove/dispose/draw, semi-implicit gravity/cap/miss, strict vertical crossing and horizontal hit, multiple-chimney iteration, legacy unused drop DOM helper. | Native model; source collision scenarios. Unused DOM reparenting helper has no native caller. |
| `chimney.js` | Constructor/pool/onInit/remove/dispose/draw/onFrame/hit/calculateScore/getHitbox. onInit does not clear the large class/flag, so pool reuse retains large geometry. Hit flag timers clear the prior hit timer. | Native model/view; FIFO pools and source hit scores/time bonuses. |
| `levels.js` | Constructor/reset, strict foreground wrap, every-third-loop transition, old/new foreground order, background motion/wrap and nextLevel notification after layout update. | Native model; snapshots cross level boundaries. |
| `controls.js` | Constructor/key flags, down/up/right priority, edge-triggered drop, tutorial dismissal, touch-start/move/end, GUI rejection, active-touch lookup and coordinate conversion. | Native keys and mouse emulate the primary touch path. Multi-touch IDs, touch orientation overlay and tutorials remain gaps. |
| HTML/SCSS | Environment gradients, house/pseudo-elements, stage 889px clipping, exact sprite crops, view origin/scale, scrolling layers, rope-end rotations, elves/sled, step-start new-present frames, hand/handle/wheel keyframes, flag transition. | `presentdrop_view.rs`; original DOM geometry capture and deterministic 3s browser comparison. Flag font rasterization still needs closer comparison. |
| Shared scoreboard/pools/utils | FIFO ensure/init/free/restart, dead cleanup, countdown ceil/negative-only gameover/hurry state, score/time/level updates, animation-class lifetime. Unused vector/DOM/animation utilities reviewed. | Native model/source execution oracle; app uses direct data/geometry instead of DOM classes. |
| Klang actions | All pd_ actions reviewed, including empty start/crash/pause handlers, source/group IDs, pull loop fade/stop, sled StereoPanner clamp to +/-0.5 and bus output gain 0.7. | `sound.rs`, `audio.rs`; stereo pan test checks an ongoing stereo sample. Full shared music/group/effect behavior remains incomplete. |

## Next scene reviews

Boatload remains selected for a later complete source review and port. The
Runner parity pass below addresses the user's next requested priority.

## Reindeer Runner parity pass

Read every line of `index.html`, `runner-scene.scss`, all 13 SCSS partials,
and all 12 files in `js/`. Also reviewed complete shared Coordinator, pools,
scoreboard, random and utility sources used by these games.

| Source | Review and resulting native behavior |
| --- | --- |
| `constants.js` | All collision dimensions remain distinct from the larger SVG art dimensions. The duplicate tall-tree entry/type in the original table is retained. |
| `game.js` | Constructor/preloadPools, start/restart, pause/resume/freeze/unfreeze, gameover/dispose, step/advanceTime/onFrame, delayed updateLevel and stripes, scene size, every spawning helper, score/time/finish relocation and setLevelClass checked. Native live timers now have a separate source oracle, alongside the existing five manual-time scenarios. Visible spawn width follows viewport width * 1.5. Restart preserves the RNG, background object and running animation clock. |
| `player.js` | Constructor/reset/newState, up/down/endSlide, every integration branch in onFrame, draw/hit cloud/magnet placement, toggleMagnet, all collision/platform helpers, land/getHitbox checked. Visuals use the hitbox-derived positions, eased cloud/magnet effects and correct state-specific sprite phases. The two-second touch slide has a native coordinator timer. |
| `controls.js` | Handle dispatch, shared Space/Up gate, Down/release, GUI touch rejection, touch start/move/end and touch lookup checked. Native mouse release maps the touch tap/up/down thresholds; actual multi-touch IDs and tutorials remain gaps. |
| `canvas.js` | imageLoader count/error/callback handling, constructor, drawDefaultImage and every draw/wrap/clear condition in onFrame reviewed. Native mountain/tree offsets preserve the source per-frame increments and wrap points. The CSS stretches the 670px canvas buffer into an 810px element; that stretch and central-only mountain clipping are now preserved. Filesystem decoding replaces asynchronous browser Image objects. |
| `ceiling.js` | Constructor/resize handler reviewed. CSS media-query scales, quantized wall height, portrait ceiling offset and stage origin are compared against original DOM geometry at five viewport sizes. |
| `present.js` | Constructor/pool/init/dispose/onFrame/hit/draw/getHitbox/setXPos/distanceFromPlayer checked. Native pickup growth/fade follows CSS ease; animation starts at spawn, magnet attraction retains its frame-dependent source arithmetic. |
| `boost.js` | Constructor/pool/init/dispose/onFrame/hit/draw/getHitbox checked. Native time/magnet groups use source geometry, spin and eased pickup effects; time bonus text has its delayed fade. |
| `platform.js`, `obstacle.js` | All constructors, pools, init/dispose/draw/frame/hit/getHitbox methods reviewed. Native renderer includes the long/short rollers and their wires, correct art sizes and obstacle bottom offset; trashcan art switches to its hit frame. |
| `finish.js` | Constructor/place/onFrame/hit/getHitbox checked. Collision height stays 265 while art is 380px tall, preserving the original finish jump. |
| `timer.js` | Constructor, pause and resume reviewed. Separate live-level snapshots exercise the delayed 200ms/2s transition path. Coordinator-driven gameplay pauses; CSS opacity/scale transitions use a separate wall clock. |
| SCSS/HTML | Correct layer origin (352.5px before the stage), discrete responsive scales, background sizing/cover/repeat, clipping and stacking, cogs/pillars, missing foreground gears/cooling station, 40px ceiling repeat, treadmill belts/negative gear delays, run/slide step-start frames, hit cloud/magnet/boost text, pickup ease and Lobster speedometer restored. Warning shape is vector geometry exported from the original CSS transforms, including its rounded corner. The source scenery speed selector does not match the root level class, so treadmill speed remains at its original constant. |
| Klang | All runner_ actions read, including empty processes and the game-over source fade. Native hit/jump/pickup IDs are explicit. Full shared bus/music/group semantics remain incomplete. |

Original/native captures now cover idle, moving, jump and slide references;
a separate native smoke window exercised jump/double-jump/down, pause/resume,
level changes, restart/slide and return home. The actual home Play button was
also clicked twice, opening Selfie and then Jetpack after returning home.
The native SVG/font rasterizers still differ from Chromium, and the speedometer
warning's synthetic bold glyph, full touch/tutorial/accessibility behavior,
rare mid-transition restart cases and all shared website functionality still
need parity work. This pass is not a claim that Runner or the site is fully 1:1.

## Home Play and recommendations

Read `modvil/index.html`'s data listener/button handler and the complete
recommendation closure in `static/entrypoint.js` (recent routes, biased sampling,
featured override and Play payload). Native `navigation.rs` now supplies both
home Play and the two overlay cards from one state object. It preserves the
source pow(random,4) bias, ordered recent-route buffer and no-replacement draws.
A source-executed fixture checks initial navigation and repeated home/game
visits. The native pool contains only five playable activities; when it runs out
of unseen options, a documented fallback chooses an older playable activity
instead of reproducing the original algorithm's undefined result on an
undersized list. Remote Firebase locks/featured scheduling and localStorage
featured-played persistence remain unimplemented.

## Shared art fitting, Selfie compositing and performance review (2026-10-23)

Reviewed the full `santa-card-player.js`, `santa-card.js`, `santa-card.scss` and
`deps/lottie.js`, the Selfie background/responsive styles and mouse scale function,
and rechecked Cloth drawHairShape/draw and Point drawHair/drawSpray/drawDecoration.
The installed ThorVG 1.1.0 Lottie loader independently scales X/Y in `resize()`;
its Rust wrapper's preserve-aspect documentation does not describe this backend.
Native Clip now uses a uniform fitted size plus centered translation, matching
lottie-web's default xMidYMid meet. A regression test compares a fitted 128x100
render with the center of a 256x100 render and checks the transparent sides.
The separate browser card fixture is an isolated **280x120** host under modvil's
minimum-width rules; its canvas is 238x72. It is not an overlay host-size oracle.
Native/source Jetpack idle art bounds in that canvas were (87,7,140,59) and
(87,7,140,58), respectively. Full overlay fonts and layout remain partial.
`examples/lottie_reference.rs` provides a native CPU PNG comparison renderer.

Selfie SVG uploads now retain resvg's premultiplied bytes; decoded raster images
are premultiplied once. Tool/background/cursor/sprite draws and beard decorations
use ONE source-over blending. Spray remains source-atop using destination alpha
and preserving beard alpha. Hair remains straight white with correct separate
alpha accumulation. Captured native tool edges and a restored red beard after
these changes; small source rasterization differences remain. Other scene views
still use straight assets plus shared transparent-edge bleed; these were not
silently converted without changing their matching materials.

The beard surface cache checks x/y/rotation/draw/spray/decoration for every point
and its target dimensions; it still composites at the current face position on
every display frame. Constraint indices are read in the same reverse order with
no per-point cloned vector. The original seeded physics/tool scenarios still pass
at 1e-7. No simulation step or input buffering delay was added. Lottie playback
now accepts fractional frame changes >=0.001 instead of dropping changes <0.5.
A reusable ThorVG target buffer reduces allocation churn; each draw clears it.

Opt-in `--profile` records live CPU stages and observed frame intervals after
30 warmup frames. A pre-change 240-frame 1440x800 muted idle Selfie run measured
CPU work p95 0.857ms, p99 0.908ms; interval p95 18.114ms. A village scroll=2200
baseline had CPU p95 4.790ms, p99 8.899ms. Post-change runs experienced long
presentation waits in this desktop session (including ~1s intervals); they are
not evidence of a speedup or input latency reduction. Cold village image/clip
loading and first-use Ogg decode remain synchronous and require separate work.
Validation: release build; 25 native tests; original Selfie/card browser capture;
native frozen Selfie and restored colored-beard captures. No additional native
gamemode was completed in this shared rendering/performance pass.

## Gift Slingshot / Boatload (2026-10-23)

Read in full: scene `index.html`, `boatload-scene.scss`, all six scene Sass
partials and all eight JavaScript files (`game`, `controls`, `player`, `present`,
`boat`, `iceberg`, `bubble`, `constants`). Re-read the shared pools, Coordinator
and Scoreboard implementations and the active animWithClass helper. Native
locations: `boatload.rs`, `boatload_view.rs`, `main.rs`, `sound.rs`.

| Source | Functions/behavior accounted for | Evidence/limits |
| --- | --- | --- |
| Game | Constructor state, resize/watchSceneSize/setScale, start/restart/preloaded FIFO pools, step/onFrame/manual advance, boat candidate/collision scheduling, iceberg spacing rejection, level speeds/countdown, dropPresent/hitBoat/missedBoat, pause/resume/gameover and effects | Seven source-executed scenarios compare state to 1e-7, including 100ms of original Controls interval ticks, boat hits, ice attachments, misses, all ten levels, timer and level game over, restart, resize and overlapping refill callbacks after rapid restart. Live native simulation uses display delta like source onFrame; deterministic advance uses the source 60Hz test helper. |
| Player | Reset/rail dimensions and retained distance/flags, target clamping, up/down/stop, arrival overshoot, touch dropWhenStopped, prepare/reverse-throw Coordinator callbacks, addX/rope/elf transforms, autoPrepare, delayed refill | Source timer callbacks are distinct and continue while paused. Restart retains pooled/random state; new scene navigation creates a new game. The unused soundFrameCounter does not trigger audio in source and isn't exported as a native observable. |
| Present | Constructor type RNG, onInit/dispose, flight speed/3, pre-step end test, endpoint clamp, all-entity collision order, scaling arc, missed Coordinator drift/sink | Native visible/pool state is separate from active entities so presents attached to ice continue drawing/moving. OnInit type stays fixed across reuse. Original inline rectangle/ribbon SVG geometry is drawn natively. |
| Boat/Iceberg | Type geometry, hitboxes, movement/disposal, nextBubble RNG, repeat-hit multipliers/time bonus, ice attached-present cleanup and motion | FIFO pools preserve reuse. Logic dimensions retain the original one-pixel differences from CSS art boxes. |
| Bubble | Constructor type RNG, pooled reuse, position and 600ms opacity transition | onInit overrides the namespaced CSS animation with unresolved `bubble-left`; native keeps the source's stationary fading behavior rather than inventing motion. |
| Controls | Source Up precedence, Down/stop targets, Space/Right shared latch, 10ms charge ticks of 0.15 capped at 3, prepare/release, touch target/release semantics | Native mouse emulates the original touch path. Multi-touch identifier bookkeeping has no separate native multi-touch implementation. Tutorial prompt dismissal calls are reviewed; the tutorial overlay itself remains unported. |
| HTML/SCSS | Image preloads, layer order, 42% stage/transform origin, responsive scale/minimum scene size, river border/shadow, repeating rail/side strokes, original tree/boat/iceberg art, crossbow/elf/rope geometry, inline present colors, effects and frozen classes | A browser/native 1440x800, seed test1234, advance .5s idle comparison had full-image mean RGB error 0.282/255. This is one idle capture, not full visual parity. Score pop-up text/animation and all effect/responsive phases still need matched captures. |
| Audio | Read Klang action bodies for bl_game_start/stop, bl_shoot, bl_hit_ice/water, bl_score_yellow/red, bl_rope_stretch and p_rope_stretch_stop | Native explicit source play/stop/fade actions retain rope-stop side effects. Full mixer/music bus fidelity is still incomplete. |

Analytics calls are not sent by the native application. DOM listener binding,
requestAnimationFrame cancellation and pools.empty on iframe destruction become
native input/window ownership and reconstructing a scene; no JS runs in product.
The optional source `?level=` debug override and tutorial are not implemented.
The shared LevelUp object is constructed by source but never shown by its level
update path; native does not invent a level-up screen. Coverage is **partial**.

## Christmas route engine and visibility (2026-10-23)

Read all of `modvil/elements/maputils.js`, `src/api/spherical.js`,
`src/core/season.js`, `modvil-tracker.js`, `modvil-tracker-stats.js` and their
tracker/stat Sass, plus the scene's tracker construction/config handler and
`prod/src/remote-config-defaults.js` route settings. The original configured
English route URL was reachable and returned 420 destinations. Its schedule
is from 2019; source fetchRoute deliberately skews that schedule to the current
year's Dec 24 10:00 UTC through Dec 25 11:00 UTC. The bundled route retains
the complete original JSON, including photo/attribution metadata for future UI
work; provenance is recorded beside it. Native currently reads DataManager and
timezone fields.

`tracker.rs` translates route skew (including zero timestamp preservation),
DataManager index advancement/backward reset, home/stop/flight details,
30% transit / 70% city present interpolation with JS rounding, cumulative and
partial great-circle distance, heading, spherical interpolation's small-angle
fallback, geographic nearest arrival and timezone/midnight fallback. A separate
visibility-gated entry returns None outside the delivery window and rebases
for later years. The user requested delivery-only visibility; the source's
broader Dec 1-25 season check PLUS remote showTracker flag is not conflated
with that flight interval. Boundary tests cover before takeoff, takeoff,
just before return, return, off-season and a new year.

3,020 states executed from the original fetchRoute/DataManager (every stop's
arrival/departure +/-1ms, 500 flight samples, reverse time and home endpoints)
match gifts, flags and stop ID exactly; distance/heading/next within 1e-6 and
coordinates within 1e-9. Enabling serde_json float_roundtrip was necessary to
avoid decimal epoch-ms parsing losing one ULP and shifting an interpolated
position. This improves JSON numerical fidelity; all existing native source
oracles still pass.

**Not implemented:** native tracking map/screen, Google Maps integration,
StopManager markers/trails, map focus/zoom/resizing, feed/photos, remote config,
online route refresh, destination cache and geolocation. `tracker_reference`
is a headless diagnostic only. Do not describe the Christmas tracker screen
as working based on these engine tests. The source's fallback fetch typo and
negative stop-index bug were reviewed; native does not reproduce broken network
calls or expose that unused negative-index accessor. There is no fake map or
always-visible tracker added to the normal village.
