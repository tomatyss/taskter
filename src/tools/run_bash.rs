use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Command;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/run_bash.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid run_bash.json")
}

/// Runs a shell command using `sh -c`.
///
/// # Errors
///
/// Returns an error if the `command` argument is missing, if the command fails
/// to execute, or if it exits with a non-zero status.
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

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "run_bash",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
