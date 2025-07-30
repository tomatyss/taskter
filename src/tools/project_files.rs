use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;

const DECL_JSON: &str = include_str!("../../tools/project_files.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid project_files.json")
}

/// Executes file operations in the project directory.
pub fn execute(args: &Value) -> Result<String> {
    let action = args["action"]
        .as_str()
        .ok_or_else(|| anyhow!("action missing"))?;
    match action {
        "create" => {
            let path = args["path"]
                .as_str()
                .ok_or_else(|| anyhow!("path missing"))?;
            let content = args["content"].as_str().unwrap_or_default();
            fs::write(path, content)?;
            Ok(format!("Created {path}"))
        }
        "read" => {
            let path = args["path"]
                .as_str()
                .ok_or_else(|| anyhow!("path missing"))?;
            let content = fs::read_to_string(path)?;
            Ok(content)
        }
        "update" => {
            let path = args["path"]
                .as_str()
                .ok_or_else(|| anyhow!("path missing"))?;
            let content = args["content"]
                .as_str()
                .ok_or_else(|| anyhow!("content missing"))?;
            fs::write(path, content)?;
            Ok(format!("Updated {path}"))
        }
        "search" => {
            let query = args["query"]
                .as_str()
                .ok_or_else(|| anyhow!("query missing"))?;
            let mut matches = Vec::new();
            for entry in walkdir::WalkDir::new(".") {
                let entry = entry?;
                if entry.file_type().is_file() {
                    if let Ok(contents) = fs::read_to_string(entry.path()) {
                        if contents.contains(query) {
                            matches.push(entry.path().display().to_string());
                        }
                    }
                }
            }
            if matches.is_empty() {
                Ok("No matches found".to_string())
            } else {
                Ok(matches.join("\n"))
            }
        }
        _ => Err(anyhow!("unknown action")),
    }
}

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    // Register under both "project_files" and alias "file_ops"
    let decl = declaration();
    map.insert(
        "project_files",
        Tool {
            declaration: decl.clone(),
            execute,
        },
    );
    map.insert(
        "file_ops",
        Tool {
            declaration: decl,
            execute,
        },
    );
}
