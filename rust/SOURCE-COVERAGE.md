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
