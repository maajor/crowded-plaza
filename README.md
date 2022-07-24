# Crowded Plaza

Inspired by [Crowded City](https://crowdedcity.io)  
An experimental game with [bevy 0.7.0](https://github.com/bevyengine/bevy)

## Dev Run

`cargo run`

## Build Wasm

```
cargo build --profile release-wasm --target wasm32-unknown-unknown
wasm-bindgen --out-name game --out-dir target/wasm --target web target/wasm32-unknown-unknown/release-wasm/crowded-plaza.wasm
```

Size is around

- 7.7 MB Uncompressed
- 1.9 MB Gz Compressed

## Start Game in Browser

After build wasm, run

```
python -m http.server
```

then visit `http://localhost:8000`

# Profiling and Benchmark

Conclusion

- Bevy ECS is around 10x slower than Unity ECS
- Size of Bevy minimal wasm build is similar to Unity one
- Size of Bevy ecs wasm build is 2x smaller than Unity one

For more details, see [Benchmark](./BENCHMARK.md)
