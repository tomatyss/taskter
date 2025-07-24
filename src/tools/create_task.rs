use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::agent::FunctionDeclaration;
use crate::store::{self, Task, TaskStatus};
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/create_task.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid create_task.json")
}

/// Creates a new task in `.taskter/board.json`.
pub fn execute(args: &Value) -> Result<String> {
    let title = args["title"]
        .as_str()
        .ok_or_else(|| anyhow!("title missing"))?;
    let description = args
        .get("description")
        .and_then(|d| d.as_str())
        .map(String::from);

    let mut board = store::load_board()?;
    let id = board.next_task_id();
    let task = Task {
        id,
        title: title.to_string(),
        description,
        status: TaskStatus::ToDo,
        agent_id: None,
        comment: None,
    };
    board.tasks.push(task);
    store::save_board(&board)?;
    Ok(format!("Created task {id}"))
}

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "create_task",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
