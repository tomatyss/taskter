//! Taskter library interface exposing the core modules so they can be
//! reused from integration tests and (potentially) other binaries.

#![allow(
    clippy::pedantic,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::used_underscore_binding,
    clippy::if_not_else,
    clippy::single_match_else
)]

pub mod agent;
pub mod cli;
pub mod config;
pub mod scheduler;
pub mod store;
pub mod tools;

pub use cli::{Cli, Commands, ShowCommands};

// The TUI heavily depends on a terminal backend which is not easily testable in
// automated environments. We expose it behind the `tui` feature so that normal
// consumers – including the main binary – can still use it without imposing
// unnecessary dependencies on test builds.
#[cfg(feature = "tui")]
pub mod tui;
