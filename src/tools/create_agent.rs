use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::agent::{self, FunctionDeclaration};
use crate::tools;

const DECL_JSON: &str = include_str!("../../tools/create_agent.json");

pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid create_agent.json")
}

pub fn execute(args: &Value) -> Result<String> {
    let prompt = args["prompt"]
        .as_str()
        .ok_or_else(|| anyhow!("prompt missing"))?;
    let tools = args["tools"]
        .as_array()
        .ok_or_else(|| anyhow!("tools missing"))?;
    let model = args
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("gemini-2.5-flash");

    let mut agents = agent::load_agents()?;
    let mut function_declarations = Vec::new();
    for t in tools {
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
        function_declarations.push(decl);
    }

    let new_agent = agent::Agent {
        id: agents.len() + 1,
        system_prompt: prompt.to_string(),
        tools: function_declarations,
        model: model.to_string(),
    };
    agents.push(new_agent);
    agent::save_agents(&agents)?;
    Ok(format!("Created agent {}", agents.len()))
}
