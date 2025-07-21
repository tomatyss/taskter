use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::agent::FunctionDeclaration;
use crate::store;

const DECL_JSON: &str = include_str!("../../tools/create_task.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid create_task.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let title = args["title"]
        .as_str()
        .ok_or_else(|| anyhow!("title missing"))?;
    let description = args
        .get("description")
        .and_then(|d| d.as_str())
        .map(String::from);

    let mut board = store::load_board()?;
    let id = board.add_task(title.to_string(), description);
    store::save_board(&board)?;
    Ok(format!("Created task {id}"))
}
