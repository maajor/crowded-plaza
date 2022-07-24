# minimal bevy wasm

minimal dependency to make 3D game, with features:

- `bevy_core_pipeline`, `bevy_render` and `bevy_pbr`, for rendering
- `bevy_text`, `bevy_ui` and `bevy_sprite`, for ui

## Build

```sh
# after change directory to this folder
cargo build --profile release-wasm --target wasm32-unknown-unknown

wasm-bindgen --out-name game --out-dir target/wasm --target web target/wasm32-unknown-unknown/release-wasm/bevy-min-wasm.wasm
```

## Build Wasm Size

- 7.93 MB uncompressed
- 1.97 MB gzip compressed

While according to another report, [minimal-bevy-wasm](https://github.com/anlumo/minimal-bevy-wasm), where he stripped more package and get a 3.5MB wasm file.

## Screenshot

![cube](../../imgs/bevy-min-wasm.jpg)
