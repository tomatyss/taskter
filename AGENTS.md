# Repository Guidelines

## Project Structure & Module Organization
Taskter follows standard Cargo layout. Core logic lives in `src/` with domain modules such as `commands/`, `providers/`, `scheduler.rs`, and `tui/` for the interactive board. CLI entry points are `src/main.rs` and `src/cli.rs`, while reusable primitives sit in `src/lib.rs`. Integration tests live under `tests/`, and documentation sources are in `docs/src/`. Helper scripts reside in `scripts/`, packaging artifacts in `packaging/`, and tooling utilities in `tools/`. Keep assets like `tui_example.png` in the repo root unless a feature-specific subdirectory is warranted.

## Build, Test, and Development Commands
- `cargo build` – compile the binary with all default features.
- `cargo test` – run unit and integration tests; add `-- --ignored` for slow suites.
- `cargo fmt --all` and `cargo clippy --all-targets --all-features -D warnings -D clippy::pedantic` – enforce formatting and lint gates locally.
- `./scripts/precommit.sh` – one-stop formatter, linter, and test runner used before merging.
- `./scripts/fix_lints.sh` – auto-apply `rustfmt` and Clippy suggestions.
- `just docs` – serve the mdBook at `http://127.0.0.1:3000`; requires `mdbook` installed.

## Coding Style & Naming Conventions
Adhere to `rustfmt` defaults (four-space indentation, trailing commas, etc.). Favor `snake_case` for modules and functions, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Group related modules with subdirectories (e.g., `src/commands/` for CLI verbs). Clippy pedantic warnings are treated as errors, so resolve lints instead of suppressing them; if a `#[allow]` is required, document the rationale inline.

## Testing Guidelines
Use standard Rust `#[test]` functions for unit coverage near implementation files, and add scenario tests under `tests/` for CLI and scheduler flows. Mirror behavior names in test identifiers (`handles_missing_board_dir`, `executes_cron_jobs`). No formal coverage threshold exists, but new features should include regression cases and update fixtures in `.taskter/` when behavior changes. When altering async or scheduler code, exercise both happy-path and failure modes.

## Commit & Pull Request Guidelines
Follow the prevailing Conventional Commit tone (`feat(openai): …`, `fix(commands): …`). Write concise, present-tense summaries and note breaking changes with `!` when applicable. Pull requests should include: a short problem/solution description, links to related issues, and CLI output or screenshots for TUI-affecting work (`taskter board` capture). Ensure CI-relevant scripts (`./scripts/precommit.sh`) pass before requesting review, and mention any follow-up tasks explicitly within the PR description.

## Configuration & Secrets
Runtime secrets (e.g., `GEMINI_API_KEY`, `OPENAI_API_KEY`) are loaded from the environment; never commit them or the generated `.taskter/` board data. Sample scripts (`scripts/setup_example_project.sh`) reset local state—avoid running them on real user directories. Document any new configuration keys in `docs/src/` and update `README.md` if usage instructions change.
