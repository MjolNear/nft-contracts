#!/bin/bash
set -e
rm -rf out
RUSTFLAGS='-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
mkdir out
cp target/wasm32-unknown-unknown/release/*.wasm ./out/main.wasm