use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Command;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/run_python.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid run_python.json")
}

/// Executes a Python snippet using the system `python3`.
///
/// # Errors
///
/// Returns an error if the `code` argument is missing, if `python3` cannot be
/// executed, or if the script exits with a non-zero status.
pub fn execute(args: &Value) -> Result<String> {
    let code = args["code"]
        .as_str()
        .ok_or_else(|| anyhow!("code missing"))?;

    let output = Command::new("python3").arg("-c").arg(code).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow!(
            "Python execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "run_python",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
