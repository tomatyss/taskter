use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::agent::{self, FunctionDeclaration};
use crate::tools;

const DECL_JSON: &str = include_str!("../../tools/update_agent.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid update_agent.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let id = args["id"].as_u64().ok_or_else(|| anyhow!("id missing"))? as usize;
    let mut agents = agent::load_agents()?;
    let agent = match agents.iter_mut().find(|a| a.id == id) {
        Some(a) => a,
        None => return Ok(format!("Agent {id} not found")),
    };

    if let Some(p) = args.get("prompt").and_then(|v| v.as_str()) {
        agent.system_prompt = p.to_string();
    }

    if let Some(tool_array) = args.get("tools").and_then(|v| v.as_array()) {
        let mut declarations = Vec::new();
        for t in tool_array {
            let spec = t.as_str().ok_or_else(|| anyhow!("invalid tool spec"))?;
            let decl = if Path::new(spec).exists() {
                let tool_content = fs::read_to_string(spec)?;
                let tool_json: Value = serde_json::from_str(&tool_content)?;
                serde_json::from_value(tool_json)?
            } else if let Some(built) = tools::builtin_declaration(spec) {
                built
            } else {
                return Err(anyhow!(format!("Unknown tool: {spec}")));
            };
            declarations.push(decl);
        }
        agent.tools = declarations;
    }

    if let Some(m) = args.get("model").and_then(|v| v.as_str()) {
        agent.model = m.to_string();
    }

    agent::save_agents(&agents)?;
    Ok(format!("Updated agent {id}"))
}
