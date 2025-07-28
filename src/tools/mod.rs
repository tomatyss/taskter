//! Registry for all builtin tools callable by agents.

use anyhow::Result;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;

use crate::agent::FunctionDeclaration;

pub mod email;
pub mod get_description;
pub mod run_bash;
pub mod run_python;
pub mod taskter_agent;
pub mod taskter_okrs;
pub mod taskter_task;
pub mod taskter_tools;
pub mod web_search;

/// Runtime representation of a callable tool.
pub struct Tool {
    pub declaration: FunctionDeclaration,
    pub execute: fn(&Value) -> Result<String>,
}

/// Registry of all tools bundled with Taskter.
pub static BUILTIN_TOOLS: Lazy<HashMap<&'static str, Tool>> = Lazy::new(|| {
    let mut m = HashMap::new();
    email::register(&mut m);
    get_description::register(&mut m);
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

/// Retrieves the declaration for a built-in tool by name.
pub fn builtin_declaration(name: &str) -> Option<FunctionDeclaration> {
    BUILTIN_TOOLS.get(name).map(|t| t.declaration.clone())
}

/// Executes a named built-in tool.
///
/// Individual tools may read or write files in `.taskter/`.
pub fn execute_tool(name: &str, args: &Value) -> Result<String> {
    if let Some(tool) = BUILTIN_TOOLS.get(name) {
        (tool.execute)(args)
    } else {
        Err(anyhow::anyhow!("Unknown tool: {}", name))
    }
}
