use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::agent::FunctionDeclaration;

const DECL_JSON: &str = include_str!("../../tools/text_file.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid text_file.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let path = args["path"].as_str().ok_or_else(|| anyhow!("path missing"))?;
    if let Some(content) = args.get("content").and_then(|v| v.as_str()) {
        let append = args.get("append").and_then(|v| v.as_bool()).unwrap_or(false);
        if append {
            let mut file = OpenOptions::new().create(true).append(true).open(path)?;
            file.write_all(content.as_bytes())?;
        } else {
            fs::write(path, content)?;
        }
        Ok("File written".to_string())
    } else {
        let data = fs::read_to_string(path)?;
        Ok(data)
    }
}
