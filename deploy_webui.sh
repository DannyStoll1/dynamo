#! /bin/sh

(
    cd "crates/wasm" || exit
    wasm-pack build --target web || return 1
)
mv crates/wasm/pkg/dynamo_wasm.js docs
mv crates/wasm/pkg/dynamo_wasm_bg.wasm docs
