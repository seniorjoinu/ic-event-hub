#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release --package emitter && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/emitter.wasm -o ./target/wasm32-unknown-unknown/release/emitter-opt.wasm
