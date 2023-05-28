#! /bin/sh

wasm-pack build --target web || return 1
mv pkg/fractal_wasm.js ../docs
mv pkg/fractal_wasm_bg.wasm ../docs

