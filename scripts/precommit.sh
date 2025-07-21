#!/bin/sh

# Format code
cargo fmt --all -- --check || exit 1

# Lint code with clippy
cargo clippy --all-targets --all-features -- -D warnings || exit 1

# Run tests with all features
cargo test --all-features || exit 1

exit 0
