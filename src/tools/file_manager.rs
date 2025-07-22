use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;

const DECL_JSON: &str = include_str!("../../tools/file_manager.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid file_manager.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let action = args["action"]
        .as_str()
        .ok_or_else(|| anyhow!("action missing"))?;
    match action {
        "read" => {
            let path = args["path"]
                .as_str()
                .ok_or_else(|| anyhow!("path missing"))?;
            let content = fs::read_to_string(path)?;
            Ok(content)
        }
        "create" => {
            let path = args["path"]
                .as_str()
                .ok_or_else(|| anyhow!("path missing"))?;
            let content = args["content"].as_str().unwrap_or_default();
            fs::write(path, content)?;
            Ok(format!("File {path} created"))
        }
        "update" => {
            let path = args["path"]
                .as_str()
                .ok_or_else(|| anyhow!("path missing"))?;
            let content = args["content"]
                .as_str()
                .ok_or_else(|| anyhow!("content missing"))?;
            fs::write(path, content)?;
            Ok(format!("File {path} updated"))
        }
        "search" => {
            let query = args["query"]
                .as_str()
                .ok_or_else(|| anyhow!("query missing"))?;
            let mut matches = Vec::new();
            for entry in walkdir::WalkDir::new(".") {
                let entry = entry?;
                if entry.file_type().is_file() {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains(query) {
                            matches.push(entry.path().display().to_string());
                        }
                    }
                }
            }
            Ok(serde_json::to_string(&matches)?)
        }
        other => Err(anyhow!("Unknown action: {other}")),
    }
}

pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "file_manager",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
