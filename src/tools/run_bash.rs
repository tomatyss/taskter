use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Command;

use crate::agent::FunctionDeclaration;

const DECL_JSON: &str = include_str!("../../tools/run_bash.json");

/// Return the declaration for executing shell commands.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid run_bash.json")
}

/// Run a bash command locally and return its output.
pub fn execute(args: &Value) -> Result<String> {
    let command = args["command"]
        .as_str()
        .ok_or_else(|| anyhow!("command missing"))?;

    let output = Command::new("sh").arg("-c").arg(command).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow!(
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
