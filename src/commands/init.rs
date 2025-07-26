use std::fs;

use crate::config;

pub fn run() -> anyhow::Result<()> {
    let path = config::dir();
    if path.exists() {
        println!("Taskter board already initialized.");
    } else {
        fs::create_dir(path)?;
        fs::write(path.join("description.md"), "# Project Description")?;
        fs::write(path.join("okrs.json"), "[]")?;
        fs::write(path.join("logs.log"), "")?;
        fs::write(path.join("board.json"), r#"{ "tasks": [] }"#)?;
        fs::write(path.join("agents.json"), "[]")?;
        println!("Taskter board initialized.");
    }
    Ok(())
}
