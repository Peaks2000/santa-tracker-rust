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

Reindeer Runner visual/animation parity is the next priority requested by the
user. Boatload remains selected for a later complete source review and port.
