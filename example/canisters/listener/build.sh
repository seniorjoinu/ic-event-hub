#!/usr/bin/env bash

SCRIPT=$(readlink -f "$0")
SCRIPTPATH=$(dirname "$SCRIPT")
cd "$SCRIPTPATH" || exit

cargo build --target wasm32-unknown-unknown --release --package listener && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/listener.wasm -o ./target/wasm32-unknown-unknown/release/listener-opt.wasm
