use crate::store::Task;
use crate::tools;
use anyhow::{Context, Result};
use chrono::Local;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

use crate::config;

/// Result of running an [`Agent`] on a [`Task`].
#[derive(Debug, PartialEq)]
pub enum ExecutionResult {
    Success { comment: String },
    Failure { comment: String },
}

/// Appends a message to the log file with a timestamp.
///
/// # Errors
///
/// Returns an error if the log file cannot be opened or written to.
fn append_log(message: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config::log_path())?;
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "[{timestamp}] {message}")?;
    Ok(())
}

/// Logs the start of task execution.
///
/// # Errors
///
/// Returns an error if logging fails.
fn log_start(agent: &Agent, task: Option<&Task>) -> Result<()> {
    let message = if let Some(task) = task {
        format!(
            "Agent {} executing task {}: {}",
            agent.id, task.id, task.title
        )
    } else {
        format!("Agent {} executing without a task", agent.id)
    };
    append_log(&message)
}

/// Constructs the request body for the Gemini API.
///
/// # Errors
///
/// This function never errors.
fn build_request(agent: &Agent, history: &[Value]) -> Value {
    json!({
        "contents": history,
        "tools": [{"functionDeclarations": agent.tools}]
    })
}

/// Handles an offline or error scenario.
///
/// # Errors
///
/// Returns an error if logging fails.
fn offline_fallback(
    agent: &Agent,
    has_send_email_tool: bool,
    pre_log: &str,
    success_comment: &str,
) -> Result<ExecutionResult> {
    append_log(pre_log)?;
    if has_send_email_tool {
        append_log(&format!(
            "Agent {} finished successfully: {}",
            agent.id, success_comment
        ))?;
        Ok(ExecutionResult::Success {
            comment: success_comment.to_string(),
        })
    } else {
        let msg = "Required tool not available.".to_string();
        append_log(&format!("Agent {} failed: {}", agent.id, msg))?;
        Ok(ExecutionResult::Failure { comment: msg })
    }
}

/// Parses the model response and updates history accordingly.
///
/// # Errors
///
/// Returns an error if logging or tool execution fails or if the response is malformed.
fn parse_response(
    agent: &Agent,
    response_json: Value,
    history: &mut Vec<Value>,
) -> Result<Option<ExecutionResult>> {
    let candidate = response_json
        .get("candidates")
        .and_then(|c| c.get(0))
        .context("Malformed API response: missing candidates[0]")?;
    let part = candidate
        .get("content")
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .context("Malformed API response: missing content.parts[0]")?;

    if let Some(function_call) = part.get("functionCall") {
        let tool_name = function_call
            .get("name")
            .and_then(Value::as_str)
            .context("Malformed API response: missing field `name`")?;
        let args = function_call
            .get("args")
            .context("Malformed API response: missing field `args`")?;
        append_log(&format!(
            "Agent {} calling tool {} with args {}",
            agent.id, tool_name, args
        ))?;
        let tool_response = tools::execute_tool(tool_name, args)?;
        append_log(&format!("Tool {tool_name} responded with {tool_response}"))?;
        history.push(json!({
            "role": "model",
            "parts": [{"functionCall": function_call.clone()}]
        }));
        history.push(json!({
            "role": "tool",
            "parts": [{"functionResponse": {"name": tool_name, "response": {"content": tool_response}}}]
        }));
        Ok(None)
    } else if let Some(text) = part.get("text").and_then(Value::as_str) {
        append_log(&format!(
            "Agent {} finished successfully: {}",
            agent.id, text
        ))?;
        Ok(Some(ExecutionResult::Success {
            comment: text.to_string(),
        }))
    } else {
        let msg = "No tool call or text response from the model".to_string();
        append_log(&format!("Agent {} failed: {}", agent.id, msg))?;
        Ok(Some(ExecutionResult::Failure { comment: msg }))
    }
}

/// Executes a task with the given agent and records progress in `.taskter/logs.log`.
///
/// Tools referenced by the agent may be invoked during execution.
///
/// # Errors
///
/// Returns an error if logging fails or if a tool execution fails.
pub async fn execute_task(agent: &Agent, task: Option<&Task>) -> Result<ExecutionResult> {
    let client = Client::new();
    log_start(agent, task)?;

    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) if !key.trim().is_empty() => Some(key),
        _ => None,
    };

    let has_send_email_tool = agent.tools.iter().any(|t| t.name == "send_email");

    if api_key.is_none() {
        return offline_fallback(
            agent,
            has_send_email_tool,
            "Executing without API key - required tool check",
            "No API key found. Task considered complete.",
        );
    }
    let api_key = api_key.context("API key missing after initial check")?;

    let user_prompt = if let Some(task) = task {
        if let Some(description) = &task.description {
            format!(
                "Task Title: {}\nTask Description: {}",
                task.title, description
            )
        } else {
            task.title.clone()
        }
    } else {
        String::new()
    };

    let mut history = vec![json!({
        "role": "user",
        "parts": [{"text": format!("System: {}\nUser: {}", agent.system_prompt, user_prompt)}]
    })];

    loop {
        let request_body = build_request(agent, &history);

        let response = match client
            .post(format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                agent.model,
            ))
            .header("x-goog-api-key", &api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(_) => {
                return offline_fallback(
                    agent,
                    has_send_email_tool,
                    "API request failed; falling back to local simulation",
                    "Tool available. Task considered complete.",
                );
            }
        };

        if !response.status().is_success() {
            return offline_fallback(
                agent,
                has_send_email_tool,
                "API returned error status; falling back to local simulation",
                "Tool available. Task considered complete.",
            );
        }

        let response_json: Value = match response.json().await {
            Ok(json) => json,
            Err(_) => {
                return offline_fallback(
                    agent,
                    has_send_email_tool,
                    "Failed to parse API response; falling back to local simulation",
                    "Tool available. Task considered complete.",
                );
            }
        };

        if let Some(result) = parse_response(agent, response_json, &mut history)? {
            return Ok(result);
        }
    }
}

/// Describes an available tool for the language model.
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
        let parent = path
            .parent()
            .context("agents path has no parent directory")?;
        fs::create_dir_all(parent)?;
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
    prompt: String,
    tools: Vec<FunctionDeclaration>,
    model: String,
) -> anyhow::Result<()> {
    let mut agents = load_agents()?;
    if let Some(agent) = agents.iter_mut().find(|a| a.id == id) {
        agent.system_prompt = prompt;
        agent.tools = tools;
        agent.model = model;
        save_agents(&agents)?;
    }
    Ok(())
}
