//! Constants and helpers for locating Taskter's `.taskter` files.

use std::path::Path;

pub const DIR: &str = ".taskter";

pub const BOARD_FILE: &str = ".taskter/board.json";
pub const OKRS_FILE: &str = ".taskter/okrs.json";
pub const LOG_FILE: &str = ".taskter/logs.log";
pub const AGENTS_FILE: &str = ".taskter/agents.json";
pub const DESCRIPTION_FILE: &str = ".taskter/description.md";
pub const EMAIL_CONFIG_FILE: &str = ".taskter/email_config.json";
pub const RUNNING_AGENTS_FILE: &str = ".taskter/running_agents.json";

#[must_use = "use the path to access Taskter data"]
pub fn dir() -> &'static Path {
    Path::new(DIR)
}

#[must_use = "use the path to access Taskter data"]
pub fn board_path() -> &'static Path {
    Path::new(BOARD_FILE)
}

#[must_use = "use the path to access Taskter data"]
pub fn okrs_path() -> &'static Path {
    Path::new(OKRS_FILE)
}

#[must_use = "use the path to access Taskter data"]
pub fn log_path() -> &'static Path {
    Path::new(LOG_FILE)
}

#[must_use = "use the path to access Taskter data"]
pub fn agents_path() -> &'static Path {
    Path::new(AGENTS_FILE)
}

#[must_use = "use the path to access Taskter data"]
pub fn description_path() -> &'static Path {
    Path::new(DESCRIPTION_FILE)
}

#[must_use = "use the path to access Taskter data"]
pub fn email_config_path() -> &'static Path {
    Path::new(EMAIL_CONFIG_FILE)
}

#[must_use = "use the path to access Taskter data"]
pub fn running_agents_path() -> &'static Path {
    Path::new(RUNNING_AGENTS_FILE)
}
