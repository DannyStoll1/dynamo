#! /bin/sh

# export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'
(
    cd "crates/wasm" || exit
    wasm-pack build --release --target web || return 1
)
mv crates/wasm/pkg/dynamo_wasm.js docs
mv crates/wasm/pkg/dynamo_wasm_bg.wasm docs
