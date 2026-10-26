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


## Shared tutorial, Selfie and overlay follow-up (2026-10-23)

Reviewed every line of `_shared/js/tutorial.js`, `src/elements/santa-tutorial.js`
and its Sass, media/prefix helpers, global input-filter configuration, loader
holder styles, and the five native games' queue/dismiss handlers. `tutorial.rs`
translates ordered queues, persistent dismissals, reset, filter/duplicate quirks,
load failures and the 1500ms display delay. Three original-class scenarios in
`data/tutorial-reference.json` pass. The renderer uses original SVG strips,
cloud mask, source steps/end timing, group opacity and shadow. The original Ski
video is a 95-frame/16fps atlas, preserving its 54:65 sample aspect ratio; see
`assets/tutorial/elfski.provenance.json`. No video decoder or JavaScript runs in
product. Desktop keyboard is the tested profile; touch/pressure remains partial.
Earlier statements that native action-game tutorials are absent are superseded.

Re-read Selfie background/tools/santa/expressions styles, HTML, game timing,
Mouse scaling, full Face/Utils and hair drawing. Wall-clock blink deadlines and
RNG now match 1200 original frames including paused cloth. Eyes contain the
original blink/closed artwork, dryer expressions use source branches, and hair
uses the source backing-canvas dimensions. The six-tool mobile tray, flex wrap,
short-desktop offset and exact media-query boundaries match exported DOM boxes.
SVG background fitting preserves the mirror/chair aspect; nose scaleY is applied
after fitting. `data/selfie-parity-layout-reference.json` and the toolbox unit
test cover desktop, mobile and boundary sizes. The matched 1440x800 idle mean
RGB error improved from 1.225 to 0.610/255. These are controlled captures, not
complete visual certification or a performance/input-latency measurement.

Re-read full `santa-overlay.js`/Sass. `overlay-layout-reference.json` has 18
pause/game-over DOM cases over nine viewport sizes. Rust geometry tests cover
compact cards, column/row breakpoints, hero/navigation and recommendations.
The below-content transition now uses its own opacity and correct translation
axis; short landscape puts recommendations to the right. Game-over text uses
the source exclamation mark. A matched 1440x400 game-over capture has mean RGB
error 0.679/255. Font raster/bold treatment, card asset loading/fades, all
transition phases and further viewport captures remain incomplete.

Tutorial captures at 1440x800 have mean RGB errors 0.301 (space frame 0), 0.312
(space frame 1), 0.247 (up/down frame 1), and 0.221/255 (Ski at 2.5s).
These numbers describe only those captures. Examples `tutorial_reference`,
`selfie_reference` and `overlay_reference` render production Rust views.
The separate GPU `opaque_surface_reference` verifies byte-identical RGB and
alpha=255 after sealing 30,896 previously translucent output pixels. This fixes
unintended window/output transparency without erasing source artwork outlines.
Small SVG/filter/font differences still need review. All seven scenes remain
partial; full audio/music, touch, sharing and whole-site parity are unfinished.

Release validation: 33 tests pass; offline release build succeeds. An owned
Boatload window exercised tutorial display, pause/resume, next queued hint and
keyboard dismissal; all captured final surfaces had alpha 255. Desktop capture
can stall, so this smoke is not evidence of minimum input latency.


## Elf Maker and Elf Jamband native ports (2026-10-24)

Reviewed the complete Elf Maker index.js/index.html, defs.js geometry functions
and embedded SVG definitions, maker-control/chooser/elf/photo JavaScript and
all four Sass files, plus shared santa-choice input/scroll handling and styles,
santa-button behavior/styles and the corresponding Klang actions. Native model
covers category order, seven saved choices/seven colours, weighted randomization,
source SVG colour classes, base64 comma/nonce format, restore and all eight motion
sequences. Arm targets preserve the source's false/right versus render-left
naming quirk, sinusoidal target sway and sine interpolation from the old pose.
Blink uses the 5.234s alternating source keyframes; unused old dance helpers and
_isDancing do not affect the source render and are not invented as new movement.
The original 7 bodies, 39 hairstyles, 3 glasses, 9 earrings, 10 hats/accessories/
backgrounds are retained as SVG/image data. Geometry is generated in Rust.
21 original control states match choices, colours and serialization; 35 body/
pose cases compare original versus Rust-generated SVG rasters (total byte error
under 100 per 400x560 image). Desktop/mobile layout measurements are bundled in
elfmaker-layout-reference.json. Camera/export, flash, floating photo, category/
colour/preview selection, desktop drag/held-arrow/wheel scrolling and native
saved-code restore are implemented. Invalid out-of-range category codes are
clamped rather than allowing the original's later undefined-body render crash.

Reviewed all Jamband game/audio/instruments/dragdrop/drawer/lights JavaScript,
extern declarations, full HTML/layer order and all seven Sass partials plus
shared legacy button styles. Native model translates 120 BPM next-bar scheduling,
callback cancellation, six pattern/volume positions, drag swap/replace/return,
reset, Fisher-Yates randomization, duplicate-ID restore and empty-band encoding.
19 original browser action/state cases exercise the real Draggable handlers;
programmatic putOnStage alone does not perform their swap logic. Ten characters
retain their nested original SVG layers (including Sax and Xylo child transforms),
waiting/playing classes, both patterns and CSS timing/origins. 1,568 browser
part-transform/opacity samples pass within 0.02 source pixels/matrix coefficients
and 0.001 opacity. Stage/drawer boxes at 1440x800 are exported separately. Source
keyframe and art data is interpreted by Rust; no browser/JS is shipped or used
at runtime. Native lighting thresholds, fades, paused/resumed floor motion,
crowd/background animation, drawer hide/scroll and collapsing containers exist;
further full-scene phase/capture checks remain necessary.

Read Klang's jamband_play_track and preview_instrument actions completely:
native uses the two exact loop sources per instrument, 0.2s fade-outs, stage
volumes, maximum active loop position for joining tracks and one-second preview
fade-outs. Original crowd loop is included. Elf Maker has source switch type/
colour/photo actions and the switch-item counter's extra sound every eighth
call, plus lounge music. Samples are decoded on scene entry, avoiding decoding
at the first musical bar; cold entry can still block. Full global mixer/bus
transition fidelity and audio-device timing are not certified. The source
n6Xc process started by these scenes has an empty action.

Known native differences: file export replaces host URL/link sharing; Maker's
photo floating/perspective/reveal/hide and save-button transitions need complete
source matching; full touch, keyboard focus/accessibility and all responsive
sizes have not been verified. Jamband drawer scroll easing and every drag/light
transition still need review. Selected idle captures and per-part matrices do
not establish whole-mode 1:1 parity. Both modes remain partial.

A release smoke exposed Macroquad's 32-pipeline limit when every view was
constructed together. Identical Shapes materials now share weakly cached GPU
resources; uniforms still break batches before each draw. Weak ownership lets
the last view release the resources before its GL context closes.

Performance follow-up: the first Maker renderer spent about 59ms/frame in scene
CPU work (90 measured frames after 30 warmup). A CPU-only diagnostic isolated
its SVG Gaussian drop-shadow: 67.584ms with premultiplied output, versus 0.569ms
for the same unfiltered SVG. The runtime now retains premultiplied raster pixels
and applies the same 4px/4px, sigma-2 shadow with a GPU Gaussian alpha kernel;
it does not change pose calculations, animation clocks or movement. The matched
1440x800 idle mean RGB error is 0.393/255 (previous software-filter capture 0.408).
Jamband's initial idle profile showed 16.73ms mean intervals and 0.47ms scene
CPU work. These are local measurements, not input-to-photon latency guarantees.

After the shadow change, the same Maker idle release profile (90 frames after
30 warmup, 1440x800, seed test1234) averaged 16.59ms frame intervals, 1.36ms
scene CPU work and 1.39ms total CPU work. Mean intervals before were 60.16ms.
Full summary statistics are in data/new-scenes-performance.json. These results
cover the tested idle costume, not every costume/photo/input path.

Live release interaction checks passed for Maker category selection, randomize,
photo reveal/save (640x1000 PNG plus original-format state), and Jamband drawer
drag-to-stage, delayed playing, band export, hide/open and randomize. Audio ran
through SDL's dummy device for decoder/mixer smoke; this is not an audible A/B
comparison. Shared-material GPU check creates 64 Shapes views and verifies
distinct per-draw colours and the unchanged-RGB opaque output pass.

Final regression: all 37 tests pass in the clean Rust-only distribution; the
offline release build succeeds. All nine scenes start and save opaque captures
using only distribution assets. No browser/reference source is required.


## Elf Glider and activity-menu repair (2026-10-24)

Read every Glider JS file (constants, entity, game, controls, player, item,
obstacle, building, chimney, present, cloud, collision and utils), complete HTML
and all seven Sass partials/scene styles. Rechecked shared SmartValue, FIFO
pools and Scoreboard and the corresponding Klang actions. The scene graph is
translated into native entity/pool IDs with chimney children; constructor-time
random consumption and FIFO reuse match the original, including the two
preloaded random present styles and twenty preloaded random building types.

Seven original-code scenarios compare 52 states at 1e-7 numeric tolerance:
idle, movement/held drops, pause/hit/drop callbacks, restart callbacks, chimney
attempts, forced candy/clock/chimney/balloon collisions and extended progression.
The scoring case verifies +200 candy, +5 seconds and a +300 chimney. Source
resize scaling, velocity acceleration/bounds, shake, current-position delayed
drops, reload timing, gravity, old wall callbacks surviving restart, invincibility,
balloon cleanup, weighted items, inherited obstacle animateBack and deterministic
building intervals are retained. Rendering follows allocation/DOM order within
each layer rather than graph insertion order when pooled elements are reused.
The original's misleading obstacle speed constants are unused: obstacles actually
use ITEM_SPEED_NORMAL/FAST, as the source does. Unused score/time helper results
do not invent rewards on obstacle hits. TOTAL_LEVELS is display metadata; source
updateLevel_ does not enforce a ten-level finish, so native does not add one.

CSS animation end for collision puffs is represented as a cancellable native
wall event; hiding a puff cancels that event, unlike the independent JS balloon
cleanup timeout. Player animation-name changes reset their CSS phase: drop,
pulse, resizing/none and normal are separate modes, with a clock that stops
while paused. Crash/reload/drop JS callbacks continue on wall time while paused.
Keyboard state ignores releases while paused, preserving the source behavior.
147 original device-motion inputs verify the asymmetric +90/-90 landscape
thresholds; native OS sensor delivery remains unimplemented. Keyboard is the
verified playable control path; this is not a claim of touch/tilt completion.

Player/balloon/bird/frisbee step-start sprite boundaries are measured in Chrome,
including duplicate 84% source keyframes. Original SVGs, cloud scaling origins,
item spins, collision scale/fade, points and score fades, background gradients/
scrolling, all 26 building visual sizes and responsive em scaling are retained.
Source CSS/default body margin gives scenery a static x offset of 8px; player
and presents have explicit left/top zero, while collision puffs also inherit
static top 8px. Physics does not inherit those visual offsets. CSS chimney
positions differing from collision constants are kept separately. Five matched
captures at 1440x800 (0/4/10s), 768x800 and 375x800 (4s) have mean RGB errors
0.462, 0.445, 0.684, 0.466 and 0.496/255. Small raster differences remain, and
these captures do not certify every animation/effect phase.

Glider is integrated into native routes, biased Play recommendations, source
keyboard tutorial queues, HUD, pause/restart/game-over and transitions. Audio
uses the exact source wind, drop, collision, chimney/score and clock actions;
samples are predecoded at scene entry. Complete global music/bus transitions,
sensor delivery, all score/effect captures and additional interactive/responsive
cases remain partial. No original JS/HTML/CSS runs in the product.

The reported Jamband hamburger failure was a real integration bug: Chrome
changed village.menu, but draw_menu and its input/transition clocks were called
only from Village::draw. activity_menu now processes the existing selector above
activity scenes, with Chrome still drawn above it and game inputs blocked.
Village ambience is not emitted by that activity-only path. Owned release checks
opened the selector and selected Santa Selfie from both Jamband and Elf Maker.

The browser player-animation lifecycle also matches fourteen original drop/hit/
pause/restart checkpoints, including a still-pending drop after rapid restart.
A 120-frame release idle profile after 30 warmup frames at 1440x800 averaged
16.90ms frame intervals (p50 16.70, p95 21.25, max 39.48), 0.38ms scene CPU
work and 0.46ms total CPU work. The occasional presentation stalls remain in
the measurements; these are not input-to-photon latency figures or guarantees
for every device/state. See data/glider-performance.json.

Final validation: all 42 Rust tests pass in both checkouts. Release startup and
opaque screenshot checks pass for all ten native scenes using only the clean
distribution assets. Three original-browser Glider tests pass.

### Activity header correction

The prior Jamband/Maker selector repair fixed menu rendering but retained an
incorrect icon decision. The original entrypoint.js sets showHome from route
and control readiness, not HUD presence; santa-chrome.js renders paths.home
and dispatches the home event for activity routes. Native Chrome now receives
the current route separately from GameHud: all nine activities use Home, and
the ready village uses Menu. A regression checks every native route, including
Jamband and Maker which have no GameHud.

Validation: 43 Rust tests pass and the release build succeeds. Release captures
show Home in Jamband and Maker; owned-window clicks enter the home transition
from both activities. Existing user windows were left open.

### Penguin Dash — native port (partial)

Read every line of constants.js, game.js, start.js, preloader.js, penguin.js,
wave.js and config/levels.js, the complete scene HTML, scene SCSS, lockscreen
partial and shared level-up SCSS/JS. Rechecked Scoreboard and the used Phaser
Arcade body pre/post updates, AABB/separation, default camera follow, shortest
angle, sprite-child transforms and tween lifecycle. Read every pnd_* Klang
action. The browser engine remains development reference material only.

The native model uses the first ten configured levels; the five old levels
remaining in the source array are retained as reference data and not exposed.
All 1,076 generated objects match source keys, positions, signed visual sizes
(accounting for flips) and body rectangles. This includes the invisible pole
image's real collider, obstacle offsets, zero-sized character bodies, rotated/
flipped finish anchors, generated shadows and seeded presents. Present Y
placement intentionally uses tile width, as in the source. Dead-level retries
retain collected presents and the current moving-platform offset.

116 checkpoints execute original Start/Penguin with the bundled real Phaser
engine, not a replacement physics mock. The development harness manually drives
its stage pre/state/tween/post pipeline at 60 Hz and the source counter at one
second intervals, and controls JS wall callbacks. They cover keyboard inertia,
angle steering, fast-ice entry, obstacle collision, falling/death retry, pause,
pause during a pending fall, moving platforms, gifts and finishes on all ten
levels. Player sprite frame indices are compared alongside gameplay state.
108 direct original orientation cases check truthy alpha/beta/gamma gating,
paused/dead filtering, asymmetric axis gains and tutorial dismissal. OS sensor
input is not connected in the desktop application.

Arcade integrates the previous velocity before scene input changes it. Fast ice
skips snow decay and doubles velocity only on entry. Falling decays velocity
by 0.93 per tick and retries after a 2.5-second JS wall timeout, adding 20
seconds to the count-up clock. That timeout can unpause the engine while the
outer paused flag remains set; the native model keeps those flags separate.
The old level remains visible until the level-up callback covers the scene.
Level completion stops platform tweens on the finish tick; the source's sprite
transform/body lag after a level reset is preserved. Final level completion
has no extra +50 award. The unused celebrating animation is retained without
inventing a gameplay trigger. The original final analytics path reads an
undefined this.scoreboard after freezing/showing game-over; the oracle ignores
that analytics exception only after those effects, and native does not panic.

Rendering uses original PNG sheets (including the actual 15 available dust
frames), source anchors, child shadow/dust transforms, original character
sequences, camera rounding and world bounds. The original water MP4 is decoded
into 194 full-resolution PNG frames with Chromium, preserving its browser color
conversion; native decoding runs on a bounded background worker and uploads
only arriving frames. Initial captures at 1440x800, 768x800 and 375x800 have
mean RGB errors 0.025, 0.196 and 0.320/255 respectively. These measure the
initial scene canvas, not all animations or the site's shared toolbar.

All source sound actions are mapped, including delayed freeze/fail, pickup,
restart, hit, level completion, slide start/stop, velocity-based slide gain and
the two-second fast-ice sound throttle. Full audio group selection/bus/pitch
fidelity and listening comparisons remain partial. Source level-up geometry/
easing is translated, but every phase still needs matched browser captures.
The desktop port does not claim full touch/tilt, accessibility or whole-mode
1:1 certification. Full-game restart still uses the supplied deterministic
seed layout; source continuous RNG consumption across restarts needs follow-up.

Owned release UI checks verified arrow movement, pause/resume, restart and
Home returning to the village. Synthetic X11 key timestamps were corrected
in the development checker so repeated keys are not mistaken for autorepeat.
A 120-frame idle release profile after 30 warmup frames measured
7.07ms mean frame intervals (p95 9.36ms,
max 14.46ms), 1.62ms scene CPU work and
1.67ms total CPU work. This includes water frame uploads;
it does not measure input-to-photon latency. See penguin-performance.json.
All 46 Rust tests pass.

Final distribution validation: all 46 tests pass against the Rust-only checkout.
Release startup/opaque captures pass for all eleven scenes using its assets.
Penguin audio predecode/mixing also passes with SDL dummy (not a listening test).

### Live Penguin bauble placement and full restart correction

Checked the deployed v202610222036 Penguin Dash HTML and 0.js bundle directly.
The live additionalGroupInit_ uses Math.random for prize type, tile selection
and x/y placement; the first ten course definitions exactly match the checkout.
All six live element-present images also match the checkout byte-for-byte;
three shapes are round baubles and three are rectangular presents.
No artwork was replaced. A preliminary artwork-mismatch inference was wrong
and was corrected after checking hashes, before making any asset changes.

The live browser audit exposes only the game handle, without seeding RNG or
changing placement functions. It executes the real restartLevel_ and full
Game.restart: level retry retains coordinates, while full restart regenerates
them. Math.random values are recorded unchanged around the original placement
function. The native production placement function reproduces every resulting
key, coordinate and size on all ten levels from that recorded tape. See
penguin-live-placement-reference.json for URLs, bundle hash, actual draws and
before/retry/restart layouts. This is direct evidence of live behavior, not
an inference from a seed-controlled mock.

Previously full native restart reset the seed and reused the same prizes.
It now continues the RNG stream; four initial waves consume twelve draws before
prize generation, and full restart takes the latest wave-renderer RNG state.
The deterministic source harness now seeds once before waves rather than
reseeding again before levels. Initial numeric fixtures/captures are refreshed
for that corrected order. Dedicated tests retain collected prizes and positions
on level retry and verify fresh layouts on successive full restarts.

The new seed order also exercised a bundled-Phaser transition quirk missed by
the earlier selected placements: a finish increments the level during group
iteration, and subsequent checks use the hidden next-level bodies with their
stale transforms at the previous camera offset. Newly shown groups can retain
that offset for their first physics tick. Native now preserves those effects
and all 116 source-engine checkpoints pass with the corrected placements.
The earlier note about deterministic full-restart reuse is superseded by this
correction. Broader scene/effect/audio parity is still partial.

Validation of this correction: 48 Rust tests pass, including all 116 original
engine checkpoints and the live random-tape comparison across ten levels.
Refreshed initial native/original captures at 1440, 768 and 375 by 800 have mean
RGB errors of 0.025, 0.196 and 0.329 out of 255 respectively. These idle captures
check initial geometry only. An owned native window was exercised through
movement, pause/resume, full restart and Home without a crash.

## Elf Maker panel inset shadow correction (2026-10-24)

The reported faint shading at the main-panel/control-strip boundary was missing.
The original elfmaker.scss #main rule uses
`box-shadow: inset 0 -12px 12px -12px rgba(0, 0, 0, 0.125)`.
The native renderer previously drew only the background and elf. It now paints
the inset shadow after the background and before the children, matching CSS
stacking. No artwork edges, geometry or animation timings were changed.

elfmaker-panel-shadow.png is a transparent 96x96 raster of that exact original
CSS declaration, exported with Chromium at device scale 1. Its 36px nine-slice
edges preserve the blur rather than stretching it with the panel. A separate
original-CSS raster at 1440x662, 768x662 and 375x400 was compared to the assembled
native mask: every RGBA byte matched at all three sizes. This checks this shadow
only; it does not certify the entire scene or explain other image-edge issues.

Release build and owned-window captures passed at 1440x800 and 375x800.
Both were visually inspected, including the lower panel edge.

## Christmas tracking controller logic (2026-10-24)

Re-read the complete maputils.js (544 lines), modvil-tracker.js (393), stats.js
(142), feed.js (183), photos.js (296), photo.js (68), spherical.js, season.js,
tracker/stats Sass and the index.html config/route lifecycle. This correction
ports controller logic, not a map renderer. Earlier statements that the
marker/trail and camera *logic* are missing are superseded below; the actual
native tracking screen remains missing.

Source-to-native mapping:
- StopManager constructor/update -> tracker_control::Markers. Initially only the
  first marker is visible. Forward/backward time changes visibility; twenty
  geodesic trail endpoint pairs retain source opacities, and the active leg
  follows Santa between stops. Trail rebuild runs only when visibleTo changes;
  home clears trails on that change, as in the source. Marker icons and real
  Google Maps objects/click dispatch remain renderer work. destroy is native
  object ownership; the source leaves polylines attached, which cannot affect
  this renderer-independent model.
- Tracker updateNow -> ClockAdjustment plus Tracker::tick. Sampling is once per
  second, with an anchored monotonic cadence, source offset and jitter seconds
  conversion. The jitter ratio is supplied to the controller; browser/local
  storage persistence and remote config delivery are not implemented. The
  user-requested gate uses the real calendar clock, not a debug/jitter offset.
  Flight begins Dec 24 10:00 UTC, ends Dec 25 11:00 UTC exclusive, and route and
  marker state rebase on a new year. Off-season tick/frame return no frame.
- shouldUpdate -> route/marker update, feed stop IDs, destination/current stop,
  heading/stopped/home state, mobile gestures/zoom control and focus commands.
  Empty destinations retain the source home fallback but have no feed stop IDs.
  Route detail photos/attributions are preserved; remote media are not loaded.
- focusOnSanta, _onFocusSantaClick and map reset -> explicit focus/map_changed
  calls, twenty-second deadline, cancellation and temporary-destination reset.
  Repeated marker changes while already unfocused do not restart the timer.
  Programmatic camera/resize events must not be submitted as user map changes.
- _onMarkerClick -> marker_click, latest-stop refocus and selected destination.
  Source tests `this.width` instead of `_width`, making its mobile early-return
  ineffective; the controller preserves this quirk. Renderer hit-testing is
  responsible for supplying actual marker clicks.
- _setLocation -> Layout::camera, preserving min/max zoom 2..6, source rounding,
  home/stop/flight/selected-stop offsets, open-photo 100px subtraction, desktop
  40px heading allowance and vertical pan. The renderer must apply commands in
  source fitBounds/setZoom/panBy order. Resize updates gesture options; changing
  only width does not invent an immediate camera refocus.
- stats formatDuration/formatArrivalTime/formatNumber and render data -> Stats.
  Original English messages, half-hour thresholds, last-ten-minute arrival
  hiding, rounded km, comma-separated numbers and countdown fields are ported.
  Other-locale Intl formatting, icons and actual stats layout are not ported.
- prepareMaps/elementMapsOverlay remain map API/DOM integration work. No API key
  is repurposed and no substitute/fake map is added. Map event registration,
  tile readiness/1500ms race, overlay projection and native IP-location lookup
  are not provided by these renderer-independent commands.
- DestinationsCache/fetch HTTP lifecycle, listener race protection, persistence
  and live route replacement remain unfinished. The supplied 420-stop original
  schedule is still used, skewed to each year just as the source does. This is
  scheduled route simulation, not evidence of a newly fetched 2026 route.
- Feed ItemChooser/runner, remote media backoff/playback/transitions, and photos
  URL sizing, async cancellation/replacement, attribution UI and animations
  remain explicitly unfinished. Reading their code is not an implementation.

The development-only tracker-control-reference.cjs executes the original
StopManager and actual tracker prototype methods. Maps marker/polyline objects
and DOM nodes are spies recording commands; it does not test real map tiles,
projection or painting. A UTC calendar and explicit wall timers isolate source
state transitions. The original updateNow body is separately executed for 12
offset/jitter combinations. The fixture includes 3,020 marker/trail states
matching every existing route checkpoint (including reverse time), 18 controller
states (selection, invalid/latest markers, timeout, mobile resize and the source
mobile-click quirk), 23 formatting boundaries, number grouping and geographic/
timezone nearest-stop cases. All match the native implementation. Separate
native tests cover frame cadence, exact seasonal boundaries despite jitter,
next-year rebasing, empty-route fallback and preservation of photo metadata.

The tracker_reference example now outputs the full visibility-gated controller
state for a real or supplied clock and timezone offset. It is a runnable native
logic diagnostic, not a working map screen. Whole-site/scene ratings are unchanged.

Validation: all 53 tests pass. The release diagnostic was exercised at October,
takeoff, one hour into flight, the last flight second and the exclusive return
boundary; visibility and serialized controller output matched those cases.

## Gift Slingshot intrinsic image geometry and filtering (2026-10-24)

The user's softness comparison prompted a new controlled browser/native pass.
Re-read the full Boatload HTML, player/environment/boats/icebergs/presents/effects
Sass, source SVG dimensions and active player/reset/animate/scene-scale paths.
The native renderer stretched the 43.3x40.9 player-elf SVG to 44x44. Source CSS
instead centers that intrinsic image inside a 44x44 element. Native upload now
rasterizes the unchanged artwork into the correctly sized/centered box. boat1
similarly retains its intrinsic 88x193.3 art in the source 88x194 top-left box;
rounding its raster to 193 pixels had clipped its bottom and altered layout.
Logical sprite dimensions are kept independent of raster sampling density.

Boatload textures now remain premultiplied through interpolation and mipmap
construction, with ONE/ONE_MINUS_SRC_ALPHA blending at draw. This preserves
source alpha and avoids mixing straight RGB independently of coverage when
images are scaled down. Original SVG/PNG files, proportions, gameplay rules,
inputs and animation timing are untouched. Other scenes' texture policies
retain their previous behavior.

The browser harness runs the actual original game with the existing manual-time
hook, seeds before construction, advances 0.5s and freezes it. Each viewport is
a fresh page/game, avoiding random consumption differences from resizing an
existing game. The original CSS and functions are not patched. Native capture
now uses the actual production scene compositor, extracted unchanged from
overlay.rs into scene_filter.rs, instead of bypassing it. A byte comparison of
the compositor body confirms this extraction did not change its behavior. The
other overlay capture example was updated to import the same module.

Matching 1440/1280/980/900/768/375 by 800 captures show lower player/elf and full
frame mean RGB error at all six widths. Elf-region errors went from
9.807/9.078/9.079/11.876/8.541/11.441 to
6.780/6.437/6.438/8.181/5.494/8.009 out of 255 (29–36% reduction).
Full-frame errors went from 0.189/0.276/0.092/0.560/0.577/0.934 to
0.181/0.268/0.086/0.537/0.560/0.852. See boatload-raster-layout/errors.json.
Desktop original/old/new crops were visually inspected, including intrinsic
height, hands/hat and board edges. These are idle raster comparisons, not proof
of all animations or whole-mode parity. The earlier 0.282 comparison used a
different capture path; the newly measured before/after values are comparable
to each other, not a direct revision of that historical figure.

Experiments with 2x SVG rasterization and four-sample offscreen MSAA were also
compared. They improved selected regions but worsened other matched cases, so
neither change is retained. The window's MSAA does not apply automatically to
the current offscreen target; matching source analytic edge coverage remains
work. Do not describe this as a global sharpness fix or add blanket sharpening.

A release profile with an owned visible window recorded 120 frames after 30
warmup frames: mean interval 16.77ms, main-thread work 0.79ms, scene CPU 0.72ms,
no intervals over 25ms. This is a current measurement, not a before/after speed
comparison or input-to-photon latency measurement. Initial concurrent-window
profiling was discarded after a timeout; the final profile ran alone.

All 53 Rust tests pass. An owned release window was also exercised through
Space, vertical movement, pause/resume, restart and Home; captures were inspected.

## Gift Slingshot score and splash effect lifecycle (2026-10-24)

Re-read the complete game.js (519 lines), source hit/miss/animate_/freeze/
unfreeze/restart paths, utils.animWithClass and full effects/player/environment/
boats/icebergs/presents Sass/HTML. The previous score renderer used a linear
scale/fade, the wrong text baseline and no CSS transform origin. The splash
used continuous shrinking borders and the game clock, losing source layout
rounding and its continuation through pause. These are now translated in
boatload_effects.rs and consumed by the actual native renderer.

- Score: 400ms CSS default-eased scale 0.5 -> 1, transform origin 50% 70%,
  -20px/-0.7em margins, normal Open Sans 48px line-box geometry. Opacity remains
  1 through 200ms, then follows a separate eased fade to 0 over the last 200ms.
  Browser DOM probes measured width 84.375, height 65 and baseline 51 for "100";
  the native text layout uses the same line box and advance widths. The source
  browser resolved system Open Sans Extrabold. The bundled font has matching
  ascent/descent metrics; an older full system-font experiment did not improve
  raster agreement and was discarded. No font/art asset was changed.
- Splash: source 500ms transitions interpolate content width, border width and
  scale. Blink's CSS border layout floors widths to integer CSS pixels with a
  1px minimum while positive, and content width truncates to 1/64px. The changing
  outer box moves each ring's center/transform origin. All four source offsets,
  widths, borders and their resulting transforms are preserved. Rings now use
  an antialiased annulus rather than a 30-sided polygon approximation.
- Separate wall and animation clocks preserve source pause/game-over behavior:
  score animations pause with the frozen class, while splash transitions keep
  running. wall_step supplies both clocks before simulation; step/collision
  paths trigger the actual effects. Restart does not clear active effect state.
- Reusing .animate changes position/text without resetting its animation or
  transition. Completed splash transitions hide the parent and reverse the
  child styles underneath it. A new miss during that return follows CSS
  reversal shortening (the sampled 200ms return produced a 341.270253ms forward
  transition). The existing native Transition model supplies this behavior;
  end_time exposes the point at which the source transitionend callback acts.

The development browser tools call the actual source hitBoat/missedBoat and
use Web Animations pause/seek to sample the original unchanged keyframes and
transitions. The missed-present object is stubbed only for the effect-trigger
call; present physics is tested separately by the existing game oracle. Actual
browser transitionend delivery removes .animate and starts hidden return
transitions; lifecycle tests do not dispatch invented completion callbacks.
DOM probes and CDP font resolution are recorded in the fixtures. Source CSS
animation-play-state confirms pause, while transitions retain their 0.5s
wall-clock behavior. No browser code or CSS engine is shipped.

22 first-cycle phase samples at 1440x800 and 768x800 match score bounds/opacity
and all ring bounds/borders to 0.003 source pixels; eight additional source
lifecycle states check reuse, hidden return, reversal duration, pause and restart.
Native tests also check score completion and animation/wall-clock independence.
The production renderer/compositor was captured at every sampled phase and
original/native text and ring crops were visually inspected. Measured mean RGB
errors in the bounded effect regions are recorded in
boatload-effects-raster-errors.json: text 0..1.896 and splash 1.035..3.313 out of
255. These quantify remaining raster differences; they do not certify 1:1 art.
The unchanged idle scene renders byte-identically to the preceding committed
renderer, checking the shared shape shader's default (non-ring) behavior.

All 56 Rust tests pass, including the existing seven Boatload simulation
scenarios and the added CSS phase/lifecycle comparisons. Broad responsive/input,
rope/present edge coverage and audio-mixer parity remain unfinished.

## Native tracker Santa component preview (2026-10-24)

Read complete santa-santa.js (81 lines), santa-santa.scss (189 lines) and the
active common.preload.assets implementation; the actual component and source
CSS polyfill run in a development-only browser reference page. Browser code is
not shipped. The preview uses the original 24 directional layer SVGs, effects
SVG and both unchanged chimney GIFs. No provider key, map image pack, substitute
cartography or webview is embedded.

- `dirForHeading`: direct 22.5-degree offset/modulo/index translation, including
  negative headings, NaN fallback south and Infinity producing no directional
  background. The source's nearby "27.5" comment is wrong; use its formula.
- `urls`/preload: all 27 assets are loaded by SantaView once. Native constructor
  reports decode/load failures instead of the host preload-event protocol.
- `properties`/render: heading selects three backgrounds; stop selects GIF mode
  floor(abs(heading)%2). Invalid mode produces no chimney background. Correct
  layer order and middle-layer ::after falling artwork are preserved. Width
  breakpoint is 768px, independently of the map's 600px controls breakpoint.
- Sass: 85%-width centered SVG backgrounds; 68x84 / 102x126 host; fixed 95x83
  contain-sized GIF; distinct Santa/front/back bounce values; ease-in-out per
  keyframe segment, not across the entire six seconds. Source GIFs are 383x333,
  36 frames each, 80ms/frame, infinite loop, with transparency/disposal deltas.
  Native decoding composites those deltas before premultiplied GPU upload.
- Hidden `.sleigh` cancels all three CSS animation objects. Actual stop/show
  probing confirms new animation objects after leaving the stop. SleighClock
  resets that clock; changing heading alone does not. GIF decoding/frame timing
  is checked independently, but browser GIF hide/show phase sharing is not yet
  verified and must not be described as matched.
- Group shadows use the composited silhouette with original offsets/opacity;
  two-times group sampling reduced the measured desktop east-facing ROI error
  from 1.857 to 1.646/255 in the selected phase. Rasterization remains different
  from Chromium and is not certified 1:1. The same group path is used in the
  native preview, with premultiplied layer filtering to avoid black fringes.
- Unfinished: component button hit-testing/accessibility and its parent tracker
  click-to-refocus, integration with native map/layout/controller, Santa overlay
  position/visibility, asynchronous asset readiness, browser GIF display clock,
  broader phase/direction/raster checks. Existing route/controller tests cover
  state/commands; this new standalone preview does not wire them into the app.

tracker-santa-reference.json contains 66 actual browser animation samples plus
18 heading cases, at 767/768/1440 widths. tracker-santa-lifecycle.json records the
actual original Web Animation cancellation/new instances. The independent Pillow
oracle in tracker-chimney-reference.json hashes all 72 visible composited frames
(transparent RGB normalized); native tests also check every frame boundary and
loop wrap. tracker-santa-raster-errors.json records isolated browser/native crops
at four directions and three widths, one matched animation phase. This does not
measure full tracker map/UI fidelity or end-to-end input delay.

The native map/display and online route refresh remain unfinished. Google Map
Tiles API supports native renderers and source JSON styles, but live tiles have
not been connected or verified and no user-owned key is currently available.
The original website's key must not be repurposed. The public release continues
to describe Christmas tracking as engine/controller tested, map/UI missing.

Final isolated sprite checks: all 59 Rust tests pass (54 library, five app tests).
The twelve selected-phase native/source crops have mean RGB error
0.241–1.820/255 within the recorded ROI; both chimney previews were inspected.
The release preview builds and exits its own capture windows. The main app
continues to show the existing village; this change does not expose tracking
off season or claim a completed map.

## Code Lab and Code Boogie — native first pass (2026-10-24)

These scenes are now initially playable in Rust, not complete 1:1 ports. Read the
own-scene JS, HTML structure/entry handlers and Sass, using bounded original reads
and comment-free Babel rendering for the longer JS groups (no code statements
removed). Original inline SVG symbols/sprite paths are retained as assets. The
bundled third-party Blockly/Closure engines are NOT fully reviewed/ported: the
native graph/editor replaces them and full editor parity remains unfinished.
`coding-function-inventory.json` explicitly inventories 377 functions in the 42
own-scene JS files as partial/unfinished; it is not a completion certificate.
Source file hashes and reference-harness scope are in coding-source-provenance.

Native equivalents and outstanding branches:

- codelab constants/map/levels: original full map and all fourteen level records
  execute unchanged in the export VM; XML starting chains, tools, fields and
  editable/movable/deletable flags become typed Rust data. Map bounds, original
  7 usable columns vs 8 rendered columns, tree blocking and jump landing-only
  collision are preserved. Native renderer iterates rows then columns right to
  left, with original tile dimensions and SVG decorations. Pooled square/present
  objects, overlapping retained rows/cleanup, and 800ms world travel are missing.
- codelab level: `checkSuccess` becomes Workspace::puzzle_success (one stack,
  exact A/B/C order, no branches/extras). `isOutsideBounds`, `processResult` and
  LevelResult defaults become lab_run/evaluate/ResultInfo. Required blocks only
  inspect editable enabled user blocks, while counts include the nondeletable
  initial movement block. Editable, movable and deletable are distinct flags;
  source isEditable does not test isDeletable. Empty containers precede extra
  root/missing-block/too-few/too-many hints. All active levels have one present;
  collection checks preserve source per-action present matching, not a new
  cumulative multi-present rule. Broader result/disabled-block inputs remain.
- codelab player: move/jump use original cardinal rounding, bounds and tree
  checks, start-direction carry across levels and length+1 landing. Actual
  source animation objects demonstrate walk=500ms and jump=1000ms, plus 200ms
  for a turn. Walk includes the 100ms sound-effect sequence before its 400ms
  position/sprite tween. Do not infer total duration from its declared 400ms
  wrapper option: the bundled animation engine yields the measured totals.
  Source shortest rotation is used; jump translation ends after 400ms while its
  sprite continues. Native lose bump, pickup fades, drop shadow and exact
  rotation/sprite phase ordering still need source engine comparisons.
- codelab block_runner: generated-code eval replaced by a typed graph interpreter
  for directions, jumps and nested repeats. Movement stops on a collision or the
  original present condition; failed playback is 1.5x slower. 800ms pickup/loss
  effects and turn durations are queued. Injected loop highlights (including
  duration borrowing), native rewind state and all cancellation interleavings
  are unfinished. The 100,000-action native preview limit is an intentional
  responsiveness difference, not a claim about original execution limits.
- both Blockly wrappers: graph roots/next/body, load/reset level, when_run insert,
  count/user/required-block queries, empty containers and extra roots are native.
  All original generated block images and their SVG bounds are exported through
  the actual bundled Blockly renderer. Computed SVG presentation is inlined
  (especially fill:none/hidden highlight paths); intrinsic painted bounds retain
  the negative top notch and bottom shadow independently of connection height.
  Repeat containers stretch their source middle region; arbitrary nested layout,
  connection precedence/insertion, hover/focus, flyout/trash details, workspace
  panning, keyboard, touch and full field geometry are not matched. Header/field
  clicks provide repeat/jump dropdowns. Codelab freezes editing during playback;
  Code Boogie remains editable during a run, as its actual wrapper allows.
- both blockly_layout classes: source toolbar/responsive formulas and when_run
  offsets are translated in part. Full cardinal/row flyout dimensions, exact
  660/1024px breakpoints, short-height behavior, portrait sliding/timers/underlay
  and touch drag routing need matched browser captures. Mini-block icons and
  large block textures still differ in some rendered states.
- blocks/patterns: source XML builders and generated SVG art are reference-only;
  native graph execution does not run JS. Original repeat/jump options and source
  source images remain. Async pattern-def injection/caching is replaced by native
  texture caching. Code Boogie now preserves generated loop structure/names for supported
  enabled dance/repeat graphs (see follow-up below). Code Lab still lists
  expanded actions; neither scene has exact textarea layout/scrolling.
- codeboogie level/dance_level/levels/step: all nine teacher sequences, stage BPM,
  ideal counts, intro lengths, special steps and all three freestyle definitions
  are original exported data. evaluateStatus and createAnimationQueue are Rust,
  including wrong-step truncation, watch/fail positioning, extra-step/fail tails,
  success specials and countdown insertion. Source result ordering is partially
  translated; serialization/deserialization/share and mutable freestyle level
  bookkeeping remain unfinished. Empty freestyle produces no result overlay.
- codeboogie animation/animation_data/character: source frame counts, 24fps
  scaled by BPM, 24 frames/strip, 622x578 canvas offsets and strict `>` elapsed
  boundary now run in codeboogie_stage::SpriteClock, retaining millisecond
  arithmetic. Dancers are absent until their first source animation is prepared.
  Watch/show CSS transforms now use the original 480px parent, 85% transform
  origin, canvas baseline gap, 1s linear interpolation and watching scale/z order.
  Original PNG strips remain lazily loaded. Async missing-image retry/reload races
  and complete asset failure/disposal lifecycle still differ; no full asset-load
  lifecycle certification. Source paused=true bootstrap is immediately followed
  by Character.play -> Animation.play on active paths; no source pause method is
  introduced. getFrame uses retained AnimationData; native crops the original
  canvas bounds. Color-loading race behavior remains unfinished.
- codeboogie animation_player/sequencer: replaced the first simplified scheduler
  with codeboogie_stage::DanceStage. onBeat runs title cleanup/count increment,
  music-bar shift/start/tile/sound/highlight, next-bar preparation, count sound,
  then lighting in source order. Preserves current animation/isPlaying/watch
  states across interrupted starts, next-bar track variants and clock reset,
  one callback for the observed beat (no catchup loop), initial empty sprite
  state, end titles and finish behavior. The three active EventTarget listeners
  map to typed Track/Sound/Step/Finish events; generic arbitrary third-party
  listener registration is not a native public API. Full route teardown,
  rendering-clock/device and host/audio presentation checks remain incomplete.
- codeboogie lights/move_tiles/title: all constructors/methods/callbacks now have
  native equivalents in codeboogie_stage.rs and coding_view.rs. Fisher-Yates
  consumes eight draws per active floor beat and keeps the permuted object
  order across beats/levels. Four floor tiles are selected; ceiling sequence is
  left red, right green, left green, right red; white beams are idle-only.
  setLevel changes flags without erasing retained beam/tile state. Move cards
  use source 124px-at-10px-em stride logic, fixed 3px borders, min(4,length) width,
  first-add immediate motion, 200ms ease-in translation, 500ms fade/scale and
  source CSS counters/icons. The uncancelled 200ms remove-all callback can also
  delete newer cards. First two classes preserve cards on finish (fadeTiles=false).
  Title spans retain smallness when their text is unchanged, count values use
  numeric/string strict-comparison replacement, destroyed spans leave on the
  next beat, and appearance/disappearance follow the BPM CSS delays. Rust
  translates these rules; no original CSS/JS engine is shipped. Original SVG
  card icons now rasterize at 4x with mipmaps to avoid enlargement blur.
- game/scene/result in both scenes: native chooser, level progression, run/retry,
  continue, basic portrait tray, original result art and English hints exist.
  Exact result fades, graphic layout, missing-block picture, final host game-over,
  tutorial queues, share/import/export and all restart/route lifecycle quirks
  remain unfinished. Source codeboogie API restart handler is a no-op; native R
  also leaves the scene unchanged. Source no-op
  focus callbacks/disposal DOM listener removal have no native DOM equivalent.
- player_sound/klang: events use original sample configuration; walk/rewind stops
  correctly fade/stop rather than starting their source. Six Code Boogie tracks
  use the original AudioGroup ordering. cb_start/end stop actions are preserved.
  Full PlayerSound one-shot guards, precise walk/rotation/pickup scheduling,
  multi-source cb_fail and performance-bus ducking are unfinished. Native tests
  are muted; full listening/device/mixer validation is still required.
- monkeypatches/i18n: browser vendor-prefix patch has no native DOM use; English
  source messages are exported as data. Other languages/accessibility incomplete.

Evidence: 16,128 actual Chromium Player.move/jump cases span every maze cell,
all four directions and jump lengths 1..3; native position/collision/direction/
activeDuration comparisons pass. No movement/animation engine is stubbed in this
browser fixture. Starts are deliberately controlled per case. Both jigsaws and
native completion paths for every maze are tested, plus graph cycles/repeats.
54 unchanged source dance status/queue cases cover all nine classes. That VM
supplies namespaces/inheritance/localized strings; it does not exercise Blockly
or the browser dance renderer and must not be described as doing so.

Main integration includes CLI/routes, Home, scene input enablement, source level
counts 14/9 and original shared recommendation selection with the expanded native
pool. A first owned UI smoke found the missing input-eligibility branch; fixed
before publishing. Earlier synthetic-pointer captures that did not actually edit
anything were discarded, not counted as successful gameplay verification.

Follow-up rendering checks corrected two concrete geometry errors: the source
world origin is 0.3 tiles below its fixed 12.6-row viewport origin, even when
width-driven tiles make it taller than the window. Native now uses that source
formula; five settled source DOM cases check scene bounds, toolbar widths,
font scale and player origin to 0.03px (CSS layout rounding). Stage backgrounds
and map contents are clipped to the scene, preserving the editor alongside them.
Original mini-block art/painted bounds and trash SVG now replace the first draft
mini/button artwork. The source touch nub is visible on movable non-mini blocks;
it is retained, while when_run is exported nonmovable. XML nonbreaking spaces
are escaped numerically, and relative SVG images are embedded as original bytes.
Native asset tests check every SVG root and reject unresolved image paths.

Owned release UI smoke `/tmp/coding-ui-s36qf370` completed both jigsaws, first
maze, first teacher dance and freestyle entry using actual mouse/key events;
result captures were inspected. Its capture exposed the world-origin and stage
clipping differences above, which were corrected afterward. Earlier SVG-export
failure on the first maze was corrected before publishing. These are interaction
checks, not a pixel-perfect comparison or validation of all twelve/nine levels.


Code Boogie chooser correction (user screenshot feedback): translated the full
shared overlay and scene chooser layout rules, preserving the source's unequal
artwork dimensions and its `choose-mode-info` normal-flow caption class. The
chooser now uses #3f49a3, Lobster title with -2px letter spacing and 6px shadow,
Roboto ExtraBold option captions and regular descriptions, the 680/1025px
breakpoints, shared 500ms background/700ms dialog entrance, 200ms artwork hover
and delayed 400ms Go transitions. Four original Chromium DOM captures compare
all title/option/artwork/label/description boxes within 0.05px. Fixture:
`data/codeboogie-chooser-reference.json`; test in `coding.rs`. Native uses font
and SVG rendering only; no HTML/CSS runtime. Removed the invented Space-to-run
shortcut and Boogie R reset, retaining the original orange Run control and
no-op restart behavior. Chooser font rasterization/hover reversal timing and
other scene overlays still require broader phase/device comparisons; this
correction does not certify whole-mode parity.

Chooser texture sampling now uses higher-resolution original SVG rasterization
with mipmaps when reduced, retaining the source teacher viewBox aspect ratio
397:371.2 inside its square background box. Code Lab's portrait callback is now
one-shot, preserving older scheduled callbacks across level changes rather than
forcing the tray closed every frame after three seconds. Both coding scenes now
honor the existing --advance render diagnostic so idle/entrance captures can be
made at a controlled phase; this is a development capture control.

Final owned native fixed-time release captures and original idle captures cover
1440x800, 1846x1014, 768x800 and 375x800. Below the independent host chrome
(y >= 200), mean RGB errors are recorded in
`data/codeboogie-chooser-raster-errors.json`; these idle comparisons do not
validate active dances, editor interactions, sound or all transition phases.
The native window manager expanded the requested 1846x966 drawable to
1846x1014, so the original was recaptured at the actual size before comparison.
The pure layout test additionally checks the requested 1846x966 DOM case.
The user's README rating for Code Boogie is Very similar; agent status remains
partial with remaining gaps listed above.


## Code Boogie stage playback and effects follow-up

Re-read complete lights.js, move_tiles.js, title.js, sequencer.js,
animation_player.js, animation.js, character.js, scene.js, game.js,
block_runner.js, dance_level.js, level.js, levels.js, step.js, constants.js,
klang.js and animation_data.js for this group, plus the complete scene SCSS and
structural HTML. Inline SVG path/style data are retained licensed assets rather
than rewritten geometry. Earlier own-scene review and third-party engine limits
remain as recorded above; the entire Blockly/Closure engine is not claimed ported.

Evidence in rust/data (relative to rust/ below):

- codeboogie-stage-reference.json: 15 scenarios / 2,775 snapshots from real,
  unchanged source AnimationPlayer/Sequencer/Title/MoveTiles/Lights/Animation
  constructors in Chromium. All nine intros run through completion, with success,
  wrong-step, freestyle/empty, interruption, skipped beats, and cross-stage/track
  changes. External performance.now/rAF/timer dispatch and a 32-bit LCG random
  stream are controlled; source scene methods are not replaced. Event output is
  recorded instead of sent to an audio device. Snapshots are cloned immediately
  to prevent later source array mutations from rewriting earlier evidence.
- codeboogie-stage-layout-reference.json: 12 settled states across six original
  DOM viewports (watching/normal). Character canvases, ceiling boxes and move-card
  boxes match native formulas to 0.055px including CSS 1/64px layout rounding.
- codeboogie-stage-phase-reference.json: 59 actual Chromium CSS Animation and
  Transition samples for title/card fade/scale, carousel shift, beam opacity and
  player/teacher transforms. Native getAnimations is saved before the original
  Web Animations polyfill overrides its wrapper; querying that wrapper alone
  missed native CSS effects. Animation currentTime is controlled; source easing,
  keyframes and layout are unchanged. Matrix string serialization limits x/y
  comparisons to 0.001px, with opacity/scale tolerance 0.00002.
- codeboogie-stage-raster-errors.json: ten original/native settled scene images,
  idle sprite frame zero, five actual viewport sizes. The stage crop omits 80px
  at the left edge to exclude the separate Run button; editor/host UI are not
  compared. Mean RGB errors range about 0.96–1.43/255. These are selected stills,
  not certification of every active image, light reversal, sound or input path.

Development harnesses: tools/codeboogie-stage-reference.cjs,
tools/codeboogie-stage-layout.cjs and tools/codeboogie-stage-phases.cjs (reference
checkout only). Native example: examples/codeboogie_stage_capture.rs shares the
actual model/renderer and exposes controlled comparison state, not extra game
controls. Large windows were resized by the WM to 1846x1014; original references
use that actual drawable size. Source `start(..., removeGuiNow=false)` is used to
exercise the real chooser close path before CSS captures; setting only hidden
on a display:block chooser did not remove it from the reference image.

Owned live mouse/F12 smoke completed a connected two-block first-lesson program,
ran the routine and reached Nice moves; graph and result images were inspected.
A first direct-input attempt never placed blocks and is not counted. The successful
run acknowledges rendered frames between down/move/up; unattended/occluded input
can otherwise coalesce. Scene CPU/frame pacing and audio were not certified by
this muted smoke. Source-style skipped-beat behavior is intentionally retained.
Remaining substantial work includes full Blockly interactions/layout, tutorials,
share/import/export, exact results/final host game-over, responsive tray transitions,
chooser exit/route lifecycle and complete audio scheduling/bus/listening validation.

## Code Boogie educational code follow-up

Read the complete original `codeboogie/js/result.js`, `blockly.js`, `blocks.js`,
`game.js`, `scene.js`, `block_runner.js`, result markup and `_result.scss`, shared
`_button.scss` and `_no-en.scss` before this work. Read the complete bundled
Blockly `core/generator.js`, `core/names.js`, `generators/javascript.js`, and the
active `Workspace.getTopBlocks` ordering path. This is a named generator-path
review, not a claim that the whole third-party editor has been ported.

- Native `coding_user_code.rs` translates the supported enabled dance/repeat/
  when_run graph's educational text. It retains repeats, two-space indentation,
  child-before-parent loop-name allocation, count→i/count2→j replacements,
  count3+ names, and the stable three-degree LTR root scan. Numeric block-ID and
  loop-highlight statements are absent by construction in this display text.
  The native app does not execute generated JavaScript.
- `codeboogie-user-code-reference.json` contains 98 actual original Blockly
  outputs from Chromium: all seven steps, counts 2–6, empty graphs/loops,
  sibling/nested loops through depth six, root scan bias/ties and 80 seeded
  generated graphs. Real XML graph loading and the original generator/Names/
  getUserCode run unchanged; the harness only exposes the compiled namespace
  and holds external scene rAF. Native output matches every full string.
- `CodingView::run` snapshots code before playback, preserving it while blocks
  remain editable. The original failure DOM has no View code link; native
  Boogie now shows it only for a successful result with nonempty code. Showing
  code is one-way until the next result, as original onShowCode specifies.
- Unsupported block types report an explicit error. Disabled blocks, comments,
  arbitrary imported graph types and RTL are not represented by the current
  native editor. Generator equivalence is limited to the supported graph scope.
- Exact result flex layout, delayed fade/slide/reversal, missing-block artwork,
  textarea font/label/scroll/selection, share/import/export and final shared
  game-over are still unfinished. The current text renderer clips the preview
  to twelve lines; this is not whole-result-screen parity.

The follow-up UI automation did not reliably deliver its planned chooser/drag
inputs: one capture had an unattached block and the second stayed at the
chooser. Both runs were discarded as evidence of the code-display flow. The
execution-time snapshot is implemented, but this particular edit-during-playback
UI path remains interactively unverified. The earlier successful first-lesson
stage smoke above is separate and still valid. Full 71-test suites pass in both
checkouts, and the updated native release binary builds successfully.
