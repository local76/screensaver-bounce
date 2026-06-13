#!/usr/bin/env bash
set -e
cargo build --release
mkdir -p dist/binaries
cp target/release/bounce dist/binaries/
