#!/bin/bash

pushd src-tauri/ 
cargo update 
popd 

yarn install 

cargo tauri dev ;
