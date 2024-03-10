#!/usr/bin/env bash

set -xe

cargo build --release
cp ./target/release/time_for_salat "$HOME"/.local/bin
