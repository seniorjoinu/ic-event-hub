#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release --package listener-counter-1 && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/listener_counter_1.wasm -o ./target/wasm32-unknown-unknown/release/listener-counter-1-opt.wasm
