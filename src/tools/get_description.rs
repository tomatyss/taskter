use anyhow::Result;
use serde_json::Value;
use std::fs;

use crate::agent::FunctionDeclaration;
use crate::config;

const DECL_JSON: &str = include_str!("../../tools/get_description.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid get_description.json")
}

pub fn execute(_args: &Value) -> Result<String> {
    let content = fs::read_to_string(config::description_file())?;
    Ok(content)
}
