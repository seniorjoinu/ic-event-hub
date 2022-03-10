#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release --package listener && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/listener.wasm -o ./target/wasm32-unknown-unknown/release/listener-opt.wasm
