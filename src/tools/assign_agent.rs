use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::agent::{self, FunctionDeclaration};
use crate::store;

const DECL_JSON: &str = include_str!("../../tools/assign_agent.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid assign_agent.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let task_id = args["task_id"]
        .as_u64()
        .ok_or_else(|| anyhow!("task_id missing"))? as usize;
    let agent_id = args["agent_id"]
        .as_u64()
        .ok_or_else(|| anyhow!("agent_id missing"))? as usize;

    let agents = agent::load_agents()?;
    if !agents.iter().any(|a| a.id == agent_id) {
        return Ok(format!("Agent {} not found", agent_id));
    }

    let mut board = store::load_board()?;
    if let Some(task) = board.tasks.iter_mut().find(|t| t.id == task_id) {
        task.agent_id = Some(agent_id);
        store::save_board(&board)?;
        Ok(format!("Agent {} assigned to task {}", agent_id, task_id))
    } else {
        Ok(format!("Task {} not found", task_id))
    }
}
