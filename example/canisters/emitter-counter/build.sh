#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release --package emitter-counter && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/emitter_counter.wasm -o ./target/wasm32-unknown-unknown/release/emitter-counter-opt.wasm
