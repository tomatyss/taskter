#![allow(
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::manual_let_else,
    clippy::single_match_else
)]

//! Executes tasks using an agent and records progress in the log.

use crate::store::Task;
use crate::tools;
use anyhow::Result;
use chrono::Local;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

use crate::config;

/// Result of running an [`Agent`] on a [`Task`].
#[must_use = "inspect the result to handle success or failure"]
#[derive(Debug, PartialEq)]
pub enum ExecutionResult {
    Success { comment: String },
    Failure { comment: String },
}

fn append_log(message: &str) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config::log_path())?;
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "[{timestamp}] {message}")?;
    Ok(())
}

async fn call_remote_api(
    client: &Client,
    agent: &Agent,
    history: &[Value],
    api_key: &str,
) -> anyhow::Result<Value> {
    let request_body = json!({
        "contents": history,
        "tools": [{"functionDeclarations": agent.tools}]
    });

    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            agent.model
        ))
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            let _ = append_log("API request failed; falling back to local simulation");
            e
        })?;

    if !response.status().is_success() {
        let _ = append_log("API returned error status; falling back to local simulation");
        anyhow::bail!("status {}", response.status());
    }

    response.json().await.map_err(|e| {
        let _ = append_log("Failed to parse API response; falling back to local simulation");
        e.into()
    })
}

fn simulate_without_api(agent: &Agent, has_send_email_tool: bool) -> ExecutionResult {
    if has_send_email_tool {
        let msg = "Tool available. Task considered complete.".to_string();
        let _ = append_log(&format!("Agent {} finished successfully: {}", agent.id, msg));
        ExecutionResult::Success { comment: msg }
    } else {
        let msg = "Required tool not available.".to_string();
        let _ = append_log(&format!("Agent {} failed: {}", agent.id, msg));
        ExecutionResult::Failure { comment: msg }
    }
}

fn handle_model_response(
    agent: &Agent,
    response_json: Value,
    history: &mut Vec<Value>,
) -> anyhow::Result<Option<ExecutionResult>> {
    let candidate = &response_json["candidates"][0];
    let part = &candidate["content"]["parts"][0];

    if let Some(function_call) = part.get("functionCall") {
        let tool_name = function_call
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Malformed API response: missing field `name`"))?;
        let args = &function_call["args"];
        let _ = append_log(&format!(
            "Agent {} calling tool {} with args {}",
            agent.id, tool_name, args
        ));
        let tool_response = tools::execute_tool(tool_name, args)?;
        let _ = append_log(&format!("Tool {tool_name} responded with {tool_response}"));

        history.push(json!({
            "role": "model",
            "parts": [{"functionCall": function_call.clone()}]
        }));
        history.push(json!({
            "role": "tool",
            "parts": [{"functionResponse": {"name": tool_name, "response": {"content": tool_response}}}]
        }));
        return Ok(None);
    }

    if let Some(text) = part.get("text").and_then(Value::as_str) {
        let comment = text.to_string();
        let _ = append_log(&format!("Agent {} finished successfully: {}", agent.id, comment));
        return Ok(Some(ExecutionResult::Success { comment }));
    }

    let msg = "No tool call or text response from the model".to_string();
    let _ = append_log(&format!("Agent {} failed: {}", agent.id, msg));
    Ok(Some(ExecutionResult::Failure { comment: msg }))
}

/// Executes a task with the given agent and records progress in `.taskter/logs.log`.
///
/// Tools referenced by the agent may be invoked during execution.
///
/// # Errors
///
/// Returns an error if writing to the log fails or if a tool execution fails.
#[must_use = "use the result to determine task outcome"]
pub async fn execute_task(agent: &Agent, task: Option<&Task>) -> Result<ExecutionResult> {
    let client = Client::new();
    let log_message = if let Some(task) = task {
        format!("Agent {} executing task {}: {}", agent.id, task.id, task.title)
    } else {
        format!("Agent {} executing without a task", agent.id)
    };
    let _ = append_log(&log_message);

    let api_key = std::env::var("GEMINI_API_KEY").ok().filter(|k| !k.trim().is_empty());
    let has_send_email_tool = agent.tools.iter().any(|t| t.name == "send_email");

    if api_key.is_none() {
        let _ = append_log("Executing without API key");
        return Ok(simulate_without_api(agent, has_send_email_tool));
    }
    let api_key = api_key.unwrap();

    let user_prompt = match task {
        Some(task) => match &task.description {
            Some(desc) => format!("Task Title: {}\nTask Description: {}", task.title, desc),
            None => task.title.clone(),
        },
        None => String::new(),
    };

    let mut history = vec![json!({
        "role": "user",
        "parts": [{"text": format!("System: {}\nUser: {}", agent.system_prompt, user_prompt)}]
    })];

    loop {
        let response_json = match call_remote_api(&client, agent, &history, &api_key).await {
            Ok(json) => json,
            Err(_) => return Ok(simulate_without_api(agent, has_send_email_tool)),
        };

        if let Some(result) = handle_model_response(agent, response_json, &mut history)? {
            return Ok(result);
        }
    }
}

/// Describes an available tool for the language model.
#[must_use = "register the declaration so the tool can be used"]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "empty_params")]
    pub parameters: Value,
}

fn empty_params() -> Value {
    serde_json::json!({})
}

/// Configuration for an autonomous agent stored in `.taskter/agents.json`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Agent {
    pub id: usize,
    pub system_prompt: String,
    pub tools: Vec<FunctionDeclaration>,
    pub model: String,
    #[serde(default)]
    pub schedule: Option<String>,
    #[serde(default)]
    pub repeat: bool,
}

/// Loads the list of agents from `.taskter/agents.json`.
///
/// The file is created if it does not exist.
///
/// # Errors
///
/// Returns an error if the file cannot be read, created or if the JSON is invalid.
pub fn load_agents() -> anyhow::Result<Vec<Agent>> {
    let path = config::agents_path();
    if !path.exists() {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, "[]")?;
    }

    let content = fs::read_to_string(path)?;
    let agents: Vec<Agent> = serde_json::from_str(&content)?;
    Ok(agents)
}

/// Writes the provided agents to `.taskter/agents.json`.
///
/// # Errors
///
/// Returns an error if the agents cannot be serialized or if the file cannot be
/// written.
pub fn save_agents(agents: &[Agent]) -> anyhow::Result<()> {
    let path = config::agents_path();
    let content = serde_json::to_string_pretty(agents)?;
    fs::write(path, content)?;
    Ok(())
}

/// Convenience wrapper around [`load_agents`].
///
/// # Errors
///
/// Propagates any error from [`load_agents`].
pub fn list_agents() -> anyhow::Result<Vec<Agent>> {
    load_agents()
}

/// Removes an agent from `.taskter/agents.json` by ID.
///
/// # Errors
///
/// Returns an error if the agent list cannot be loaded or saved.
pub fn delete_agent(id: usize) -> anyhow::Result<()> {
    let mut agents = load_agents()?;
    if let Some(pos) = agents.iter().position(|a| a.id == id) {
        agents.remove(pos);
        save_agents(&agents)?;
    }
    Ok(())
}

/// Updates an existing agent in `.taskter/agents.json`.
///
/// # Errors
///
/// Returns an error if the agent list cannot be loaded or saved.
pub fn update_agent(
    id: usize,
    prompt: Option<String>,
    tools: Option<Vec<FunctionDeclaration>>,
    model: Option<String>,
) -> anyhow::Result<()> {
    let mut agents = load_agents()?;
    if let Some(agent) = agents.iter_mut().find(|a| a.id == id) {
        if let Some(p) = prompt {
            agent.system_prompt = p;
        }
        if let Some(t) = tools {
            agent.tools = t;
        }
        if let Some(m) = model {
            agent.model = m;
        }
        save_agents(&agents)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use reqwest::Client;

    #[tokio::test(flavor = "current_thread")]
    async fn call_remote_api_returns_err_on_network_failure() {
        std::env::set_var("GEMINI_API_KEY", "dummy");
        std::env::set_var("https_proxy", "http://127.0.0.1:9");

        let client = Client::new();
        let agent = Agent {
            id: 1,
            system_prompt: String::new(),
            tools: vec![],
            model: "gemini-2.5-flash".into(),
            schedule: None,
            repeat: false,
        };
        let history = vec![json!({"role": "user", "parts": [{"text": "hi"}]})];
        let result = call_remote_api(&client, &agent, &history, "dummy").await;
        assert!(result.is_err());

        std::env::remove_var("https_proxy");
        std::env::remove_var("GEMINI_API_KEY");
    }

    #[test]
    fn simulate_without_api_behaves() {
        let agent = Agent {
            id: 1,
            system_prompt: String::new(),
            tools: vec![],
            model: String::new(),
            schedule: None,
            repeat: false,
        };
        assert!(matches!(
            simulate_without_api(&agent, true),
            ExecutionResult::Success { .. }
        ));
        assert!(matches!(
            simulate_without_api(&agent, false),
            ExecutionResult::Failure { .. }
        ));
    }

    #[test]
    fn handle_model_response_parses_tool_and_text() {
        let agent = Agent {
            id: 1,
            system_prompt: String::new(),
            tools: vec![crate::tools::run_python::declaration()],
            model: String::new(),
            schedule: None,
            repeat: false,
        };
        let mut history = Vec::new();

        let response_json = json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "functionCall": {
                            "name": "run_python",
                            "args": {"code": "print(40 + 2)"}
                        }
                    }]
                }
            }]
        });

        let res = handle_model_response(&agent, response_json, &mut history)
            .expect("tool call");
        assert!(res.is_none());
        assert_eq!(history.len(), 2);

        let response_json = json!({
            "candidates": [{
                "content": {"parts": [{"text": "done"}]}
            }]
        });

        let res = handle_model_response(&agent, response_json, &mut history)
            .expect("text response");
        assert!(matches!(
            res,
            Some(ExecutionResult::Success { comment }) if comment == "done"
        ));
    }
}
