# Crowded Plaza

Inspired by [Crowded City](https://crowdedcity.io)  
An experimental game with [bevy 0.7.0](https://github.com/bevyengine/bevy)

## Dev Run

`cargo run`

## Build Wasm

```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-name mgame --out-dir target/wasm --target web target/wasm32-unknown-unknown/release/mgame.wasm
```

## Start Game in Browser

After build wasm, run

```
python -m http.server
```

then visit `http://localhost:8000`
