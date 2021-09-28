#!/bin/bash
set -e

TARGET="${CARGO_TARGET_DIR:-target}"

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/smol_ft.wasm ./res/
