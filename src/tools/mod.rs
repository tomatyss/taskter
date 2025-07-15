use anyhow::Result;
use serde_json::Value;

use crate::agent::FunctionDeclaration;

pub mod email;

pub fn builtin_declaration(name: &str) -> Option<FunctionDeclaration> {
    match name {
        "send_email" | "email" => Some(email::declaration()),
        _ => None,
    }
}

pub fn execute_tool(name: &str, args: &Value) -> Result<String> {
    match name {
        "send_email" | "email" => email::execute(args),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}
