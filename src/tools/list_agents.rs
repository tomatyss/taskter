use anyhow::Result;
use serde_json::Value;

use crate::agent::{self, FunctionDeclaration};
use crate::tools::Tool;
use std::collections::HashMap;

const DECL_JSON: &str = include_str!("../../tools/list_agents.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid list_agents.json")
}

pub fn execute(_args: &Value) -> Result<String> {
    let agents = agent::list_agents()?;
    Ok(serde_json::to_string_pretty(&agents)?)
}

pub fn register(map: &mut HashMap<&'static str, Tool>) {
    map.insert(
        "list_agents",
        Tool {
            declaration: declaration(),
            execute,
        },
    );
}
