use std::{fs, path::Path};

use crate::config;

fn ensure_file(path: &Path, contents: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        fs::write(path, contents)?;
    }
    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    let data_dir = config::dir()?;
    if data_dir.exists() {
        println!("Taskter board already initialized.");
    } else {
        fs::create_dir_all(&data_dir)?;
        ensure_file(&config::description_path()?, "# Project Description")?;
        ensure_file(&config::okrs_path()?, "[]")?;
        ensure_file(&config::log_path()?, "")?;
        ensure_file(&config::board_path()?, r#"{ "tasks": [] }"#)?;
        ensure_file(&config::agents_path()?, "[]")?;
        println!("Taskter board initialized.");
    }
    Ok(())
}
