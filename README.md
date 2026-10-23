# Santa Tracker Rust rewrite (in progress)

The reason "rust" is in its own folder is because I was working with the java script with the rust folder inside it but only uploaded the rust files so thats why it is that way, sorry for how messy it is!

Using the javascript code of Google Santa Tracker I remade the logic function by function in rust delivering an almost 1:1 experience!


![alt text](https://i.postimg.cc/qv3kdFRr/Screenshot-From-2026-10-23-23-36-53.png)

## Working scenes and faithfulness

Nine scenes currently work in the native Rust application. **None is fully verified 1:1.** “Close in tested behavior” means the checked simulation scenarios match the original; it does not mean every animation, sound or screen is identical.

- **Village / home (`modvil`) — Partially faithful.** Original artwork, native Lottie animation, navigation, Christmas countdown and responsive cards work. Loading hitches, frame pacing, remaining interactions and some card/loader details still need work.
- **Reindeer Runner (`runner`) — Close in tested gameplay; visual parity is partial.** Jumping, sliding, level transitions, responsive layout and tutorial prompts are implemented and checked against the original. More animation/effect phases, audio and touch behavior need comparison.
- **Elf Jetpack (`jetpack`) — Close in tested gameplay; visual parity is partial.** Movement, collisions, original sprites, tutorial prompts and pause/game-over screens work. Small image/font differences, complete audio/touch behavior and transition phases remain under review.
- **Santa Selfie (`santaselfie`) — Close in tested cloth/tool behavior and selected layouts.** Beard physics, tools, paused blinking, eye artwork, mobile tool tray and background proportions have source comparisons. Small rendering differences and further tool/effect phases remain; native file export differs from the original URL-sharing flow.
- **Elf Ski (`elfski`) — Partially faithful, playable.** Native skiing, terrain, source-state comparisons and the original tutorial video frames are implemented. Touch pressure, complete music and broader visual checks remain unfinished.
- **Present Drop (`presentdrop`) — Close in tested gameplay; presentation is partial.** Movement, present drops, collisions and queued tutorial prompts work with source-state comparisons. Flag typography, full touch/audio behavior and more visual phases still need work.
- **Gift Slingshot (`boatload`) — Close in tested gameplay and one idle visual comparison.** Charging/throwing, boats, icebergs, scoring, all ten levels and tutorial prompts work. Seven original simulation scenarios match; score pop-ups, effect phases, other viewport sizes and full music/mixer behavior need further work.

- **Elf Maker (`elfmaker`) — Close in tested geometry and control logic; presentation is partial.** All character choices, colours, backgrounds, randomization, arm moves, blinking, photo export and saved-state restore work. 21 control states and 35 geometry cases match source; photo transitions, sharing, touch/accessibility and further viewport checks remain. The software-shadow bottleneck is fixed: the tested idle build now averages about 60 fps.
- **Elf Jamband (`jamband`) — Close in tested band logic and character keyframes; full-scene parity is partial.** Ten instruments, six stages, both musical patterns, synchronized starts, previews, swapping, randomization, drawer and lights work. 19 band-state cases and 1,568 animation samples match source. Complete lighting/drag transitions, audio mixing, responsive/touch behavior and website link sharing still need work.

Shared pause/game-over geometry is checked against 18 original browser cases. Font rendering, card loading/fades and all animation phases remain under review. These descriptions are evidence-based assessments, not percentage scores or user approval.

**Christmas tracking is not a working scene yet.** Route calculations and delivery-only visibility match 3,020 source states, but the native map and tracking screen are not implemented. Other source scenes have no native implementation yet.

See the [detailed report](rust/PARITY-REPORT.txt) and [source comparison evidence](rust/SOURCE-COVERAGE.md). Add your own verdict and notes in [PARITY-STATUS.json](rust/PARITY-STATUS.json).

# Running locally

or grab from releases

```sh
./rust/build.sh
./rust/run.sh --scene jetpack
./rust/run.sh --scene santaselfie
```

