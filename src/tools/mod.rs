use anyhow::Result;
use serde_json::Value;

use crate::agent::FunctionDeclaration;

pub mod add_log;
pub mod add_okr;
pub mod assign_agent;
pub mod create_agent;
pub mod create_task;
pub mod email;
pub mod get_description;
pub mod list_agents;
pub mod list_tasks;
pub mod run_bash;
pub mod run_python;
pub mod update_agent;

/// Return the names of all built-in tools by inspecting the `tools` directory.
pub fn builtin_names() -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir("tools") {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                        names.push(name.to_string());
                    }
                }
            }
        }
    }
    names.sort();
    names
}

pub fn builtin_declaration(name: &str) -> Option<FunctionDeclaration> {
    match name {
        "send_email" | "email" => Some(email::declaration()),
        "create_task" => Some(create_task::declaration()),
        "assign_agent" => Some(assign_agent::declaration()),
        "create_agent" => Some(create_agent::declaration()),
        "update_agent" => Some(update_agent::declaration()),
        "add_log" => Some(add_log::declaration()),
        "add_okr" => Some(add_okr::declaration()),
        "list_tasks" => Some(list_tasks::declaration()),
        "list_agents" => Some(list_agents::declaration()),
        "run_bash" => Some(run_bash::declaration()),
        "run_python" => Some(run_python::declaration()),
        "get_description" => Some(get_description::declaration()),
        _ => None,
    }
}

pub fn execute_tool(name: &str, args: &Value) -> Result<String> {
    match name {
        "send_email" | "email" => email::execute(args),
        "create_task" => create_task::execute(args),
        "assign_agent" => assign_agent::execute(args),
        "create_agent" => create_agent::execute(args),
        "update_agent" => update_agent::execute(args),
        "add_log" => add_log::execute(args),
        "add_okr" => add_okr::execute(args),
        "list_tasks" => list_tasks::execute(args),
        "list_agents" => list_agents::execute(args),
        "run_bash" => run_bash::execute(args),
        "run_python" => run_python::execute(args),
        "get_description" => get_description::execute(args),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}
