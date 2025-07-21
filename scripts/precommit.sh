#!/bin/sh

# Autofix linter issues
./scripts/autofix.sh

# Format code
cargo fmt --all -- --check || exit 1

# Lint code with clippy
cargo clippy --all-targets --all-features -- -D warnings || exit 1

# Run tests
cargo test || exit 1

exit 0
