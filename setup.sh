#!/bin/bash

sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev

cargo install tauri-cli --version "^2.0.0" --locked

pushd src-tauri/
cargo update
popd

cargo tauri dev
