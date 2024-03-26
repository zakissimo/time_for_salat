#!/usr/bin/env bash

set -xe

cargo build --release
cp ./target/release/"$(basename "$PWD")" "$HOME"/.local/bin
