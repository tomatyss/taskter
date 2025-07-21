use anyhow::Result;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;

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
pub mod taskter_agent;
pub mod taskter_okrs;
pub mod taskter_task;
pub mod taskter_tools;
pub mod web_search;

pub struct Tool {
    pub declaration: FunctionDeclaration,
    pub execute: fn(&Value) -> Result<String>,
}

pub static BUILTIN_TOOLS: Lazy<HashMap<&'static str, Tool>> = Lazy::new(|| {
    let mut m = HashMap::new();
    add_log::register(&mut m);
    add_okr::register(&mut m);
    assign_agent::register(&mut m);
    create_task::register(&mut m);
    email::register(&mut m);
    get_description::register(&mut m);
    list_agents::register(&mut m);
    list_tasks::register(&mut m);
    run_bash::register(&mut m);
    run_python::register(&mut m);
    web_search::register(&mut m);
    taskter_task::register(&mut m);
    taskter_agent::register(&mut m);
    taskter_okrs::register(&mut m);
    taskter_tools::register(&mut m);
    m
});

/// Returns the names of all built-in tools.
pub fn builtin_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = BUILTIN_TOOLS.keys().copied().collect();
    names.sort();
    names
}

pub fn builtin_declaration(name: &str) -> Option<FunctionDeclaration> {
    BUILTIN_TOOLS.get(name).map(|t| t.declaration.clone())
}

pub fn execute_tool(name: &str, args: &Value) -> Result<String> {
    if let Some(tool) = BUILTIN_TOOLS.get(name) {
        (tool.execute)(args)
    } else {
        Err(anyhow::anyhow!("Unknown tool: {}", name))
    }
}
