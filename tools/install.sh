#!/bin/sh
set -e
cargo build --release
mv target/release/rustpotter-cli /usr/local/bin/