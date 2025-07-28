use std::fs;

use crate::config;

pub fn set(description: &str) -> anyhow::Result<()> {
    fs::write(config::description_path(), description)?;
    println!("Project description updated successfully.");
    Ok(())
}
