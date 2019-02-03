#! /bin/bash

set -ev

if [ "$TARGET_WASM" == "0" ]; then
    cargo test
else
    rustup target add wasm32-unknown-unknown
    cargo install -f cargo-web
    cargo web build --target wasm32-unknown-unknown
fi