use anyhow::Result;
use serde_json::Value;
use std::fs;

use crate::agent::FunctionDeclaration;
use crate::config;
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/get_description.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid get_description.json")
}

/// Reads `.taskter/description.md` and returns its contents.
///
/// # Errors
///
/// Returns an error if the description file cannot be read.
pub fn execute(_args: &Value) -> Result<String> {
    let content = fs::read_to_string(config::description_path())?;
    Ok(content)
}

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "get_description",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
