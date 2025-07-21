use anyhow::Result;
use serde_json::Value;

use crate::agent::{self, FunctionDeclaration};

const DECL_JSON: &str = include_str!("../../tools/list_agents.json");

/// Return the declaration for the agent listing tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid list_agents.json")
}

/// List all agents defined in `.taskter/agents.json`.
pub fn execute(_args: &Value) -> Result<String> {
    let agents = agent::list_agents()?;
    Ok(serde_json::to_string_pretty(&agents)?)
}
