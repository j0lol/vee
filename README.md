# vee

Research library for replicating Mii functionality from decompilations in a more modern language & runtime. Ideally, this library will replicate functionality accurately from these targets:
- Cafe (WUP)
- Centrair (CTR) <sup><sub>Why did they name it after an airport‽</sub></sup>
- Nx (NX)

Targets not currently aimed for:
- Revolution (RFL)
- Nitro (NTR)


Features are limited, currently.


## bevy_viewer

Setup:
- Obtain `ShapeMid.dat` and place in `./vee/`
- `cargo run`

Web build:
- Install [`wasm-server-runner`](https://github.com/jakobhellermann/wasm-server-runner)
- `cargo run --target wasm32-unknown-unknown`

## Acknowledgements
- Arian Kordi @ariankordi for help and guidance through terse decompiled C++
  - https://github.com/ariankordi/FFL-Testing
- @aboood40091 for [FFL Decomp](https://github.com/aboood40091/ffl)
- Petari team @SMGCommunity for the [RFL Decomp](https://github.com/SMGCommunity/Petari/tree/master/src/RVLFaceLib)
- Probably other people too
