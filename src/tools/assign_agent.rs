use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::agent::{self, FunctionDeclaration};
use crate::store;
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/assign_agent.json");

/// Returns the function declaration for this tool.
///
/// # Panics
/// Panics if the embedded JSON declaration is invalid.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid assign_agent.json")
}

/// Assigns an agent to a task in `.taskter/board.json`.
///
/// # Errors
/// Returns an error if the board cannot be read or written.
pub fn execute(args: &Value) -> Result<String> {
    let task_id = usize::try_from(
        args["task_id"]
            .as_u64()
            .ok_or_else(|| anyhow!("task_id missing"))?,
    )?;
    let agent_id = usize::try_from(
        args["agent_id"]
            .as_u64()
            .ok_or_else(|| anyhow!("agent_id missing"))?,
    )?;

    let agents = agent::load_agents()?;
    if !agents.iter().any(|a| a.id == agent_id) {
        return Ok(format!("Agent {agent_id} not found"));
    }

    let mut board = store::load_board()?;
    if let Some(task) = board.tasks.iter_mut().find(|t| t.id == task_id) {
        task.agent_id = Some(agent_id);
        store::save_board(&board)?;
        Ok(format!("Agent {agent_id} assigned to task {task_id}"))
    } else {
        Ok(format!("Task {task_id} not found"))
    }
}

/// Registers the tool in the provided map.
pub fn register<S: std::hash::BuildHasher>(map: &mut HashMap<&'static str, Tool, S>) {
    map.insert(
        "assign_agent",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
