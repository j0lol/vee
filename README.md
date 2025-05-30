# VFL — Vee Face Library

[![Coverage Status](https://coveralls.io/repos/github/j0lol/vee/badge.svg?branch=main)](https://coveralls.io/github/j0lol/vee?branch=main)

Research library for replicating Mii functionality from decompilations in a more modern language and runtime.

## Targets

Ideally, this library will replicate functionality accurately from these targets:

- Cafe (WUP)
- Centrair (CTR) <sup><sub>Why did they name it after an airport‽</sub></sup>
- Nx

Not currently targeted:

- Revolution (RVL)
- Nitro (NTR)
- Any other target (Miitomo, Mii Studio, etc.)

## Libraries

This project currently has the libraries:

- `vfl` (Parent library)
- `vee_parse`
- `vee_resources`
- `vee_models`
- `vee_wgpu`

## Binaries

This project currently has three binaries:

- `vfl-cli` — Debug tool for quick interfacing with the library.
- `lightweight_viewer` — Basic test renderer, orbits a few test characters.
- `bevy_viewer` — Example of using this library in Bevy.

`bevy_viewer` is currently out of the workspace tree, so it has to be run separately.

### Running

- Dump Nx shape and texture resources, place in `./resources_here`
    - Currently, `vee` only supports `NXTextureMidSRGB.dat` and `ShapeMid.dat`
- `lightweight_viewer`
    - `cargo run --bin lightweight_viewer`
- `vfl-cli`
    - `cargo run --bin vfl-cli`
- `bevy_viewer`
    - `cd crates/bevy_viewer && cargo run`
    - Running `bevy_viewer` in the browser:
        - Install [`wasm-server-runner`](https://github.com/jakobhellermann/wasm-server-runner)
        - `cd crates/bevy_viewer && cargo run --target wasm32-unknown-unknown`

## Acknowledgements

- `@ariankordi` for help and guidance through FFL and RFL source
    - Made a working PC renderer from FFL decompilation: https://github.com/ariankordi/FFL-Testing
- `@aboood40091` for [FFL Decomp](https://github.com/aboood40091/ffl)
- `@SMGCommunity` for [RFL Decomp](https://github.com/SMGCommunity/Petari/tree/master/src/RVLFaceLib)
- Probably other people too
