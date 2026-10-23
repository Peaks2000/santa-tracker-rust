# Santa Tracker Rust rewrite (in progress)

The reason "rust" is in its own folder is because I was working with the java script with the rust folder inside it but only uploaded the rust files so thats why it is that way, sorry for how messy it is!

Using the javascript code of Google Santa Tracker I remade the logic function by function in rust delivering an almost 1:1 experience!


![alt text](https://i.postimg.cc/qv3kdFRr/Screenshot-From-2026-10-23-23-36-53.png)

## Faithfulness to the original

All seven native scenes are **partial**; none is certified completely 1:1. This table records checked behavior rather than estimated percentages. See the [detailed report](rust/PARITY-REPORT.txt) and [source comparison evidence](rust/SOURCE-COVERAGE.md). Add your verdict and notes in [PARITY-STATUS.json](rust/PARITY-STATUS.json).

| Game / scene | Implemented and checked | Still needs work |
| --- | --- | --- |
| Village / home | Original artwork and native Lottie animation, navigation, countdown, responsive cards | Loading/frame pacing, remaining interactions and visual comparisons |
| Reindeer Runner | Source simulation scenarios, jump/slide, level transitions, responsive layout, queued tutorial | More visual/effect phases and complete audio/touch fidelity |
| Elf Jetpack | Source simulation scenarios, movement/collisions, original sprites, queued tutorial, shared overlays | Remaining image/font differences, full audio/touch and transition checks |
| Santa Selfie | Cloth/tool source scenarios, paused blink timing, eye artwork, mobile tray and background proportions | Small raster differences, all tool/effect phases, original URL sharing and performance review |
| Elf Ski | Native skiing/terrain, source-state comparisons, original tutorial video frames | Touch pressure, complete music and further visual checks |
| Present Drop | Native movement/drops/collisions, source-state comparisons and queued tutorial | Flag typography, full touch/audio and more visual phases |
| Gift Slingshot | Seven source simulation scenarios, charge/throw, boats/icebergs, ten-level finish and queued tutorial | Score pop-ups/effects, more viewport checks and full music/mixer fidelity |
| Christmas tracking | Route calculations and delivery-only visibility tested against 3,020 source states | Native map and tracking screen are **not implemented** |

Shared pause/game-over geometry is checked against 18 original browser cases. Font rendering, card loading/fades and all animation phases remain under review. Other source games have no native implementation yet.



# Running locally

or grab from releases

```sh
./rust/build.sh
./rust/run.sh --scene jetpack
./rust/run.sh --scene santaselfie
```

