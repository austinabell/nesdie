#!/bin/bash
set -e

TARGET="${CARGO_TARGET_DIR:-target}"

RUSTFLAGS='-C link-arg=-s' cargo +nightly-2021-08-27 build --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/raw_contract.wasm ./res/
