#! /bin/bash

set -e

here="$(dirname "$0")"
# export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'
(
    cd "$here/crates/wasm" || exit 1
    wasm-pack build --release --target web || return 1
)
mv -v "$here/crates/wasm/pkg/dynamo_wasm.js" "$here/docs"
mv -v "$here/crates/wasm/pkg/dynamo_wasm_bg.wasm" "$here/docs"
