# Use bash with strict flags for consistent behavior
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Serve documentation locally with mdBook
# Usage:
#   - default: `just docs`
#   - override: `just docs host=0.0.0.0 port=4000`
docs host="127.0.0.1" port="3000":
    mdbook serve docs -n {{host}} -p {{port}} --open

# Build the static documentation site into `docs/book`
docs-build:
    mdbook build docs

# Install mdBook (requires Rust toolchain)
install-mdbook:
    cargo install mdbook
