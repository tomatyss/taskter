use std::{fs, path::Path};

use crate::config;

fn ensure_file(path: &Path, contents: &str) -> anyhow::Result<bool> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !path.exists() {
        fs::write(path, contents)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn run() -> anyhow::Result<()> {
    let data_dir = config::dir()?;
    fs::create_dir_all(&data_dir)?;
    let mut created = false;
    created = ensure_file(&config::description_path()?, "# Project Description")? || created;
    created = ensure_file(&config::okrs_path()?, "[]")? || created;
    created = ensure_file(&config::log_path()?, "")? || created;
    created = ensure_file(&config::board_path()?, r#"{ "tasks": [] }"#)? || created;
    created = ensure_file(&config::agents_path()?, "[]")? || created;
    if created {
        println!("Taskter board initialized.");
    } else {
        println!("Taskter board already initialized.");
    }
    Ok(())
}
