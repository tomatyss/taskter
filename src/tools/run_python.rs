use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Command;

use crate::agent::FunctionDeclaration;

const DECL_JSON: &str = include_str!("../../tools/run_python.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid run_python.json")
}

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
