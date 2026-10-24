# Santa Tracker Rust rewrite (in progress)

The reason "rust" is in its own folder is because I was working with the java script with the rust folder inside it but only uploaded the rust files so thats why it is that way, sorry for how messy it is!

Using the javascript code of Google Santa Tracker I remade the logic function by function in rust delivering an almost 1:1 experience!


![alt text](https://i.postimg.cc/qv3kdFRr/Screenshot-From-2026-10-23-23-36-53.png)

## Scenes
1:1 means every animation image and behaviour(e.g physics in Elf Jetpack) is the exact same as the website.
Almost 1:1 means so close to the website it is unnoticeable and does not have any effect on gameplay.
Very Similar means it plays almost exactly like the website but small things could be a little off like the size of an elf (not a real example as far as I am aware.)

| Name | State |
| --- | --- |
| Village / Home | 1:1 |
| Reindeer Runner | Almost 1:1 |
| Elf Jetpack | 1:1 |
| Santa Selfie | 1:1 |
| Elf Ski | Almost 1:1 |
| Present Drop | Very similar |
| Gift Slingshot | Very similar |
| Elf Maker | Very similar |
| Elf Jamband | Very similar |
| Elf Glider | Very similar |
| Penguin Dash | Very similar |
| Christmas Tracking | Unplayable |

# Running locally

or grab from releases

```sh
./rust/build.sh
./rust/run.sh --scene jetpack
./rust/run.sh --scene santaselfie
```

