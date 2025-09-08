#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::manual_let_else,
    clippy::single_match_else,
    clippy::used_underscore_binding,
    clippy::ignored_unit_patterns,
    clippy::redundant_closure_for_method_calls,
    clippy::implicit_hasher,
    clippy::map_unwrap_or,
    clippy::doc_markdown,
    clippy::stable_sort_primitive,
    clippy::cast_possible_truncation,
    clippy::semicolon_if_nothing_returned,
    clippy::if_not_else,
    clippy::assigning_clones,
    clippy::match_wildcard_for_single_variants,
    clippy::unused_async,
    clippy::non_std_lazy_statics
)]

//! Taskter library interface exposing the core modules so they can be
//! reused from integration tests and (potentially) other binaries.

pub mod agent;
pub mod cli;
pub mod commands;
pub mod config;
pub mod scheduler;
pub mod store;
pub mod tools;
pub mod providers;

pub use cli::{Cli, Commands, ShowCommands};

// The TUI heavily depends on a terminal backend which is not easily testable in
// automated environments. We expose it behind the `tui` feature so that normal
// consumers – including the main binary – can still use it without imposing
// unnecessary dependencies on test builds.
#[cfg(feature = "tui")]
pub mod tui;
