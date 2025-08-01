//! Terminal user interface for managing tasks interactively.

pub mod app;
mod handlers;
mod render;

pub use handlers::run_tui;
