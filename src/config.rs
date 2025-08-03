/// Constants for paths inside the `.taskter` directory.
use std::path::Path;

pub const DIR: &str = ".taskter";

pub const BOARD_FILE: &str = ".taskter/board.json";
pub const OKRS_FILE: &str = ".taskter/okrs.json";
pub const LOG_FILE: &str = ".taskter/logs.log";
pub const AGENTS_FILE: &str = ".taskter/agents.json";
pub const DESCRIPTION_FILE: &str = ".taskter/description.md";
pub const EMAIL_CONFIG_FILE: &str = ".taskter/email_config.json";

#[must_use]
pub fn dir() -> &'static Path {
    Path::new(DIR)
}

#[must_use]
pub fn board_path() -> &'static Path {
    Path::new(BOARD_FILE)
}

#[must_use]
pub fn okrs_path() -> &'static Path {
    Path::new(OKRS_FILE)
}

#[must_use]
pub fn log_path() -> &'static Path {
    Path::new(LOG_FILE)
}

#[must_use]
pub fn agents_path() -> &'static Path {
    Path::new(AGENTS_FILE)
}

#[must_use]
pub fn description_path() -> &'static Path {
    Path::new(DESCRIPTION_FILE)
}

#[must_use]
pub fn email_config_path() -> &'static Path {
    Path::new(EMAIL_CONFIG_FILE)
}
