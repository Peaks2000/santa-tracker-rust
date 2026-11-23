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
  Code Boogie result flex layout, child transitions, missing-repeat symbol and
  code pane now have native equivalents (see result follow-up below). Code Lab
  result layout, complete host presentation/lifecycle, tutorial queues, share/import/export and
  all restart/route lifecycle quirks remain unfinished. Source codeboogie API restart handler is a no-op; native R
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
  At that pass Boogie showed it only for successful nonempty code; the later
  result follow-up corrects the source CSS hidden-link quirk. Showing
  code is one-way until the next result, as original onShowCode specifies.
- Unsupported block types report an explicit error. Disabled blocks, comments,
  arbitrary imported graph types and RTL are not represented by the current
  native editor. Generator equivalence is limited to the supported graph scope.
- At this educational-text pass, result layout/transitions and the twelve-line
  preview were still incomplete. The following result pass supersedes those
  portions; text selection, exact soft wrap/scrollbar/resize, sharing and final
  host game-over remain unfinished.

The follow-up UI automation did not reliably deliver its planned chooser/drag
inputs: one capture had an unattached block and the second stayed at the
chooser. Both runs were discarded as evidence of the code-display flow. The
execution-time snapshot is implemented, but this particular edit-during-playback
UI path remains interactively unverified. The earlier successful first-lesson
stage smoke above is separate and still valid. Full 71-test suites pass in both
checkouts, and the updated native release binary builds successfully.

## Code Boogie result panels follow-up

The selected original `result.js`, `_result.scss`, complete shared `_button.scss`
and `_no-en.scss`, result HTML structure and active game/scene/block-runner/
LevelResult paths were read before implementation. This is a native translation,
not a CSS/HTML engine. Additional entrypoint/host overlay paths were inspected
but final host game-over/sharing are explicitly still unfinished.

| Original behavior | Native equivalent / limits |
| --- | --- |
| constructor / bindEvents_ | `Spec`, two independent `Panel` states and native pointer capture. Keyboard focus/activation/accessibility are unfinished. |
| show | Copies verdict, localized hint, retry/freestyle/final labels, missing-block and submitted-code snapshot; resets show-code class. Native live specs still omit shareUrl because host sharing is not implemented. Exported Boogie levels currently have empty requiredBlocks; the optional missing-repeat display is exercised by the diagnostic contract cases. |
| hide | Root visibility/opacity changes immediately, preserving child clocks/state for later show. Native initially assumed a parent fade, but actual Chromium disproved it. `transition: opacity 0.3s, visibility 0 0.3s` is rejected in full because of the unitless time. Computed transition is `all` with zero duration. This source quirk is preserved. |
| onRetry | Hides result and clears pending report state without clearing/editing blocks or resetting the still-running player. Broader overlapping controller/input paths still need comparison. |
| onContinue | Advances class; continuing freestyle restarts teacher mode directly, matching Game.bumpLevel rather than returning to the chooser. The following host-completion pass replaces the placeholder with the shared game-over overlay; complete presentation/lifecycle matching remains unfinished. |
| onShare | Unfinished. Source-sized share button is measured in diagnostic layouts but not supplied by live native result specs. No working-share claim. |
| onShowCode | One-way code-view class until next show; immediate flex relayout, source label/border/padding/line height. The source a[hidden] still displays due to its inline-block CSS, so empty-code success retains a clickable View code row. Native preserves that quirk. |
| _result.scss layout | Centered column flex, title 70/140px, details 20/30px, fixed margins, text min-content limits, SVG shrink factors 1/4, textarea padding minimum, width 570 / height 500 breakpoint, button 768 breakpoint and wrapped inline button rows. Rustybuzz shapes original retained font data; no browser runtime. |
| child transitions | Title opacity/translate, details/missing/art fade after 400ms, buttons subtree after 800ms; positive delays, shortened reversals and hidden retained phases use Transition::set_delayed. Buttons subtree composites once through a premultiplied offscreen target. |
| button appearance | Original purple face/dark extrusion/black shadow, rectangular radii, font and 50ms press/250ms release formulas. Full pressed/focus/hover/device image-phase coverage is unfinished. |

**Evidence and limits**

- `codeboogie-result-layout-reference.json`: 70 original Result.show/onShowCode
  display configurations at seven sizes (1440×800, 1024×800, 768×800, 570×500,
  569×499, 375×800, 800×375). Includes success/retry/failure/missing hints, final,
  empty code, freestyle/share row and code view. Native boxes match within
  0.08px. These are English default-font configurations, not all translations.
- `codeboogie-result-phase-reference.json`: 96 actual Chromium opacity/transform/
  visibility samples, including show/hide and interruption at five phases in
  both directions. Native scalar differences are below 2e-5 and translation
  differences below 1e-4px. The pre-polyfill native animation APIs control only
  external animation currentTime; original Result/CSS methods are unchanged.
- `codeboogie-result-raster-errors.json`: eight settled full-panel captures at
  three actual sizes plus four entrance phases, over the same isolated flat
  background. Mean RGB errors range about 0.046–0.875/255; per-component region
  errors are also recorded. Visually inspected original/native images and 8×
  difference images. Desktop title/art/buttons/code pane closely align; small
  raster/font/primitive-edge differences remain, especially at small sizes.
  Isolating the panel does not verify the editor/stage behind it or host chrome.
- Restored the original missing-repeat symbol as a separate native SVG asset,
  including embedded original icon bytes. It is not the editor's full repeat
  block artwork. Result geometry tests alone did not catch its first asset-route
  mistake; the owned native capture did, and the route was corrected before
  publication.
- Chromium CSS.getPlatformFontsForNode identifies the original Linux generic
  code font as NotoSansMono-Regular. Retained the same installed Noto Sans Mono
  font and OFL license, matching selected code-pane captures. This does not
  certify other platforms' browser fallback fonts.
- `codeboogie_result_capture` uses the live renderer. Its `flow` diagnostic runs
  the actual native intro, creates the first source program, invokes Run, adds
  a third block during playback and reaches the successful result. Assertions
  verify the submitted two-block code/verdict survive the edit; the final code
  pane was inspected. This is input-independent controller/render evidence,
  not a successful synthetic drag/click/keyboard test.
- Code Boogie no longer has the old twelve-line preview limit: its native code
  pane clips and scrolls. Precise textarea soft-wrap choice, scrollbars, dragging
  its resize handle, text selection/copy and keyboard accessibility remain
  unfinished. Code Lab still uses the older result renderer. Neither whole mode
  nor all result interactions are certified 1:1.

Final verification for this result pass: all 73 tests pass in both checkouts;
the updated native release binary and result-capture example build successfully.
The README user rating remains Very similar; full site parity is unfinished.

## Coding-scene completion and host restart

Read in full for this path: both scene `js/game.js` files, the Code Boogie result
callback/scene and both HTML API handlers; shared `scene/api.js`, `global.js`,
`santa-gameloader.js`/SCSS, `iframe-load/main.js`, `lib/iframe.js`, `lib/channel.js`,
CSS polyfill, shared overlay JS/SCSS and santa-button JS/SCSS. Read the complete
active entrypoint state subscriber, restart listeners and incoming-message runner.
The third-party Unistore/DOM implementations execute in the browser oracle; this
is not a claim of having ported/reviewed their entire engines. Iframe/network/
analytics boot has no native DOM counterpart and is outside this completion pass.

- `coding_host.rs` preserves level number, retained editor level, missing current
  game level after completion, last score/max level and game-over state. Teacher
  Code Boogie reports stop level 9, while Code Lab reports 15 after its fourteen
  levels; the original host ignores this payload and retains the last score.
  A subsequent Continue after Boogie's no-op host restart reports 10, not 9.
- Both native scenes now render the last scene behind the production shared
  game-over overlay. Removed the invented Great / Play again completion page.
  The header offers restart on game-over even for scenes without pause, matching
  the unconditional game-over branch in the original entrypoint.
- Code Lab handles host restart by resetting to the first jigsaw. Code Boogie's
  original HTML restart handler does nothing: the host hides its overlay and
  re-enables the retained last scene/editor. Native preserves this, including
  the player queue/clock, instead of returning to the chooser. These restart
  paths no longer stop all native audio; full mixer/listening parity remains
  unverified. Removed Code Lab's invented R restart shortcut; coding restarts
  use the original host controls.
- `SceneFilter::finish_live` refreshes the disabled coding scene after the dim
  transition settles. The original loader disables input and applies CSS but
  does not stop scene rAF/music. The existing cached disabled filter would have
  frozen Code Boogie's dancers/lights. Other scene filter paths are unchanged.

**Checks**

- `coding-host-reference.json`: 29 captured original states across both scenes,
  every level continuation, final completion, actual host restart-button dispatch
  and another continuation. Real Game/Result classes, scene API MessageChannel,
  global Unistore and SantaGameLoader/Overlay/Chrome run in Chromium. The original
  entrypoint subscriber/runner closures are executed unchanged, with external
  Firebase/router/analytics/audio/tutorial boot scaffolded. The scene rAF is held
  externally and Continue callbacks are invoked directly: this is not a complete
  puzzle/dance playthrough or pointer-input verification.
- Native completion/restart fields and the header action match every captured
  state. All 75 tests pass (69 library, six binary); native release and the new
  diagnostic build successfully. The usual existing overlay_reference unused
  re-export warning remains; no compiler errors.
- `coding_completion_capture` uses the live view, shared filter, overlay and
  header. It asserts last-scene retention, Code Lab reset, and Boogie editor/
  queue/clock preservation, then captures both completed scenes. Its controlled
  1/60-second scene clock continues under game-over. A far-right scene crop changes
  between fully dimmed frames 90 and 120 in Boogie (mean RGB change 7.22/255),
  proving this capture is not a cached frozen image. Lab's idle crop is static;
  this does not independently certify its other animated effects.
- Original and native completion captures were visually inspected. The shared
  hierarchy/art/cards are present; the original browser's auto-focus ring and
  heavier title glyph raster are still visibly different. Backgrounds use
  different controlled animation phases, so no whole-image error/certification
  is claimed. Shared blur approximation, keyboard/focus, microtask/frame input
  order, responsive route/orientation changes and audio still need matching.
- The reference harness must import the original CSS/attribute polyfills before
  creating components. Await Lit updates, force real CSS styles and wait for
  opacity=1 / blur(4px) before a settled capture. Merely observing hidden=false
  or calling document.getAnimations().finish() initially left the overlay at
  opacity zero; those premature images were discarded.

Freestyle/custom import/share and complete host boot/error/loading lifecycle are
still partial or unimplemented. This pass verifies the selected teacher/Lab
completion path, not every mode or the full site.

## Elf Jetpack level-up gradient flash

User reported a slight flash while going up a level. Read the complete original
`js/background.js`, `sass/_background.scss`, scene HTML and scene SCSS, plus the
active game frame/updateLevel ordering. Original background is 200% of the
viewport: solid for its first half, then continuously fading to transparent.
The declared JS BACKGROUND_HEIGHT=150 is unused; the actual stylesheet is 200%.
Constructor/reset/endTransition swap/preparation, five-second progress and strict
`> 1` completion remain represented by the existing native model. This pass does
not change simulation/level timing or certify the rest of Jetpack.

The native sky used 1.1px-high strips on a 1px stride. At some fractional vertical
translations every screen pixel in the fade received two overlapping draws.
The pulse reproduced at translation fraction 0.45: a middle pixel in the late
purple/blue transition jumped from red 43 to 23 while the original stayed at 42.
This was a rendering overdraw error, not an original level-transition effect.

`JetpackView::draw_sky` now uses one continuous mesh with a solid half and a
linearly fading half sharing edges. Each covered pixel blends once, matching the
source CSS structure. Level clouds/number artwork and other scene presentation
remain outside this isolated gradient correction.

- `jetpack-background-reference.json`: 12 actual original Background/DOM/CSS
  captures at 1440×800, spanning three colour pairs and four adjacent subpixel
  translations. Source reset/transition/onFrame run unchanged. External scene
  rAF is held and unrelated objects/::after art are hidden solely to isolate
  the gradient. Original RGB column bytes are retained as hex for native checks.
- `jetpack_background_capture` exercises the actual production sky renderer on
  the GPU, comparing all 800 rows / three channels / 12 frames with original
  bytes (28,800 comparisons, max difference 2/255). Its raw GL readback is mapped
  from bottom-up to the source PNG's top-down rows. The old renderer exceeds
  this tolerance by about 20 channels at the reproduced flashing phase.
- `jetpack-background-raster-errors.json`: whole-image mean RGB error at the
  flashing phase falls from 4.00–6.73/255 to 0.44–0.52/255. Neighboring native
  middle-pixel colours now stay stable. Original/native and before/after images
  were inspected. Small gradient quantization differences remain; these are
  selected gradient cases, not whole-mode/device/transition-art certification.
- All 75 existing tests pass and the updated release binary builds successfully.
  The GPU comparison also passes. User approval remains unreviewed.

## Snowball Storm — initial native implementation (27 October 2026)

Target remains 1:1; this pass is **Very similar**, not certified 1:1. All own-scene JavaScript, entry HTML and SCSS were read in full before the corresponding translation. `data/snowball-function-inventory.json` accounts for each file/function span and distinguishes checked formulas from partial implementation. It does not certify the bundled third-party THREE engine.

| Original source | Native equivalent / measured scope | Remaining scope |
| --- | --- | --- |
| `index.js`, `index.html`, `index.scss` | `snowball.rs`, `snowball_view.rs`: original splash assets, sleigh bob, pressed button, five-second cloud/elf intro, one-second retained splash, local match/restart. Weather has no `active` attribute: its cloud-frac/clouds/directional-strength attributes are unused by the actual component. | Responsive intro phases, keyboard/accessibility and route teardown need complete comparisons. |
| `snowball-game.js`; engine `Game`, `Gamelike`, `ClockSystem`, render/input systems; Main/Local levels | Separate `begin_frame` and `finish_frame` retain Level/camera/erosion → render → input → simulation ordering, one update per rendered frame, wall-derived 60Hz tick, strict 16/625/128 tick gates, 100-player cap, shared host game-over and restart. | Async asset readiness, exact initialization/random consumption, persistent collision registration, input octree sorting and full host lifecycle are incomplete. Source win repeats stop notifications; native stores game-over state. |
| `game-map.js`, `MagicHexGrid`, `HexGrid`, `HexCoord`, `SeedRandom`, priorityqueue | `snowball.rs`: source grid/coordinate formulas, seeded island/trees/raised frontier, independent erosion seed, rings, habitat/nearby selection, source greedy path cost and queue tie ordering, waypoint compression. | Unused general conversions/network serialization not ported. The nearby selection's original origin-vs-candidate test is retained. |
| State/Client/Player/Bot systems; Elf; all component classes | `snowball.rs`: arrival queues, per-render-frame 2.25 movement / 0.75 bot speed, click-to-move and blocked-tile/elf targeting, quantities/swapping fallback, spawn/population cadence, camera tether/damping, last valid host time/count, death/knockout. | Bot random order and pooled object reuse are not matched end to end; source late queues, offscreen LOD and interrupted action state need work. Registration is now persistent and contact-handler order is checked below; full landing/freezing/pooling cadence still needs comparisons. |
| Collision2DSystem, QuadTree, collision shape helpers, Tree | `snowball_collision.rs`, `snowball.rs`: persistent native quad partition/candidate order, Set registration order, directed pair notifications/asymmetric measurement cache, tree and shot/shot contacts, original circle containment quirk and shared collision-limit camera lag/reset. | Full spawn/landing/freezing/restart cadence, UUID/object pool reuse and generic subscription/debug methods remain partial. Trees are absent from the local dynamic rebuild but still query moving objects from the outer collidables loop; the earlier inactive-tree conclusion was incorrect. |
| `models.js`, Elf texture and animation methods, actual GLTFLoader/Skeleton path | `snowball_model.rs`: original glTF/bin, skin weights/joints/inverse bind transforms, source linear/spherical interpolation, vertex colours, idle/run/throw/three falls. `snowball_view.rs`: original generated colour bands, cache key, source material/light parameters. | Complete THREE action interruption, thrown-action crossfades, fall callback timing, model loading/LOD phase and pooled material reuse remain partial. Selected pose position tests do not certify normals or every action combination. |
| HexMap/Obstacles and shader partials | Source cylinder vertex/index fixture, 144-degree gimbal and actual camera orientation, original atlas/obstacle tilt, per-fragment tile tones/alpha, shaking/sinking/raised geometry and continuous meshes. | Large viewports, all erosion phases and transparent object ordering still need coverage. GPU batching avoids Macroquad geometry clamping. |
| Snowball/BigSnowball, SnowballSystem, Trajectory, Trail/Splat and effects | Source 256px/42-frame distance-dependent shot, arc, ignored skew preserved, large radius/quantity, snowball rotation/colour shader, source trail/splat equations and textures. | Source preallocated particle-ring/random consumption, overlapping effects and collision/exhaustion removal ordering remain partial. |
| Drop/DropSystem, Parachute/ParachuteSystem, Iceberg/EntityRemovalSystem | Original present geometry/colour bands/scales, stored 300-tick/420px canopy state, source carry/arrival/reparent queues and parent rotation offset, rocking parachute asset, pickups/300ms spin, 1500ms hold + 700ms teleport, frozen drift/bob/additive ice rendering. | Normal carry/landing parent matrices and system-stage offsets are checked below. Freezing/teleport stages and selected parent overlaps are checked in the later removal follow-up below. Full actor/model reuse, asynchronous LOD, general teardown/restart and all overlapping removals remain partial. |
| Powerup UI/status, population UI | Original art, 128px frames and quantity icon transforms, 300ms transitions and 768px scaling. Population panel remains hidden, swapping click remains disabled, per source. | Phase/reversal/quantity geometry not yet measured across devices; scissor rounding can differ. |
| NetworkSystem, Socket, messages, Network/Lobby levels/status | Reviewed, inactive in the actual entrypoint, which explicitly starts `local`. | No invented online service or multiplayer route is shipped. |
| Audio calls and host chrome | Original existing Klang sample/group resolver supplies `snowball_throw`, hit, pickup, frozen, button, generic music. Home (not menu), no pause and no invented keyboard controls; gameplay continues behind disabled game-over. | Listening checks, exact group/retrigger/mixer/bus timing and shared overlay focus/blur are not certified. |

Evidence:

- `snowball-reference.json`: actual unchanged modules in Chromium, external RAF held and `performance.now` controlled; four seeds, twenty complete 4096-tile snapshots through 249 erosion steps, coordinate/neighbour/ring samples, six paths/waypoints, twenty trajectory cases, nine powerup steps and forty actual quad-tree queries.
- `snowball-skin-reference.json`: actual source Model + bundled AnimationMixer/GLTFLoader/Skeleton, six clips at five times. Native tests compare all 2746 vertex positions (247,140 axis values; tolerance 0.00003). Harness uses looped `Model.play` for pose isolation; native gameplay uses one-shot falls separately.
- `snowball-render-reference.json` / `assets/snowball/reference-textures`: original 1440×800 render state; local landing tile set to 2080 to keep actors in the sample, map seed 1, source simulation stepped explicitly. Native diagnostic imports the measured state/textures to isolate renderer geometry. It omits source presents/projectiles from this isolated comparison and resets animation phase, so this is **not** a complete seeded match or gameplay equivalence test. The initial original capture taken before model readiness was discarded.
- Owned native mouse check entered the five-second intro, arrived and visibly followed a clicked path. An isolated `reference-pointer` check then held the measured source state still and verified that clicking actor 4 at (1248,278) creates a local shot and emits `snowball_throw`. Earlier moving-target misses/stale-state captures are discarded; moving-target/touch behavior is not certified. Native simulation also advanced 150 seconds without panic. These are smoke checks, not full input/lifecycle certification.

Development exporters stay in `rust/tools/` in the original checkout only. The app and distributed verification code use native Rust, original assets and data fixtures; no browser engine or JavaScript is shipped.

The isolated 1440×800 image mean RGB error is recorded in `snowball-raster-errors.json` (about 0.358/255). Its omissions/phase reset are explicit: this does not establish whole-mode visual equivalence. `snowball-input-smoke.json` records the limited owned input checks.

### Snowball Storm loaded-model animation follow-up — 22 November 2026

`src/snowball_animation.rs` replaces the previous elapsed-time throw overlay and
hard-coded fall-to-idle deadlines. Each player retains the original active
Model/THREE action lifecycle: cached loop/clamp/time-scale fields; same-name play
no-op; stop-all resets; positive/zero-delta timing; per-action weight/fade state;
mutable action-cache iteration; finished listeners retained across cancellation;
repeated throws; fall callbacks with their original unfiltered event behavior;
property accumulation and original-pose fallback. Fade control times/results
retain the source Float32 rounding. The scene uses linear translation/quaternion
tracks; unused engine scheduling/warping/ping-pong/cubic paths are not claimed.

The production game queues throw animation after idle/run selection and updates
models during parachute arrival as well as after landing. The renderer consumes
the resulting persistent pose. It no longer recomputes a heuristic throw pose
from a timestamp. The original model-root scale of five is now applied inside
the skeleton hierarchy, preserving its root translation; the previous outer
scale moved that translation too. The player marker remains attached to the
unscaled dolly as in the source.

Evidence is `snowball-animation-reference.json` and its provenance file:

- Fifteen original Model sequences / 650 states, including four longer stress
  sequences. The unchanged original Model/Elf callbacks and bundled THREE mixer
  run in Chromium; only external RAF/time and the requested fall variant are
  controlled. Native comparisons include all node TRS poses, action time/weights,
  flags, cache order, fades, listeners and finished events (tolerance 1e-9).
- Three actual ready-model `Elf.update` sequences / 42 states check the native
  game's idle/run/throw/die call order and movement. These explicitly import the
  original incoming transform because `Elf.onAllocated` retains pooled position
  and facing. LOD/input registration and collision removal are scaffolded in this
  isolated source check; neither those systems nor full pool behavior is certified.
- Fourteen sampled vertices and normalized skin directions in all 650 states
  check the actual source skeleton palette after root scaling. Native position
  tolerance is 0.0002 model units; direction tolerance is 0.00003. The existing
  thirty full-vertex isolated pose cases remain passing through the production
  pose-to-mesh path. These checks do not certify all GPU pixels/material phases.
- `snowball_capture ... action-4` (or another state index from the first sequence)
  renders the real native pose/mesh path in the measured original render layout.
  One such throw capture was inspected; no complete gameplay raster claim follows.

The original Elf pool also retains its cached Model across reuse; full native
pool/collision lifecycle and initial asynchronous model/LOD timing remain open.
The scene rating remains Very similar and user feedback remains unreviewed.


### Snowball Storm tree collision and shot queue correction — 22 November 2026

The earlier claim that local static trees could not hit projectiles was wrong.
`Collision2DSystem.update` rebuilds its quad with dynamic objects, but iterates
**all** registered colliders as primary objects. Each tree can therefore query
and notify nearby snowballs. Native tree colliders now use the source radius 14,
grid offset -32 + 15, tile registration order, strict circle intersection and
original limit containment. Eroded tiles do not unregister their trees. Multiple
tree contacts notify before removal, including shots marked hit earlier in the
same frame. The shared collision/LOD limit now uses the previous frame's camera;
measurement/resize recreates it at the origin, as in MainLevel.measure.

`SnowballSystem` uses newSnowballs.pop(), then iterates an active array whose
splice removal still increments the loop index. The native implementation now
sets up queued shots in reverse throw order and skips the shifted successor
after either collision removal or trajectory expiry. Setup and first trajectory
update are tracked separately: a skipped freshly set-up snowball is already
collidable even while its tickWhenThrown remains -1.

Evidence: `snowball-collision-reference.json` and its provenance. Eight cases run
unchanged Collision2DSystem/QuadTree/Circle/Tree with actual bundled THREE in
Chromium, covering tree-first and multiple-tree hits, strict tangency, large
balls, camera edges, the source circle-Y containment quirk and a split quad.
Eight cases run actual SnowballSystem methods with Object3D actor fixtures to
measure LIFO/setup/splice ordering; actor update supplies expiry and collision/
effect callbacks record calls, so these do not certify trajectory/pooling/effect
implementations. Four limit frames use actual LodSystem after collision and a
resize reset. Native tests exercise production tree queries, the game shot loop,
map-tree placement and that shared limit sequence.

At this stage dynamic contacts were still grouped by shot and pickups used a
later scan. The ordered-contact follow-up below supersedes those limitations,
including persistent quad topology and shot/shot contacts. Full arrival/freezing
phase timing, pooled actors and exact full-match randomness remain unfinished.
The rating stays Very similar; these fixtures do not establish whole-mode 1:1.

Validation for this correction: all 87 native tests pass and the release binary builds.


### Snowball Storm persistent collision and concrete contact actions — 22 November 2026

The production game now retains collider registration order and quad topology
instead of rebuilding grouped player/drop/shot lists every frame. It translates
the original directed notification and asymmetric measuredCollisions branches,
including the initial empty cache entry. Quad.clearDynamic preserves split child
nodes (the source checks this.nodes.length, not child contents). A source bounds
reset reinserts registered objects, including statics. Removal changes the Set,
not the current quad; bot queries use that same frame's retained quad.

Stable shot identities survive vector splices. New player setup, arrived-handler
activation, drop landing/spin removal, shot LIFO setup and shot/player removals
now change registration at their corresponding native system stages. Full
parachute/arrival phase timing and freezing-parent transforms are still partial.
The active local contact handlers run in the original notification order:

- Snowballs hit other snowballs, trees, presents and elves other than their
  thrower; every notification updates collidedWith and emits its hit event.
- An elf faces the incoming trajectory with the original -PI/2 formula. Each
  collision calls die/fall again, retaining repeated fall callbacks, while the
  first health/presence exit timestamp is preserved.
- A present keeps the first touching Elf instance, including one killed in the
  same pass. Pickup no longer scans living players later in vector order. Its
  collider persists through the next collision pass before DropSystem removes
  it during spin. PlayerSystem batches local pickup sound once per frame, even
  for multiple presents. The subsequent spin transforms retain source clocks.

`snowball-collision-lifecycle-reference.json` records five actual original
Collision2DSystem sequences / 22 frames: ordered directed notifications, nearby
candidate lists, node counts, moves across quadrants, removals/readds, bounds
reset with statics, stale statics retained by the original quad and primary-only
camera limits. Callbacks here only record notifications; they do not mutate
registration or XY during a pass. The actual active local callbacks also do not
change registration/XY during dispatch, allowing measured pairs to be dispatched
in the same order by Rust. Generic mid-pass mutation/subscription/debug behavior
is not certified.

`snowball-contact-actions-reference.json` separately runs unchanged actual
Elf/Drop/Snowball handlers plus Player/Drop/Snowball/EntityRemoval systems and
ready Model/THREE in five sequences / 20 frames. It checks shot/shot contact,
competing pickup order, multiple local pickups, repeated simultaneous lethal
hits, thrower exclusion, dead-player pickup, last collidedWith, sounds, registry
order, fall/listener state, exit timestamps and spin geometry. Native comparisons
use tolerance 1e-9. Actors start arrived with explicit measured transforms and
scales; map states are held at 1, and bot/parachute/LOD/input/effect integration
is inactive. Source collision random selects fall 2; native test selects a seed
with that sequence. This is not random/pool/arrival or whole-match certification.

The reference server rewrites dependency URLs to ?module. Direct harness imports
must use that same canonical URL for classes tested with instanceof. An initial
noncanonical handler run loaded two Elf/Snowball class identities and invalidated
pickup/death results; it was diagnosed with an unmodified-function debugger
breakpoint, discarded and regenerated with canonical imports before comparisons.
Do not preserve its inactive-handler conclusion. The older formula/shape oracles
do not rely on these instance checks.

All 89 native tests pass. Broader gaps remain: seeded full-match random/UUID and
object/model/particle pooling, exact parachute/freezing phases and asynchronous
LOD readiness, responsive/input/host lifecycle, audio buses/scheduling and full
visual/effect phase coverage. The scene stays Very similar, target 1:1.

The updated release builds, and a 150-second native simulation completes without panic.
That smoke check does not certify full-match equivalence or all rendered phases.


### Snowball Storm parachute and landing stages — 22 November 2026

`src/snowball_parachute.rs` translates the active ParachuteSystem queue and
stored canopy state. New Arrival.droppedTick starts at Infinity. DropSystem
queues new presents before ParachuteSystem, whereas PlayerSystem queues new elves
after it. Only the actual carry update sets droppedTick and resets local actor
position, so an irregular next frame no longer uses a guessed +1 tick. Descent
and sway use the exact source formulas without clamping a long frame. Rendering
uses the last completed system state, since the original renders before systems.

Arrival and joining the map are separate: elves are reparented later in the same
system update, but presents remain detached until the next DropSystem update.
That one render-frame absence is a source behavior, not filled with a fabricated
landing pose. Registration and Elf handler activation follow those stages.
Player movement/target processing follows landing; queued bot target positions
retain their actor identity until the post-landing throw origin/target is read.

The renderer now composes T(canopy) * Ry(sway) * T(0,0,-84) before actor/model
transforms. The previous flattened translation lost the rotated carrying offset.
Canopies draw independently of the carried actor's visibility, and the client's
marker is hidden until arrival per Elf.update. Unparented newly allocated actors
and detached landed presents do not render. Freezing/teleporting parent overlaps
and asynchronous original LOD/model visibility still need work.

Evidence and limits:

- `snowball-arrival-reference.json` / provenance: four actual original
  ParachuteSystem/Parachute/Arrival/THREE cases, 21 updates with before/after
  queues, local resets, descent/sway, arrival, explicit cancellation/reuse and
  the source teardown forEach/splice successor skip plus deferred detach.
  Plain Object3D actors have explicit child transform inputs; no full gameplay
  or LOD integration is asserted. Native state comparisons use 1e-9.
- The production renderer's canopy, carrying-dolly and child matrices match
  210 actual THREE matrices from those snapshots at tolerance 0.0003, reflecting
  native Float32 GPU transforms. This checks parent composition, not all pixels.
- `snowball-arrival-stages-reference.json` / provenance: three actual original
  Player/Drop/Parachute/Collision sequences, 22 frames. Explicit tiles, held and
  irregular ticks, late spawns and a long frame verify registration, local and
  map positions, handler activation, detached/map/parent state, drop spin start
  and both before-render and after-update queues. One actual Bot.update case
  verifies a target position reference surviving same-frame landing. Random
  selection is controlled and drop scales imported; maps are held passable.
  Full spawn random consumption, bots/population, LOD/model loading, effects,
  host/audio/erosion and restart are outside this fixture's scope.
- `snowball-arrival-render-reference.json` and `snowball-arrival-raster-errors.json`:
  four actual original and native 1440x800 present/parachute renders at ticks
  0, 30, 130 and 299, with a blank map and explicit matching camera. The native
  diagnostic imports measured canopy state to isolate parent rendering.
  Whole-image mean RGB error is 0.00294–0.00391 / 255; the object-region errors
  are 0.409–0.533 / 255. Original/native/difference crops were inspected. Small
  edge filtering/raster differences remain. Both source textures were awaited;
  an initial capture before canopy texture readiness was discarded. These are
  isolated present/canopy phases, not scene-wide fidelity or loading certification.

All 92 tests pass, the release builds, and a 150-second native simulation runs
without panic. Full-match random/UUID/pooling, freezing parents, scene lifecycle,
async LOD/loading, input/responsiveness, audio and other effect phases remain
unfinished. Snowball Storm remains Very similar with a 1:1 target.


### Snowball Storm freezing, teleport and actual parents — 22 November 2026

`src/snowball_removal.rs` replaces elapsed-time/render-only approximations with
original Presence and EntityRemovalSystem state. Freeze requests mark exiting
at their source system stage; the following removal update pops them LIFO,
allocates/reuses an iceberg, sets actor local position (0,0,-16), reparents it,
and advances the iceberg. Presents therefore stay visible on the freeze-request
frame and then inside the moving iceberg, rather than disappearing immediately.
Drift is a normalized 1/3 step per update; bob, turn and opacity use the stored
integer tick. Bounds removal is strict > against half-window diagonal plus
(rings+2)*cellSize. A null map returns after pending attachment, before frozen
motion or teleport processing. Pool reuse preserves source transform/uniform
state; indexToPosition explicitly resets z before adding 32.

Actors retain explicit Detached/Map/Canopy/Ice parents. Render transforms follow
that parent, not the frozen flag: a frozen elf can later reparent from iceberg
to map as its original parachute finishes. Icebergs and canopies render
independently from carried-actor visibility. Completed removal changes Presence
and system membership once; normal NPC cleanup detaches its map parent, while
the local client remains parented as the source does. Presence, Visibility and
material opacity are separate: at teleport completion Visibility becomes zero,
but Elf.update no longer applies that value once exiting is false, leaving the
last material opacity on the retained local mesh. Native rendering now consumes
stored opacity and entity z instead of evaluating a new fade with render time.
Drop spin/frozen tilt and opacity also retain completed system-update values.
Client death observation occurs before PlayerSystem, with the source's separate
wasDeadAt/strict >2500ms fallback and gone shortcut. This verifies that decision,
not the complete Client input/arrival-marker/camera/HUD event implementation.

Iceberg rendering uses stored transforms/opacity and source rotated cylinder
geometry. Opacity travels through a Float32 vertex channel instead of being
quantized into vertex-color alpha. Native root and child matrices are compared
with actual THREE, and the graphic still draws after contained actors.

Evidence and scope:

- `snowball-removal-reference.json` / provenance: four unchanged original
  EntityRemovalSystem/Iceberg/Presence/allocator cases, 26 updates including
  before/after queues, presence-less objects, repeated freeze/teleport guards,
  LIFO binding, retained pooled state, null-map return, strict bounds and exact
  1500ms hold/700ms teleport thresholds. Plain Object3D actors and explicit map,
  clock/window inputs isolate these operations; no game/actor pool claim follows.
- `snowball-removal-stages-reference.json` / provenance: three actual original
  Player/Drop/Parachute/Removal/Client/Collision sequences, 19 frames, with ready
  original elf models. They verify freeze-request versus attachment frames,
  present visibility/tilt, local and NPC cleanup, stored fade/material behavior,
  death observation/game-over timing and the frozen-parachute landing overlap.
  Explicit tiles/passable map plus requested eroded cells and selected fall
  randomness do not certify full seeded map/bot/LOD/loading or restart behavior.
- `snowball-removal-render-reference.json`: five actual original and native
  frozen-present/iceberg renders, blank map and explicit matched camera, with
  imported measured source state. Ten production parent/model matrices match
  THREE at 0.0003 tolerance. `snowball-removal-raster-errors.json` records mean
  RGB error 0.000997–0.00513 / 255 across the whole image, and 0.354–1.186 / 255
  in object-region crops. Original/native/difference crops were inspected.
  These are isolated effects, not full-scene visual or lifecycle certification.

The direct-window diagnostic's softer edges were initially attributed to the
application's MSAA. That conclusion was incorrect: the application already
renders through SceneFilter's single-sample target. The following compositor
follow-up corrects the diagnostic and records the actual high-DPI allocation bug.
Historical direct-window image metrics above are not application-raster proof.
Other remaining work includes full actor/model/material/particle reuse, complete
initialization/scene restart and LOD/readiness, Client arrival marker/camera/input,
responsive host overlays, audio buses/scheduling and all effect combinations.
All 95 tests pass; release builds and a 150-second native simulation completes.
Snowball Storm remains Very similar, target 1:1.


### Production compositor, device density and clipping — 22 November 2026

The Snowball diagnostic now follows the actual application path:
SceneFilter.begin -> SnowballView.draw -> SceneFilter.finish_live. It previously
drew directly into a 4x-MSAA window, while main.rs already used a single-sample
scene target. The earlier claim that the app's scene MSAA caused those edges was
wrong. Old direct-window metric files retain their numbers but are explicitly
marked historical. No normal-density application MSAA setting changed here.

A separate real issue was found in the shared compositor: scene and filtered
targets used logical dimensions even when the native window had a higher pixel
ratio. The original RenderSystem sets renderer pixel ratio from self.devicePixelRatio.
SceneFilter now allocates logical width/height * native screen_dpi_scale,
while layout, camera, mouse input and final presentation dimensions remain
logical. The disabled copy uses an explicit logical destination rectangle so a
higher-resolution texture fills the intended area. Density/size changes replace
the targets and invalidate the cached disabled image. Scene targets explicitly
remain single-sample. Existing blur shader formulas/timing remain unchanged;
this pass does not certify the original CSS blur kernel.

`src/surface.rs` converts logical scissor rectangles using the active render-pass
texture dimensions, or the window's physical dimensions when drawing directly.
24 clipping calls across Snowball, coding/results, Elf Maker, Elf Jamband, Selfie,
Present Drop, toolbar, village and shared action renderer now use that helper.
Their logical rectangle expressions and ordering are preserved (audited after
whitespace/optional trailing-comma normalization). Empty clipping remains empty,
including at fractional ratios. This is a GPU coordinate-unit correction, not a
translation/change to those scenes' gameplay or CSS layout decisions.

Evidence:

- `scene-filter-density-smoke.json` and provenance: actual production GPU
  allocation/readback at explicit ratios 1, 2, 1.25, 1, 2. Checks physical target
  dimensions, logical geometry and clipping, opaque enabled presentation, then
  disabled cached presentation while changing density. The test detects wrong
  size, crop, clip placement and cache invalidation. Actual OS DPI is 1; these
  explicit-ratio checks do not test monitor change events or every font/UI path.
- `snowball-compositor-raster-errors.json`: 27 isolated original/native present,
  canopy and iceberg captures at ratios 1, 1.25 and 2 (1440x800 logical; 1440x800,
  1800x1000 and 2880x1600 physical). Original THREE runs at actual browser DPR;
  native captures production scene target RGB at an explicit density, with the
  same opaque A=1 presentation as SceneFilter. Imported source transforms and
  blank maps isolate raster behavior; no full-match/whole-scene claim follows.
  Mean whole-image RGB error is 0.0000299–0.002165 / 255 and object-region error
  0.0187–0.3015 / 255. Original/native/difference crops were inspected. All nine
  selected normal-density images are byte-identical before/after the allocation
  change. Residual edge/filter/shader differences remain.
- The density GPU diagnostic is `cargo run --manifest-path rust/Cargo.toml
  --example scene_filter_density -- /tmp/density.json`. Snowball capture accepts
  a final pixel-ratio argument after mode (for example output.png 0 removal-81 2).
  This exports enabled scene RGB at target resolution, not native OS DPI events
  or a disabled host screenshot. Original render exporters support the same DPR.

All 96 native tests and the GPU density/clipping/cached-presentation diagnostic
pass. Source comparison still does not certify full high-DPI font/asset raster
resolution, host/overlay behavior, input, model loading or all scenes. No README
rating or user approval changes follow; full 1:1 remains unfinished.

The current release builds. Owned normal-density application captures for Elf Maker, Santa Selfie and Present Drop also render without panic and were inspected; they are smoke checks, not whole-mode visual certification.

### Snowball client arrival marker and camera — 22 November 2026

Read the complete ClientSystem, PlayerMarker, MainLevel, LocalLevel,
TetheredCameraTracker, Elf and PlayerSystem source, together with the engine
Game/SnowballGame update ordering. The two markers have separate parents and
lifetimes: Elf's shared clientPlayerMarker is under the dolly, whereas Client's
arrivalMarker lives directly in playerLayer and disables depth testing.

- `snowball_client::Arrival` stores the arrival marker. Client-stage updates
  attach it only for a living, unarrived player with tileIndex > -1 and no
  existing parent; the recorded transform stays unchanged on later tile/dolly
  changes. Its local position is (tile.x, tile.y + dolly.y, 19), rotation.x is
  dolly.rotation.x. Attachment snaps camera to (tile.x, tile.y * -0.75).
  Arrival detaches it; no-player and dead early returns leave it unchanged.
  Start clears the marker and resets level camera to the origin; it no longer
  applies the old invented dolly-y offset to camera placement.
- MainLevel's camera follows a living arrived player only while neither exiting
  nor gone. The extracted tracker retains source per-update acceleration,
  friction and browser `length` quirk (zero in the checked source context).
  Dead-player map picks retain their separate camera target.
- Both ring render paths now use the actual original PlayerMarker constructor's
  ShapeBufferGeometry (50 vertices and exact indices), instead of an invented
  48-segment ring. The material remains unlit, 0x22cc88, opacity .5, straight
  alpha. Arrival uses disabled depth testing, separately from the elf's ring.

`snowball-client-reference.json` and provenance run unmodified original Client,
MainLevel.update and tracker with actual THREE/grid and explicit player states.
Four cases / 28 frames cover invalid tile, attach-once, altered tile/dolly,
landing, exiting/gone camera guards, diagonal friction, resize, dead-camera
picks, death during descent, missing player and marker teardown. The native
oracle runs actual production Snowball.begin_frame for the camera branch and
production Arrival.update for the client branch. UI/event sinks and concrete
PlayerSystem removal are stubbed; clock remains 100ms, so this evidence does
not extend the previous death-timer certification or verify input/network.

Remaining source functions/branches: assignTarget cloning and deferred target
queue, assignDestination/PlayerSystem forwarding/network hooks, full HUD/host
score event timing, and actual player assignment/removal remain partial. Setup
is empty in source. Constructor's unused pendingSpawnAt has no active consumer;
source-wide unused imports/constants are not runtime behavior. Full restart
retained client fields, pooled shared elf-marker parent/visibility, model/LOD
readiness and transparent object sorting remain unfinished. Native local spawn
still picks the tile eagerly, while source PlayerSystem chooses a missing tile
AFTER ClientSystem; these tests use explicit tiles and do not prove that complete
initialization sequence. Do not infer full 1:1 from these bounded checks.

Four isolated original/native marker captures at 1440x800 were inspected with
20x difference crops. Attached/retained masks have identical coverage with a
one-unit green-channel difference; landed removal is pixel-identical blank
background. The dead-during-descent capture has one edge coverage discrepancy
(max channel error 41; union-object RGB MAE .47034/255). Full-image MAE is at
most .0000964/255; these tiny masks do not certify a full scene. Durable metrics
are in `snowball-marker-raster-errors.json`. Prior isolated arrival-130 and
removal-81 captures remain byte-identical after reusing the shader pipelines.
All 97 tests pass and the release builds. Adding two separate marker pipelines
initially exceeded Macroquad's fixed 32-pipeline capacity in the actual app;
existing compatible clip/particle pipelines now select the unlit marker branch
with a negative vertex tag instead. Normal geometry/random tags remain outside
that branch. Actual app startup checks pass; an owned live app with injected
X11 splash/F12 events also showed descent with the separate marker, followed
by game over. The latter is a smoke check, not a verified landed/whole-match
comparison. The earlier --advance-only captures remained on the splash screen.

The user then requested that tests stop interrupting their game and reported
unclear textures. No more windows/input tests will run during their game. Offline
`snowball-texture-audit.json` verifies the seven distributed PNGs retain source
bytes and dimensions. This rules out file replacement/resizing for those PNGs,
not filtering/material/camera issues. Native trilinear filtering is applied more
broadly than original THREE's three memoized maps; CSS UI and generated textures
still need matched sampling/render-scale evidence. No speculative sharpening,
nearest filtering or art changes were applied, and clarity remains unresolved.


### Snowball intro SVG density — 22 November 2026 (no window tests)

Source reread: complete scene index.js, index.html and index.scss, the powerup
status element, and the active SVG decoder/draw paths. Actual CPU decode, not
an assumed viewBox size, establishes the intro sampling gap:

- The loading-cloud vector decodes to 118x71 and the existing draw stretches it
  to 426.546x256 logical pixels on desktop. Its sample coverage was about .277
  per display pixel at 1x and .139 at 2x. Raster scale 8 produces 944x568;
  coverage is now at least 1.106 samples per display pixel at 2x.
- The parachuting-elf SVG has a 512x512 viewBox and old decode; the selected
  quarter is 256x256, displayed at 160x160 (or 100x100 on mobile). It already
  covered 1x, but its .8 samples per pixel at 2x required enlargement. Raster
  scale 2 produces a 1024x1024 sheet / 512x512 selected crop, covering 2x at
  1.6 samples per pixel. The initial unmeasured 90x90 claim was incorrect and
  was corrected before this change. Do not repeat it as source evidence.

Only two SVG raster-scale branches changed. Original SVG bytes, PNGs,
scene_size logical bounds, fractional source crop, destination size/position,
animation clocks, input and world textures remain unchanged. The production
CPU decoder reports the same decoded bounds after dividing out the raster
scale, and no enlarged bitmap is needed for the recorded 1x/2x cases.
Decoded RGBA storage grows by 5,256,984 bytes for these two textures; GPU mipmaps
add further storage. Cold-load/upload timing has not been profiled.

The new ordinary-main `snowball_texture_audit` example runs without a window,
GPU or input. It exports top-down CPU PNG bytes directly (Macroquad's Image
export_png flips GPU readback rows, so is unsuitable here). Before/after records,
source hashes and scope are in `snowball-intro-density-audit.json`. An offline
bilinear illustration of decoded pixels was inspected: the cloud no longer
shows the pronounced low-resolution enlarged edge. This is not a native GPU or
Chromium screenshot comparison; browser filter/compositor parity remains open.
All 97 existing tests pass and the release binary builds. No window or interactive tests were run, preserving
the user's request to leave their game alone.

This addresses a specific intro bitmap-density error. It does not establish
that all reported unclear Snowball/world textures are fixed. CSS PNG sampling,
3D minification, generated elf-map eye raster and their visual appearance still
require matched evidence; do not replace source filters with arbitrary nearest
sampling or sharpening. The saved original elf texture samples differ from the
current binary-circle reconstruction at 302 eye-edge pixels per 512x512 sheet;
that offline reconstruction is only a locator for future native-generator work,
not verification of all generated textures or the cause of scene-wide blur.


### Snowball generated elf maps and glTF sampling — 22 November 2026

No windows or input were used. Reread the complete generateElfTexture closure
and initializeModel callback, Texture constructor/clone/copy, GLTFLoader's full
loadTexture method, source textureMemo, active THREE upload/parameter functions,
model JSON and native upload/draw/backend wrap path.

Confirmed sampling distinction: tiles/snowball/parachute use TextureLoader's
clamp defaults, but the glTF elf's empty sampler resolves to RepeatWrapping on
both axes, linear magnification and trilinear minification. Elf clones that map
and replaces only its image; Texture.copy preserves wrapping/filtering/flipY.
The 512x512 generated map is power-of-two, so these settings remain active even
in WebGL1. Native Macroquad/miniquad upload defaults to clamp. configure_texture
now applies Repeat to the generated elf map and isolated imported elf reference
map only. Other world-map wrap/filter settings are unchanged. Among 2,746
source UVs, two extend below zero in U (minimum -.0023183972); both-axis source
wrapping is relevant, not an arbitrary sharpening choice. Native draw_mesh uses
the original texture directly, and the locked OpenGL backend maps Repeat to
GL_REPEAT. This is source/API tracing, not a GPU state query or rendered proof.

GLTFLoader's embedded-buffer/DDS/handler/object-URL branches are inactive for the
bundled external elf PNG and empty sampler. Generic glTF loading, asynchronous
readiness and errors remain partial; this change does not port those branches.
Other scenes' glTF maps require their own sampler audit. Default anisotropy is
one; blanket nearest filtering or extra anisotropy is not a source match.

`src/snowball_texture.rs` replaces the binary integer-distance eye raster with
native Canvas-like rectangle/path paints in the source order. It retains exact
512x512 dimensions, palette values, pony-tail clear/paint region, white hat,
skin/clothes/boots rectangles and final buckle. A fresh canvas is equivalent
here because all pixels are cleared/covered by the original operations. Source
retains its eye path without beginPath, appending the same two complete circles;
repeated same-winding contours describe the same filled region. Native uses
those two antialiased circle paths. Tiny-skia versus browser Skia edge coverage
still differs; this is explicitly not an exact Canvas raster implementation.
View still owns the source (major, minor) cache key, ignoring later hair/pony;
allocation/RNG/full restart/cache population order is not certified by this pass.
Direct RGBA upload replaces data-URL encoding without changing the intended
image; readiness/lifecycle timing remains a separate unfinished area.

The CPU-only `snowball_elf_texture_audit` example compares actual native output
and the immutable preceding production loop against all 28 saved original
browser-generated texture PNGs. Recorded major/skin colors are used; because
this older fixture omitted hair/pony, only those constant non-eye regions are
used to infer those parameters. No original eye pixels/coverage enter the new
generator. Pixels outside x=[23,171), y=[186,228) match exactly in all 28 maps.
Mean RGBA error decreases 88.47–88.75%; remaining differences are 284–290
pixels per map, max channel error 20–36, all on eye edges. Inspected enlarged
source/old/new eye crops and 4x differences. Whole-map means are small because
the changing region is tiny; they cannot certify complete textures/rendering.

Evidence: `snowball-elf-texture-errors.json`, `snowball-world-texture-provenance.json`
(source/backend hashes, actual sampler JSON and out-of-range UVs), and the new
28-image regression. That test demands exact non-eye regions, per-channel
coverage bounds and at least fourfold reduction against the measured legacy
error; it does not assert pixel equality or upgrade the mode's rating. All 98
tests pass; the release binary builds. No runtime/window/driver test was run.
Full world-texture clarity, minified GPU appearance, CSS PNG sampling and model
lighting/render/compositor fidelity remain open. Source linear/trilinear world
filtering is retained rather than changed based on the blur report alone.


### Snowball queued commands and destination marker — 22 November 2026

Read the complete ClientSystem/PlayerSystem/SnowballSystem command paths,
MagicHexGrid playerWaypointsForMap, Elf.update and the active components,
MainLevel.update, MapSystem.update and DestinationMarker constructor/setup.
The original Game/SnowballGame order is Level -> render/input -> collision/LOD
-> Snowball -> removal/drop/parachute/effects/map -> Client -> network/bots ->
Player. Native dispatch now respects the stages instead of executing targets
at input and movement/fallback throws directly from the next level update.

- Map picks stay pending until MainLevel, which forwards a living client's
  destination to Client. Enemy target picks clone the point at input. Client
  forwards both queues only while alive; death retains them. Player applies
  destinations only after arrivals have been positioned, discarding unarrived
  or dead entries at the end of that pass. Per-player maps keep insertion order
  and replace values on repeated assignment.
- A failed local path queues a Client target for the next update; failed NPC
  paths do not throw. All destination requests precede all targeted positions.
  Shots created at Player stage remain unparented/unborn until the following
  Snowball update; multiple new shots still set up LIFO. The previous early
  input throw bypassed this delay. Large-charge type/depletion is decided at
  dispatch, after collection/swap, once per replaced target entry.
- Bot target positions retain a player reference until Player command handling,
  so an arriving target's reparented map position is used. Client's cloned point
  remains fixed even if the selected actor later moves. Assigning a dead
  player's target sets the source animation-request flag, even though no shot
  fires; it remains until a living Elf update. Living MainLevel updates also
  expire any old dead-camera pick, as the source always reassigns tracker.target.
- MapSystem snapshots destination-marker position/visibility BEFORE commands
  and movement. Renderer uses this state, preserving both appearance delay and
  final reached-frame visibility. The cross now uses the actual 12-vertex
  ShapeBufferGeometry instead of two overlapping screen-space line strips:
  size 10, arm width 10/3, rotation z=PI/4, z=cellSize*.5 (32 in the game),
  y=destination.y-20, color
  0x22cc88/opacity .5, double-sided and depth testing disabled. Existing marker
  shader pipelines are reused; no extra GPU pipeline is allocated. The camera
  matrix is extracted unchanged into a context-free helper for CPU verification.

`tools/snowball-command-reference.cjs` runs unchanged original modules and bundled
THREE in Node SourceTextModule (canonical module identities), not a browser.
14 cases / 64 frames include cloned targets, level/render delay, blocked paths,
explicit target plus fallback, unarrived discard/landing-frame destination,
death retention, per-player replacement, LIFO setup, landing target reference,
failed NPC path, retained dead animation request, powerup charge selection and
depletion, alive/dead camera target changes, and marker endpoint timing.
Native production begin_frame/input APIs/finish_frame compare 192 full state
snapshots (render, after input, after Player), including points/paths/facing,
flags/queues, camera/velocity, charges, shot IDs/origins/targets/birth/setup/type,
marker state and selected sound actions. The source's afterClient snapshots
are also stored but not compared at a native intermediate hook.

The reference uses explicit map/arrival/death/command inputs, Math.random=.5,
empty actor/shot pools, low-LOD actors without loaded models, and inert image/
Canvas/DOM I/O. UI/network/event sinks, collisions, LOD, effects and parachute
callbacks are stubs; actual bot AI is not run. Native collision registration is
reset empty per test frame for the same isolation. Explicit bot commands are
queued before native finish (they commute with the earlier isolated stages in
these cases, with no same-player Client/bot overwrite conflict). This does not
certify asynchronous/loaded-model animations, actual collision interactions,
RNG/UUID/pooling, all spawn/restart states, network or raw octree input.

Three supplied viewports' original THREE marker clip coordinates also match
the production native mesh/matrix helpers, including cell sizes 64 and 32. These are geometry/state comparisons, not GPU pixel renders.
No window/input checks were run while the user plays. The pointer diagnostic
now dispatches one real finish_frame after an input release so a queued target
can be processed; that interactive path was updated but not executed this pass.
No README faithfulness rating or user approval was changed. Full mode 1:1,
rendering, touch/keyboard/host input and actual sound-bus fidelity remain open.

Validation for this pass: all 100 native tests pass (89 library, 11 application
unit tests); the release application and snowball_capture example build. These
checks do not launch a window or certify interactive/GPU behavior.


### Snowball elf root visibility and shared ring — 22 November 2026

Read the complete original LodSystem, Elf constructor/allocation/setup/update/
teardown/lod setter, Parachute constructor/carry/lod/onFreed and ParachuteSystem
queue/registration order. Traced Rectangle/Point containment and existing
Player/Removal/Client order. No window, browser or OS input was used.

`src/snowball_lod.rs` translates current HIGH/LOW, lodNeedsUpdate, root visible
and model-requested flags. Constructor is LOW/dirty/visible/unrequested; setters
ignore dead players and only mark dirty on a changed value. Elf.update consumes
dirty: HIGH+alive makes root visible/requests the model; otherwise it hides root.
Health revival alone does not retry a pending HIGH that was consumed while dead:
this source quirk remains until another LOD transition. modelInitialized in the
source means requested, not promise resolved; the native field is named that way.

LodSystem runs after collisions using the current camera's x/y*-4/3 and the
width+256 by height*4/3+256 rectangle. Full 15x45 local elf collider containment
is inclusive. A canopy's Point LOD forwards to its carried object; it registers
after that elf and can override the local-collider result. Native preserves both
assignments and their dirty transitions, before Player.update consumes them.
Forwarding still occurs when the actor has been reparented elsewhere, because
source carriedObject is retained until free. Default Drop.lod is an assigned but
unused property, not a reason to hide gifts. Canopy graphics are not hidden by
this forwarded LOD. Generic duplicate registration/redrop/pool aliasing still
needs broader checking; the new state is not a whole LOD engine certification.

The shared clientPlayerMarker under the dolly is separate from Client's arrival
marker. Setup attaches it; arrived controls its mesh visibility. Death alone
keeps it attached. Only a dead exiting Elf update detaches its own marker; a
freeze at the end of the Player pass therefore retains it for that next render.
If a time jump skips all exiting Elf updates, it can remain attached after gone;
Native preserves this behavior rather than hiding every dead elf's ring. An old
owner cannot remove a marker reparented to another dolly. Elf.teardown itself
does not detach it. Native renderer checks stored owner/mesh visibility and root
visibility, replacing the inferred alive&&arrived test. It uses existing shaders;
no new GPU pipeline is allocated. Per-frame skeleton updates remain independent
of root visibility, as in source.

The source module marker retains visibility across scene instances. Native start
keeps that flag but maps an old dolly outside its cleared actor set to no current
owner, until setup. Raw old-parent identity and complete pooled scene teardown
are still unimplemented, not established by this abstraction.

The CPU-only source VM uses unchanged Elf/Player/Lod/Parachute/Removal/Client and
actual THREE. Original GLTFLoader reaches an I/O adapter that leaves FileLoader
pending. Thus it verifies requested/root/ring decisions while no model resolves;
it cannot prove loaded-model callbacks, texture/material updates or animation
readiness. Image/Canvas, collision/input/event/network sinks are inert; no AI or
LocalLevel runs. Source UUIDs use a seeded LCG, not the earlier constant-random
command harness, so Removal's UUID keys stay distinct. Pools start empty per
case; source global model promise/marker retain their normal memoized state.

`snowball-lod-reference.json`/provenance cover 8 cases, 46 core frames and 13
separate ring operations. Actual native finish_frame compares 92 post-Player/
post-command snapshots (root/LOD/request flags, parent, active/arrival/presence,
position/camera, ring owner/visibility and hierarchy eligibility). Source afterLod
data is retained but not directly hooked in native. Cases cover inclusive and
outside bounds, resize, dead HIGH/LOW retention, pending-HIGH death/revival,
late freeze removal, skipped exiting updates, canopy override/boundary/death
and landing. The separate ring sequence exercises reassignment and teardown,
not the port's complete fixed-local-index controller. sceneVisible is hierarchy
eligibility, not a frustum or pixel-render check.

All 102 native tests pass. Model IO remains eager/shared in the view; async
completion, pooled loaded objects/material opacity/actions, input octree and
GPU appearance remain unfinished. The legacy frozen reference capture imports
poses/textures but no LOD/readiness flags: it explicitly keeps ready-to-draw
root isolation and restores the known living/arrived local ring. That diagnostic
was updated/compiled, not run while the user is playing. No rating or user
approval changes follow this state-machine verification.

Release validation: the application and snowball_capture diagnostic build. No
window/GPU/input path was executed in this pass.


### Snowball basic-material alpha correction (22 November 2026)

Read complete Drop, Parachute, Obstacles and texture modules. Actual bundled
THREE evaluated in CPU Node VM confirms MeshBasicMaterial alphaTest=0,
opacity=1, depthWrite=true, premultipliedAlpha=false; both source gift/canopy
materials are transparent and leave alphaTest unchanged. Obstacles instead
explicitly discard texture*fade alpha below .1. The native shared clip shader
previously applied that tree cutoff to gifts and parachutes too. It now selects
a no-cutoff branch for those basic materials in the same existing pipeline.
Gift opacity travels in a float attribute instead of a byte color, preserving
the source continuous material fade. Canopy uses opacity=1. Marker and tree
branches remain unchanged; no extra GPU pipeline is allocated.

CPU array comparison also confirms all 72 position/48 UV/72 normal scalars and
36 indices in the shared ski-present geometry exactly match Snowball Drop.
The source maps still request linear/trilinear filtering; no unsupported
sharpening or filter change was made. See snowball-basic-alpha-provenance.json
for hashes and limits. No window, GPU or input was used. These corrections
address a known fringe/late-fade mismatch, not proof of the reported fuzziness's
cause. Transparent ordering, loaded materials/pools and GPU appearance remain
unverified. No README rating or user approval changes.

Release application build passes. Shader runtime/pixel comparison was not run;
prior 102 state tests were not rerun for this render-only change.


### Snowball output-encoding correction (22 November 2026)

The reused ski-standard fragment included LinearToGamma output with gamma=2.
Source Elf Ski explicitly enables gammaOutput; complete Snowball RenderSystem
uses new WebGLRenderer with its false default and no override anywhere in the
scene. MainLevel's ambient light is 0xbbaaaa at PI; native values already match.
GLTF material base-color factor is .6399999713897699 and metallic=0; native
float .64/roughness=1/metalness=0 matches those factors/default. GLTFLoader
sets base-color map encoding=sRGB; Elf clones the material/map before replacing
the image and preserves that encoding. No texture-decode change is appropriate.

CPU snowball-encoding-reference.cjs exposes unchanged bundled WebGLPrograms
and texel encoding/decoding helpers through extra exports. It evaluates the
literal original renderer configuration assignments, supplies source-reviewed
glTF sRGB map encoding and executes getParameters without creating a renderer,
GPU, canvas, browser or input. Unrelated capabilities/empty light arrays/current
default framebuffer are adapters; it is not actual glTF or renderer lifecycle.
Actual helpers select LinearEncoding (3000) for Snowball output and GammaEncoding
(3007) for Elf Ski; both maps decode sRGB (3001). Fixture stores exact source
helper strings and source hashes. This verifies configuration selection, not
shader compile/execution, illumination integration or framebuffer pixels.

Production Snowball now uses snowball_material::elf_fragment to replace only
the inherited output helper with LinearToLinear, retaining texture decoding,
shared lighting, vertex colors and Elf Ski's separate gamma shader. Regression
checks compare the emitted helpers to actual source-selected helpers for both
scenes. No arbitrary sharpening/filter change. The old square-root output lifted
midtones, so this corrects a known color/contrast mismatch, but the user's overall
texture-clarity report is not considered solved without visual verification.
Other shaders, transparency sorting, loading/pools and full world/UI parity
remain incomplete. No window/focus/input or README/user-rating change.

All 103 tests pass (92 library + 11 binary). Release application and capture
diagnostic build; neither was executed. No window, GPU context or input used.


### Snowball point-effect raster configuration (22 November 2026)

Complete TrailEffect, SnowsplatEffect, EffectSystem, Trail/Splat components and
normal/big Snowball source reviewed. Both effect layers use Points with explicit
RawShaderMaterial gl_PointSize. Renderer refreshUniformsPoints multiplies size
by pixel ratio only for isPointsMaterial, which these materials are not. Native
quad expansion previously divided size by logical screen dimensions, enlarging
points at high DPI. It now divides by the active physical framebuffer size,
including offscreen targets. For an analytical 60px point at 1/1.25/2x, logical
width is 60/48/30px. These are viewport equations, not observed raster footprints.
Existing active-target size lookup was extracted from surface::scissor and
shared; clipping arithmetic and caller coordinates remain unchanged.

Native emits trails before impacts rather than interleaving chronological
emissions. CPU original effect constructors plus actual THREE WebGLRenderList
sort (pushed in reverse to distinguish insertion from stable ID sorting) confirm
trail then impact for their equal transform/depth/order roots. Raw material flags,
depthTest=false, capacities 500/2000 and literal point-size shader assignments
are observed. The reference script adds only an internal export to the bundle;
source modules are unchanged. Constant RNG=.5 applies only to construction's
unused random array, not gameplay parity. No renderer/window/GPU/network/input.

Vertex arithmetic now starts from f32 uniform/attribute inputs like source: time
and displayTime cast separately before subtraction, random cast before fract,
float size calculations; trail retains its own PI=3.14159 literal. Native also
honors source trail displayTime<=0 and impact displayTime<1 fragment guards.
No new pipeline or texture filtering change. GPU transcendental precision,
point coverage/center rounding, hardware size-range clamping, texture orientation
and complete scene transparent ordering remain unverified. Fixed source 500/2000
slot rings, persistent per-slot randoms, wrap quirks, exact effect-stage emission
and pooled registration/restart lifecycle are NOT implemented by this pass.
Native remains chronological age-retained emissions within each layer.

All 103 tests pass (92 library + 11 binary); no new mirror tests were added for
these source-traced render changes. See snowball-particle-raster-provenance.json
for observed flags, source hashes and explicit limits. General texture clarity
and full mode/site 1:1 remain unfinished; user feedback/README ratings preserved.

Release application and snowball_capture diagnostic build. Neither was run;
GPU appearance remains unverified while the user plays.


### Snowball fixed effect buffers and stage clock (22 November 2026)

Read complete effect modules/components, Snowball/BigSnowball, SnowballSystem,
SnowballGame, engine Game/Gamelike/ClockSystem and index.js restart path. Native
snowball_effects now stores 500 trail and 2000 impact records matching source
Float32 attributes. Random values are filled once per impact slot at game start,
not per hit. Trail registration preserves duplicates and splice(indexOf,1)'s
absent-object removal of the last registration. Optional/show-false trail data
skips writes; colors retain source integer hex channels. Trail cursor wraps per
write. Queued splats are stable component keys resolved at flush, in LIFO order;
repeated keys see mutations made before processing. At end overflow, splats
write at zero but advance the OLD cursor, preserving source's overlap quirk.
Exact remaining-capacity fill, zero quantity and oversized typed-array writes
(ignored outside capacity) are covered, not normalized into a different model.

Production Snowball setup registers trails and removal unregisters/queues splats.
Effects now flush after Removal/Drop/Parachute and before Map/Client/Bot/Player,
as in SnowballGame.update. Removed native Shot values are stable until that
flush because Player allocates new shots later; global pooled Snowball/Splat
component aliases are still incomplete. A bounded slot-order render snapshot
filters expired records and reuses Vec capacity; source GPU instead submits
all slots and discards expired fragments. Source GPU size/coverage remains open.
Renderer uses stored Effects.time, as the source renders before the next effect
uniform update; current-frame time no longer advances an older particle buffer.
Source frame/ClockSystem getter timing and host clock synchronization are not
fully reproduced by this bounded constant-clock verification.

EffectSystem.teardown inherits no-op attribute teardown. This does NOT mean
buffers survive a website restart: complete index.js startGame destroys the old
Game and constructs a new SnowballGame/EffectSystem. Native start now creates
fresh buffers/random slots. Unused splash map/effect placeholders consume no
gameplay random calls; effect random samples are consumed before the playable
map seed, like source construction. All THREE UUID/object/pool random draws and
full seeded match order remain unverified. Ordinary restarting resets source
Game's clock as well; native effects use the existing match time namespace.
The source's unused EffectSystem.trailedObjects array is not reproduced.

CPU snowball-effects-reference.cjs runs unchanged original modules with actual
THREE Float32 BufferAttributes. Only TextureLoader image I/O is replaced by an
empty texture handle; seeded source constructor RNG includes THREE UUID calls.
Native test imports the resulting 2000 random attributes, which certifies slot
retention/writes rather than whole-game RNG ordering. Fourteen steps compare
every field in all 2500 slots each time: 35000 records/280000 numeric fields plus
cursors, registration order and stored time uniforms. Cases include null/false
Trail, duplicates/absent remove, 503-object trail overflow, duplicate mutated
splat keys, null Splat, 1998->7->7 wrap (cursor 1998->5->12), oversized 2003,
exact fill, zero quantity, post-lifetime data retention and teardown. GPU expiry
fragments were not executed. Source delta fixture stores all
changes; native reconstructs full expected buffers before every comparison.

A separate production finish_frame regression checks pending shot setup ->
trail emission -> removal/impact -> new-game reset, no reroll on impact and the
stored render-time lag. That check is native plus source-order tracing, not a
whole original game/renderer execution. All 105 tests pass (94 lib + 11 binary).
Map readiness, full component pool identity/aliasing, hidden/paused clock timing,
GPU point range/center snapping/texture orientation and all scene composition
remain unfinished. Source/provenance JSON records scope and hashes. No window,
GPU context, focus change or input; no user approval or README rating changes.

Release application and snowball_capture diagnostic build successfully. Neither
was launched; user game/window/input remained untouched.


### Projectile material clock and size-specific reuse (22 November 2026)

Original Snowball update sets its material time before thrown/birth/advance
branches; render already happened earlier that frame. Constructor initializes
random/time once. onAllocated resets trajectory/presence/birth/skew/visibility
but does NOT reroll random or reset material time. Static Allocatable class
pools survive the index.js new-Game restart; normal and BigSnowball classes
have separate weak-map pools. Native now retains random/time in size-specific
LIFO material-state pools, including pending-before-active free order on restart.
Removal before update caches the old clock; removal after advance caches the
new one. Native updates shader_time only for actually visited active shots.
Batched projectile shader uses that per-shot float attribute rather than the
new frame's global time. No additional pipeline or texture sampling change.

Source onAllocated copies Vector2 trajectory.origin to Vector3 position, leaving
z undefined until advance sets it. Native dispatch previously put a shot at z=0;
it now represents undefined height as f64 NaN. Source layer/visible eligibility
is distinct from birth tick: a newly set-up shot skipped by splice may be
parented/visible with birth=-1 and material time still old/zero. Renderer uses
setup && no collidedWith (native hit) instead of the birth gate. Undefined/NaN
positions are compared as JSON null. This is state equivalence, not proof about
nonfinite GPU clipping or actual pixels. Native passes nonfinite coordinates
through its geometry path; driver behavior remains unverified.

CPU snowball-projectile-reference.cjs executes unchanged SnowballSystem, normal/
BigSnowball, Allocatable and bundled THREE. Clock is supplied; collision/effect/
audio sinks are inert and TextureLoader image I/O returns a texture handle.
Thirteen frames/26 before-after snapshots cover 13 throws (5 fresh, 8 reused),
initial height/birth, mixed-size LIFO setup, skipped new-shot update, collided
hidden actor retained by splice, expiry, cross-system reuse and teardown with
pending before active. Native actual finish_frame checks position/birth,
layer/visibility eligibility, stored clocks/randoms and pool order. Fresh source
constructor random inputs are imported ONLY for source-verified fresh allocation;
native pool-empty decisions are asserted first, and reused values must come
from native caches. This does not certify whole-game seed/UUID draw order.

Native pools graphics random/time state, not complete source actor UUID, collider
object, subscription or Splat component alias identity. Existing effect-stage
flush precedes subsequent Player allocation; removed Shot values remain stable
until that flush. Full original model/image readiness, route realm/clock lifetime,
queued pool aliases and GPU geometry/UV/transcendental/pixel behavior remain
unfinished. Fixture restart recreates only SnowballSystem with a supplied caller
clock; full index/Game restart is source-traced separately. All 106 tests pass
(95 lib + 11 binary). No window, GPU context, focus/input action or rating/user
approval change. See projectile reference/provenance JSON for hashes and scope.

Release application and snowball_capture diagnostic build. No window or GPU
check was run; user game/focus/input stayed untouched.


### Map performance clock and float fade attributes (22 November 2026)

Read complete GameMap, HexMap, Obstacles, MapSystem, shader partials and active
LocalLevel->GameMap erosion path. GameMap.setTileState writes performance.now()
through a Float32 InstancedBufferAttribute. Raised tiles use this setter during
construction too. HexMap.update and Obstacles.update store performance.now(),
not ClockSystem.time. Performance namespace continues across index.js new Game,
whereas projectile/effect game clock resets. Source Game renders before the
MapSystem uniform update; a LocalLevel tile transition can therefore be newer
than the stored shader time in that render. Preserve the negative elapsed value
and source fade/scale expressions instead of clamping time or advancing clocks.

Native Map::new_at/set_state now retain float32 timestamps. Map::new remains
the time-zero controlled constructor. Runtime uses scene-monotonic wall for map
creation/erosion, separate from match-relative self.time. Stored hex/obstacle
shader times initialize/reset to zero and update after Effects at the Map stage;
render tone/deformation uses those stored times, with separate f32 casts before
time subtraction. Native logical wall is the model of supplied performance time,
not proof of exact browser navigation/load/host/background clock alignment or
within-frame multiple performance getter timings. Those remain unverified.

Tile fragment fade previously traveled through vertex color's u8 alpha. It now
travels through unused float UV.x. Tree fade uses float normal.w with tag -3,
then the original .1 alpha cutoff; gift/canopy tag -2 still has no cutoff and
marker -1 keeps its uniform color. Tile side alpha and highlight/raised tones
remain in the original fragment expressions. No new pipeline/filter/texture-size
change. These are source-traced corrections; shader/CPU float transcendental,
coverage and actual alpha behavior have NOT been verified with GPU captures.
The exported erode shader helper is not imported by HexMap/Obstacles; their
active formulas are shake/sink/fade, so it is not silently substituted.

CPU snowball-map-clock-reference.cjs executes actual original setup/rebuild,
GameMap setters/erosion, MapSystem updates and teardown/reconstruction with
bundled THREE attributes/materials. Source performance clock is constant per
stage and independent of supplied relative game time; input/collision sinks are
inert, TextureLoader image I/O returns an empty texture handle. Ten frames cover
4096 initial tile state/time pairs, 203 changed pairs, fractional timestamps,
10-million-ms precision, erosion RNG, newer transitions versus old shader clocks
and restart. Native reconstructs and compares ALL 4096 pairs before/after real
finish_frame each frame, plus erosion RNG and both material clocks. This proves
bounded JS state and typed-array timing, not GLSL execution or all-map rendering.
Fixture restart reconstructs MapSystem; full Game/host lifetime is traced apart.

The legacy frozen full-reference capture explicitly imports wall and both map
shader clocks: its original exporter performed two updates at time=6000 and
then explicit rendering at that same supplied performance timestamp. Other
isolated blank-map captures are unaffected. Diagnostic changed/built, not run
while the user plays. All 107 tests pass (96 library + 11 binary). Complete
map/input/render/clock/loading parity and overall texture clarity remain open;
no user approval/README rating change and no window/input/focus operation.

Release application and snowball_capture diagnostic build. Neither was launched;
GPU appearance remains unverified and user game/focus/input stayed untouched.


### Snowball material blending, sidedness and plane winding (22 November 2026)

Source Elf.initializeModel clones the glTF material/map and explicitly sets
transparent=true. Native reused Ski's opaque standard pipeline (phong=false),
so elf opacity/texture alpha did not blend like source. Snowball now supplies
its own straight-alpha pipeline to the shared standard-shader factory. Ski's
existing lit wrapper still passes precisely its former pipeline(phong), so no
Ski material-setting change follows this pass. All source scene material kinds
are transparent: default normal RGB SrcAlpha/OneMinusSrcAlpha and separate alpha
One/OneMinusSrcAlpha, except additive iceberg whose blendFunc uses SrcAlpha/One
for BOTH RGB and alpha. Native iceberg alpha previously used normal alpha factors.

Source FrontSide applies to elf, hex tiles, obstacles, gift, canopy, player ring
and arrival ring; destination, snowball, iceberg and both point effects use
DoubleSide. Native pipelines previously culled nothing. Front-only kinds now
use back-face culling/CCW; source default BACK cull selection and positive object
world determinant rule were traced in THREE. Current glTF has no negative-scale
nodes/matrix nodes or scale animation targets; arbitrary reflected transforms
are NOT supported/certified by this fixed-policy pass. Source-render geometry
and native projection/skin transform checks from prior passes remain bounded.

The native generic plane/obstacle quads were clockwise and used a different
diagonal. They now use source PlaneBufferGeometry triangles, remapped to native
TL/TR/BR/BL storage as 0,3,1,3,2,1. This correction accompanies culling rather
than enabling culling on oppositely wound geometry. Unit check compares actual
source plane triangle positions and existing top-left-upload adapted UVs at two
base offsets. Real GPU winding/clip/pixels remain unverified while user plays.

Arrival ring shares the existing no-depth particle pipeline: fragment marker
tag -1 discards !gl_FrontFacing; destination cross tag -4 allows both faces.
Depth-tested player ring uses the culled clip pipeline. No new GPU pipeline;
Macroquad's global 32-pipeline constraint is preserved. Native no-depth pipelines
disable effective depth writes, matching GL behavior when depth test is disabled
even though those source material.depthWrite properties remain true. No other
texture filtering/size/timing or game logic changes. Full transparent sorting
and every material/geometry interaction remain unfinished.

CPU snowball-material-state-reference.cjs constructs actual original entities,
client arrival marker and effects with bundled THREE and setup methods. Actual
GLTFParser.loadMaterial executes with an image dependency adapter, then the
source-reviewed Elf clone/transparent override is applied manually; this is NOT
actual createElf/skeleton/async callback execution. Original extracted setMaterial,
setBlending and setFlipSided function bodies execute unchanged with recording
GL/state adapters, enum constants and caches reset unknown per kind. This
records requested backend state, not WebGLState construction/real GL calls or
call counts. Source initial BACK cull default is traced separately. The fixture
supplies positive object-world determinant. Actual GPU output is not inferred.

Production pipeline policy compares cull/front/depth and RGB/alpha blend factors
to those recorded source function results for all 12 kinds. All 109 tests pass
(98 library + 11 binary); release application and snowball_capture diagnostic
build. Neither was launched. Texture/model readiness, driver front-face behavior,
source callback/full object identity, global transparency sorting and all pixels
remain unverified. Source/provenance JSON records hashes and precise scope.
No window/focus/input change, user approval change or README rating change.


### Iceberg shell render priority and ordering (23 November 2026)

Source Iceberg.setup gives its graphic renderOrder=1 explicitly so frozen
children render correctly. THREE sorts transparent entries by groupOrder,
renderOrder, descending object-origin clip depth then Object3D.id. Source
ordinary scene groups/render orders are zero; iceberg shells therefore draw
after ordinary world objects even when arrival/destination markers disable
depth testing. Native previously drew shells inside effects(), then markers.
Shell pass now runs after those markers and before HUD, preserving priority.
This does NOT finish the global order-zero queue: other native draw groups
remain fixed, and their source depth/ID interleaving still needs implementation.

Shells now sort by original default local camera/gimbal double clip depth, not
GPU/f32 vertex depth. Source quaternion-derived gimbal rotation and camera's
fixed X=-PI/Z=0 (after Game.setLevel lookAt/z reset), near=1/far=100000 are
represented in f64. Actual source camera/matrix comparisons bound this helper;
arbitrary camera projections/rotation are not generalized. Equal shell depths
use body slot creation order: source cellSize is fixed64, so each radius32
graphic is created once per body and retains its Object3D ID across free/reuse.
This equivalence is checked against actual graphic IDs; changing radius can
recreate a graphic and requires separate ID-generation tracking, still outside
this pass. Native body pool indices are not claimed as all scene object IDs.

New source comparator mirrors branch order, including pairwise NaN subtraction
returning JS-sort equality without falling through to ID. Only finite production
shell poses are in the ordered corpus; arbitrary multi-NaN sort algorithm
behavior is not certified. Generic comparator is currently used for shells,
not as a claim that all THREE rendering has been translated.

CPU snowball-render-order-reference.cjs uses unchanged original Iceberg/marker/
client/effect constructors, fixed-radius allocate/free/setup reuse, actual THREE
world/camera matrices, Frustum and WebGLRenderList.sort. Eligibility traversal is
fixture code for visible mesh/points with default ancestor visibility/layers and
groupOrder=0; it is not the whole renderer. Six controlled scenes cover mixed
depths/heights, equal/near ties, reverse insertion/reparenting, pool reuse and
mobile/camera changes. All six shells remain within source frustum in every
case. Production helper depths match within 2e-15 and sorted slots match exactly.
Native may still submit offscreen shells; GPU clipping/blending/coverage, whole
scene IDs/async model readiness, global sorting and radius changes remain open.

Comparator tests also reproduce actual source ordinary+ice order from captured
source keys, verifying shell priority despite nearer markers, plus group/priority/
depth/ID/signed-zero/NaN pair branches. This is CPU requested draw order, not
GPU output or original full-game simulation. All 111 tests pass (100 library +
11 binary). No window, focus/input action, user approval or README rating change.

Release application and snowball_capture diagnostic build successfully. Neither
was launched; user game/focus/input stayed untouched.


### Deferred spawn tile/count staging (23 November 2026)

Read complete PlayerSystem/DropSystem, Arrival/StateSystem, GameMap habitable
lookup and ParachuteSystem update branches. Original addPlayer/addDrop creates
arrival request -1 (or explicit tile), without calling map habitable selection.
DropSystem newDrops.shift processes setup then fallback tile choice and canopy
queue. PlayerSystem newPlayers.shift chooses fallback later, then setup/queue
and recordPlayerConnected. Native previously chose during alternating actor
creation and incremented allTime there, changing the map RNG/tile order.
Native creation now stores -1; Drop and Player setup select at their actual
system stages in FIFO order, and player setup increments cumulative allTime.
Explicit tile values remain untouched. Other Population fields/remote events
are not certified by this allTime change. Local-level gating uses the corrected
connected count; no fixed caps or new game rules were introduced.

Source exhausted habitable lookup returns undefined, distinct from request -1.
Native UNDEFINED_TILE=i32::MIN is an internal tag, not a valid grid index; no
random repick for an existing undefined component. Grid position of that tag
is NaN, and NaN position->index stays invalid instead of Rust integer-casting
onto tile0. Source tile/NaN fields compare as JSON null. This models the checked
failure path, not every JS numeric/default-argument/network coercion. A dedicated
case injects undefined into Arrival after construction (distinct from default
function arguments). Native missing/sparse ring defensive skips still differ
from source potential undefined-ring exceptions; empty-ring return is checked.

CPU snowball-spawn-reference.cjs runs unchanged GameMap, Player/Drop/Parachute/
Client/LOD/State and actual THREE, canonical modules and fresh actor pools per
case. Global constructor RNG seeded; GLTF FileLoader remains pending, image/
Canvas IO adapted; collisions/input/network/events are inert. No LocalLevel/AI
or collision step, renderer, browser, GPU or input. Five cases cover original
26-player/25-drop initial cohort, explicit and late cohorts, 30 erosion updates
between creation and processing, existing undefined indices and exhausted rings.
Native actual finish_frame compares both map RNGs, connected count, all actor
tile/arrival/dropped-tick/setup/position/parent records, client marker/camera and
canopy queues/transforms. There are 26 frames/52 before-after snapshots plus
five creation snapshots, 70 fresh actor allocations. No map RNG draw occurs
at creation, matching source; drop draws precede player draws. Carry/landing
through the undefined failure path also matches these bounded snapshots.

All 112 tests pass (101 library + 11 binary). New snowball_state_smoke is an
ordinary CPU main with no window/graphics/assets/audio/input APIs. Running
`cargo run --manifest-path rust/Cargo.toml --release --example snowball_state_smoke
-- rust/data/snowball-spawn-smoke.json` simulates 180 seconds/10800 updates.
It passes, reaches allTime100 with the existing cap, and retains 2500 particle
slots. Report player/drop counts are retained native records, not source active
array sizes. This is native stability coverage, NOT original whole-match parity
or frame-rate/input-latency measurement. Pooled retained actor position/facing/
model state, full global RNG/UUID/appearance, AI, sparse ring exceptions, model
readiness/graphics and full boot/host clocks remain unfinished. No user approval
or README rating change, and no window/focus/input was used.

Release application/capture builds pass; CPU-only state-smoke also builds/runs.
No app window, GPU context, focus change, audio playback or input was used.
