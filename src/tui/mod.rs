pub mod app;
mod handlers;
mod render;

pub use handlers::run_tui;

#[cfg(test)]
mod tests;
