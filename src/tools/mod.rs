use anyhow::Result;
use serde_json::Value;

use crate::agent::FunctionDeclaration;

pub mod add_log;
pub mod add_okr;
pub mod assign_agent;
pub mod create_task;
pub mod email;
pub mod get_description;
pub mod list_agents;
pub mod list_tasks;
pub mod run_bash;
pub mod run_python;

/// List of built-in tool names available to agents.
pub const BUILTIN_TOOLS: &[&str] = &[
    "send_email",
    "create_task",
    "assign_agent",
    "add_log",
    "add_okr",
    "list_tasks",
    "list_agents",
    "run_bash",
    "run_python",
    "get_description",
];

/// Returns the names of all built-in tools.
pub fn builtin_names() -> Vec<&'static str> {
    BUILTIN_TOOLS.to_vec()
}

pub fn builtin_declaration(name: &str) -> Option<FunctionDeclaration> {
    match name {
        "send_email" | "email" => Some(email::declaration()),
        "create_task" => Some(create_task::declaration()),
        "assign_agent" => Some(assign_agent::declaration()),
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
