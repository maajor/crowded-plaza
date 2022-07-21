# first-bevy-game

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-name mgame --out-dir target/wasm --target web target/wasm32-unknown-unknown/release/mgame.wasm

https://github.com/anlumo/minimal-bevy-wasm

https://bevy-cheatbook.github.io/introduction.html

todo:

- game over
- restart game
- city blockout model
