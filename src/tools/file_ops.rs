use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/file_ops.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid file_ops.json")
}

/// Perform file operations in the project directory.
pub fn execute(args: &Value) -> Result<String> {
    let action = args["action"]
        .as_str()
        .ok_or_else(|| anyhow!("action missing"))?;
    let path = args["path"].as_str().ok_or_else(|| anyhow!("path missing"))?;

    match action {
        "create" => {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            fs::write(path, content)?;
            Ok(format!("Created {path}"))
        }
        "read" => {
            let content = fs::read_to_string(path)?;
            Ok(content)
        }
        "search" => {
            let query = args["query"].as_str().ok_or_else(|| anyhow!("query missing"))?;
            let content = fs::read_to_string(path)?;
            let mut results = Vec::new();
            for (i, line) in content.lines().enumerate() {
                if line.contains(query) {
                    results.push(format!("{}:{}", i + 1, line));
                }
            }
            Ok(results.join("\n"))
        }
        "update" => {
            let content = args["content"].as_str().ok_or_else(|| anyhow!("content missing"))?;
            fs::write(path, content)?;
            Ok(format!("Updated {path}"))
        }
        _ => Err(anyhow!("Unknown action: {action}")),
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
