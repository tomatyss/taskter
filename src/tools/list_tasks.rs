use anyhow::Result;
use serde_json::Value;

use crate::agent::FunctionDeclaration;
use crate::store;
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/list_tasks.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid list_tasks.json")
}

/// Lists tasks stored in `.taskter/board.json`.
pub fn execute(_args: &Value) -> Result<String> {
    let board = store::load_board()?;
    Ok(serde_json::to_string_pretty(&board.tasks)?)
}

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "list_tasks",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
