#! /bin/sh

wasm-pack build --target web || return 1
mv pkg/dynamo_wasm.js ../docs
mv pkg/dynamo_wasm_bg.wasm ../docs

