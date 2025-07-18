use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::agent::FunctionDeclaration;

const DECL_JSON: &str = include_str!("../../tools/manage_file.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid manage_file.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let filename = args["filename"]
        .as_str()
        .ok_or_else(|| anyhow!("filename missing"))?;
    let mode = args["mode"].as_str().unwrap_or("read");
    match mode {
        "read" => {
            let content = fs::read_to_string(filename)?;
            Ok(content)
        }
        "write" => {
            let content = args["content"]
                .as_str()
                .ok_or_else(|| anyhow!("content missing"))?;
            fs::write(filename, content)?;
            Ok(format!("Written to {filename}"))
        }
        "append" => {
            let content = args["content"]
                .as_str()
                .ok_or_else(|| anyhow!("content missing"))?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename)?;
            write!(file, "{}", content)?;
            Ok(format!("Appended to {filename}"))
        }
        _ => Err(anyhow!("invalid mode")),
    }
}
