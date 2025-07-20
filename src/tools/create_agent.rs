use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::Path;

use crate::agent::{self, Agent, FunctionDeclaration};
use crate::tools;

const DECL_JSON: &str = include_str!("../../tools/create_agent.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid create_agent.json")
}

fn parse_tools(value: &Value) -> Result<Vec<FunctionDeclaration>> {
    if value.is_null() {
        return Ok(vec![]);
    }
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("tools must be an array"))?;
    let mut declarations = Vec::new();
    for spec_val in arr {
        let spec = spec_val
            .as_str()
            .ok_or_else(|| anyhow!("tool spec must be a string"))?;
        let decl = if Path::new(spec).exists() {
            let tool_content = std::fs::read_to_string(spec)?;
            let tool_json: serde_json::Value = serde_json::from_str(&tool_content)?;
            serde_json::from_value(tool_json)?
        } else if let Some(built) = tools::builtin_declaration(spec) {
            built
        } else {
            return Err(anyhow!(format!("Unknown tool: {spec}")));
        };
        declarations.push(decl);
    }
    Ok(declarations)
}

pub fn execute(args: &Value) -> Result<String> {
    let prompt = args
        .get("prompt")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("prompt missing"))?;
    let tools_val = args.get("tools").cloned().unwrap_or(Value::Null);
    let declarations = parse_tools(&tools_val)?;
    let model = args
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("gemini-2.5-pro");

    let mut agents = agent::load_agents()?;
    if let Some(id_val) = args.get("id") {
        let id = id_val
            .as_u64()
            .ok_or_else(|| anyhow!("id must be an integer"))? as usize;
        if let Some(existing) = agents.iter_mut().find(|a| a.id == id) {
            existing.system_prompt = prompt.to_string();
            existing.tools = declarations;
            existing.model = model.to_string();
            agent::save_agents(&agents)?;
            return Ok(format!("Agent {id} updated"));
        }
    }
    let id = agents.len() + 1;
    let new_agent = Agent {
        id,
        system_prompt: prompt.to_string(),
        tools: declarations,
        model: model.to_string(),
    };
    agents.push(new_agent);
    agent::save_agents(&agents)?;
    Ok(format!("Agent {id} created"))
}
