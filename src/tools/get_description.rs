use anyhow::Result;
use serde_json::Value;
use std::fs;

use crate::agent::FunctionDeclaration;

const DECL_JSON: &str = include_str!("../../tools/get_description.json");

/// Return the declaration for the description retrieval tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid get_description.json")
}

/// Read and return `.taskter/description.md`.
pub fn execute(_args: &Value) -> Result<String> {
    let content = fs::read_to_string(".taskter/description.md")?;
    Ok(content)
}
