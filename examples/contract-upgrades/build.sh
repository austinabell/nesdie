#!/bin/bash
set -e

TARGET="${CARGO_TARGET_DIR:-target}"

cargo build --all --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/upgrade_a.wasm ./res/
cp $TARGET/wasm32-unknown-unknown/release/upgrade_b.wasm ./res/
