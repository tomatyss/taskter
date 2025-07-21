#!/bin/sh

# Format code
cargo fmt --all

# Fix clippy lints
cargo clippy --fix --all-targets --all-features --allow-dirty -- -D warnings
