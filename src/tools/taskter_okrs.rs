use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Command;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/taskter_okrs.json");

fn taskter_bin() -> std::path::PathBuf {
    std::env::var("TASKTER_BIN")
        .or_else(|_| std::env::var("CARGO_BIN_EXE_taskter"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| "taskter".into())
}

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid taskter_okrs.json")
}

/// Invokes the `taskter okrs` subcommand.
///
/// # Errors
///
/// Returns an error if the `args` array is missing, contains non-string
/// arguments, or if the command fails to run or exits with a non-zero status.
pub fn execute(args: &Value) -> Result<String> {
    let arg_list = args["args"]
        .as_array()
        .ok_or_else(|| anyhow!("args missing"))?;
    let mut cmd = Command::new(taskter_bin());
    cmd.arg("okrs");
    for a in arg_list {
        if let Some(s) = a.as_str() {
            cmd.arg(s);
        } else {
            return Err(anyhow!("args must be strings"));
        }
    }
    let output = cmd.output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow!(
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "taskter_okrs",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
