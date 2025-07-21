use anyhow::Result;
use serde_json::Value;

use crate::agent::FunctionDeclaration;
use crate::store;

const DECL_JSON: &str = include_str!("../../tools/list_tasks.json");

/// Return the declaration for the task listing tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid list_tasks.json")
}

/// List all tasks stored in `.taskter/board.json`.
pub fn execute(_args: &Value) -> Result<String> {
    let board = store::load_board()?;
    Ok(serde_json::to_string_pretty(&board.tasks)?)
}
