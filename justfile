# Use bash with strict flags for consistent behavior
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Show all available recipes when running `just` with no arguments
default:
    @just --list

# -----------------------------
# Build & Test
# -----------------------------

# Compile the project; pass extra args like `just build --release`
build *args:
    cargo build {{args}}

# Typecheck without building artifacts
check *args:
    cargo check --all-targets --all-features {{args}}

# Run formatter and verify no diff
fmt-check:
    cargo fmt --all -- --check

# Format the codebase in-place
fmt:
    cargo fmt --all

# Lint with Clippy pedantic flags
lint:
    cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic

# Run the full test suite; forward extra cargo args if needed
test *args:
    cargo test {{args}}

# Execute ignored tests (e.g., `just test-ignored scheduler`)
test-ignored *args:
    cargo test {{args}} -- --ignored

# Convenience target for the pre-commit script
precommit:
    ./scripts/precommit.sh

# Auto-fix formatting and Clippy suggestions
fix-lints:
    ./scripts/fix_lints.sh

# Run the binary and forward CLI arguments: `just run -- task list`
run *args:
    cargo run -- {{args}}

# Remove build artifacts
clean:
    cargo clean

# -----------------------------
# Documentation
# -----------------------------

# Serve documentation locally with mdBook (`just docs host=0.0.0.0 port=4000`)
docs host="127.0.0.1" port="3000":
    mdbook serve docs -n {{host}} -p {{port}} --open

# Build the static documentation site into `docs/book`
docs-build:
    mdbook build docs

# Install mdBook (requires Rust toolchain)
install-mdbook:
    cargo install mdbook

# -----------------------------
# Project Helpers
# -----------------------------

# Rebuild the example board in `.taskter`
setup-example-project:
    ./scripts/setup_example_project.sh

# Populate the default multi-agent roster (requires initialized board)
setup-agent-roster:
    ./scripts/setup_agent_roster.sh
