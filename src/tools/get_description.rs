use anyhow::Result;
use serde_json::Value;
use std::fs;

use crate::agent::FunctionDeclaration;
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/get_description.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid get_description.json")
}

pub fn execute(_args: &Value) -> Result<String> {
    let content = fs::read_to_string(".taskter/description.md")?;
    Ok(content)
}

pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "get_description",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
