#! /bin/sh

wasm-pack build --target web | exit
mv pkg/fractal_wasm.js ../docs
mv pkg/fractal_wasm_bg.wasm ../docs

