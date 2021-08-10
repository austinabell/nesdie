#!/bin/bash
set -e

TARGET="${CARGO_TARGET_DIR:-target}"

RUSTFLAGS='-C link-arg=-s' cargo +nightly build --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/multisig.wasm ./res/
