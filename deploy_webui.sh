#! /bin/bash

set -e

here="$(dirname "$0")"

(
    cd "$here/crates/wasm" || exit 1
    # Size-optimized wasm build. opt-level=z + codegen-units=1 + strip
    # are applied via env overrides so only this wasm build is affected
    # (the native `release` profile keeps opt-level 3 for fractal-compute
    # throughput). The wasm-opt size pass (-Oz --all-features) is set in
    # Cargo.toml package metadata; wasm-pack honors it for `--release`.
    CARGO_PROFILE_RELEASE_OPT_LEVEL=z \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    CARGO_PROFILE_RELEASE_STRIP=true \
        wasm-pack build --release --target web
)

mv -v "$here/crates/wasm/pkg/dynamo_wasm.js" "$here/docs"
mv -v "$here/crates/wasm/pkg/dynamo_wasm_bg.wasm" "$here/docs"
