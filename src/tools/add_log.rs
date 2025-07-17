use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;

use crate::agent::FunctionDeclaration;

const DECL_JSON: &str = include_str!("../../tools/add_log.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid add_log.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let message = args["message"]
        .as_str()
        .ok_or_else(|| anyhow!("message missing"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(".taskter/logs.log")?;
    writeln!(file, "{}", message)?;
    Ok("Log entry added".to_string())
}
