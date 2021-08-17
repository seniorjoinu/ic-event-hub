#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release --package listener-counter-2 && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/listener_counter_2.wasm -o ./target/wasm32-unknown-unknown/release/listener-counter-2-opt.wasm
