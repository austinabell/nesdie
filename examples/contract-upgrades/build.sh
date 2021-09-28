#!/bin/bash
set -e

TARGET="${CARGO_TARGET_DIR:-target}"

cargo +nightly-2021-08-27 build --all --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/upgrade_a.wasm ./res/
cp $TARGET/wasm32-unknown-unknown/release/upgrade_b.wasm ./res/
