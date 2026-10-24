# Santa Tracker Rust rewrite (in progress)

The reason "rust" is in its own folder is because I was working with the java script with the rust folder inside it but only uploaded the rust files so thats why it is that way, sorry for how messy it is!

Using the javascript code of Google Santa Tracker I remade the logic function by function in rust delivering an almost 1:1 experience!


![alt text](https://i.postimg.cc/qv3kdFRr/Screenshot-From-2026-10-23-23-36-53.png)

## Scenes

| Name | State |
| --- | --- |
| Village / Home | Playable — partial fidelity |
| Reindeer Runner | Playable — partial fidelity |
| Elf Jetpack | Playable — partial fidelity |
| Santa Selfie | Playable — partial fidelity |
| Elf Ski | Playable — partial fidelity |
| Present Drop | Playable — partial fidelity |
| Gift Slingshot | Playable — partial fidelity |
| Elf Maker | Playable — partial fidelity |
| Elf Jamband | Playable — partial fidelity |
| Elf Glider | Playable — partial fidelity |
| Christmas Tracking | Not implemented |

# Running locally

or grab from releases

```sh
./rust/build.sh
./rust/run.sh --scene jetpack
./rust/run.sh --scene santaselfie
```

