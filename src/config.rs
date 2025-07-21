use std::path::{Path, PathBuf};

/// Root directory for all Taskter state.
pub const TASKTER_DIR: &str = ".taskter";

/// Return the path to a file inside the `.taskter` directory.
pub fn path(file: &str) -> PathBuf {
    Path::new(TASKTER_DIR).join(file)
}

pub fn board_file() -> PathBuf {
    path("board.json")
}

pub fn okrs_file() -> PathBuf {
    path("okrs.json")
}

pub fn agents_file() -> PathBuf {
    path("agents.json")
}

pub fn logs_file() -> PathBuf {
    path("logs.log")
}

pub fn description_file() -> PathBuf {
    path("description.md")
}

pub fn email_config_file() -> PathBuf {
    path("email_config.json")
}
