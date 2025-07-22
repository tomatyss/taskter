use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;

const DECL_JSON: &str = include_str!("../../tools/file_ops.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid file_ops.json")
}

/// Performs basic file operations in the project directory.
pub fn execute(args: &Value) -> Result<String> {
    let action = args["action"]
        .as_str()
        .ok_or_else(|| anyhow!("action missing"))?;
    let path = args["path"]
        .as_str()
        .ok_or_else(|| anyhow!("path missing"))?;
    match action {
        "read" => {
            let content = fs::read_to_string(path)?;
            Ok(content)
        }
        "write" => {
            let text = args["text"]
                .as_str()
                .ok_or_else(|| anyhow!("text missing"))?;
            fs::write(path, text)?;
            Ok("File written".to_string())
        }
        "search" => {
            let query = args["text"]
                .as_str()
                .ok_or_else(|| anyhow!("text missing"))?;
            let content = fs::read_to_string(path)?;
            let mut matches = Vec::new();
            for (i, line) in content.lines().enumerate() {
                if line.contains(query) {
                    matches.push(format!("{}: {}", i + 1, line));
                }
            }
            Ok(matches.join("\n"))
        }
        _ => Err(anyhow!("unknown action")),
    }
}

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "file_ops",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
