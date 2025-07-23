#!/bin/sh

# Format the code
cargo fmt --all || exit 1

# Automatically apply Clippy suggestions
cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged || exit 1

exit 0
