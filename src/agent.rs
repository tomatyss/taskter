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
use serde_json::Value;
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
        .open(config::log_path()?)?;
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "[{timestamp}] {message}")?;
    Ok(())
}

fn simulate_without_api(agent: &Agent, has_send_email_tool: bool) -> ExecutionResult {
    if has_send_email_tool {
        let msg = "Tool available. Task considered complete.".to_string();
        let _ = append_log(&format!(
            "Agent {} finished successfully: {}",
            agent.id, msg
        ));
        ExecutionResult::Success { comment: msg }
    } else {
        let msg = "Required tool not available.".to_string();
        let _ = append_log(&format!("Agent {} failed: {}", agent.id, msg));
        ExecutionResult::Failure { comment: msg }
    }
}

use crate::providers::{select_provider, ModelAction};

/// Executes a task with the given agent and records progress in `.taskter/logs.log`.
///
/// Tools referenced by the agent may be invoked during execution.
///
/// # Errors
///
/// Returns an error if writing to the log fails. Tool execution failures are
/// captured as [`ExecutionResult::Failure`] so callers can inspect the outcome.
#[must_use = "use the result to determine task outcome"]
pub async fn execute_task(agent: &Agent, task: Option<&Task>) -> Result<ExecutionResult> {
    let _guard = RunningAgentGuard::new(agent.id);
    let client = Client::builder().no_proxy().build()?;
    let log_message = if let Some(task) = task {
        format!(
            "Agent {} executing task {}: {}",
            agent.id, task.id, task.title
        )
    } else {
        format!("Agent {} executing without a task", agent.id)
    };
    let _ = append_log(&log_message);

    let provider = select_provider(agent);
    let has_send_email_tool = agent.tools.iter().any(|t| t.name == "send_email");

    let requires_api_key = provider.requires_api_key();
    let mut api_key = config::provider_api_key(provider.name())?;
    if api_key.is_none() && requires_api_key {
        // Backwards compatibility with legacy environment variables.
        api_key = std::env::var(provider.api_key_env())
            .ok()
            .filter(|k| !k.trim().is_empty());
    }

    if requires_api_key && api_key.is_none() {
        let _ = append_log("Executing without API key");
        return Ok(simulate_without_api(agent, has_send_email_tool));
    }
    let api_key = api_key.unwrap_or_default();

    let user_prompt = match task {
        Some(task) => match &task.description {
            Some(desc) => format!("Task Title: {}\nTask Description: {}", task.title, desc),
            None => task.title.clone(),
        },
        None => String::new(),
    };

    let mut history = provider.build_history(agent, &user_prompt);

    loop {
        let action = match provider
            .infer(&client, agent, &api_key, &history)
            .await
            .inspect_err(|e| {
                let _ = append_log(&format!(
                    "API request failed; falling back to local simulation: {e}"
                ));
            }) {
            Ok(a) => a,
            Err(_) => return Ok(simulate_without_api(agent, has_send_email_tool)),
        };

        match action {
            ModelAction::ToolCall {
                name,
                args,
                call_id,
            } => {
                let agent_id = agent.id;
                let _ = append_log(&format!(
                    "Agent {agent_id} calling tool {name} with args {args}"
                ));
                let tool_response = match tools::execute_tool(&name, &args) {
                    Ok(response) => response,
                    Err(err) => {
                        let message = format!("Tool {name} failed: {err}");
                        let _ = append_log(&format!("Agent {agent_id} failed: {message}"));
                        return Ok(ExecutionResult::Failure { comment: message });
                    }
                };
                let _ = append_log(&format!("Tool {name} responded with {tool_response}"));
                provider.append_tool_result(
                    agent,
                    &mut history,
                    &name,
                    &args,
                    &tool_response,
                    call_id.as_deref(),
                );
            }
            ModelAction::Text { content } => {
                let _ = append_log(&format!(
                    "Agent {} finished successfully: {}",
                    agent.id, content
                ));
                return Ok(ExecutionResult::Success { comment: content });
            }
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
    pub provider: Option<String>,
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
    let path = config::agents_path()?;
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, "[]")?;
    }

    let content = fs::read_to_string(&path)?;
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
    let path = config::agents_path()?;
    let content = serde_json::to_string_pretty(agents)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn load_running_agents() -> anyhow::Result<Vec<usize>> {
    let path = config::running_agents_path()?;
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, "[]")?;
    }
    let content = fs::read_to_string(&path)?;
    let ids: Vec<usize> = serde_json::from_str(&content)?;
    Ok(ids)
}

pub fn save_running_agents(ids: &[usize]) -> anyhow::Result<()> {
    let path = config::running_agents_path()?;
    let content = serde_json::to_string_pretty(ids)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn set_agent_running(id: usize, running: bool) -> anyhow::Result<()> {
    let mut ids = load_running_agents()?;
    if running {
        if !ids.contains(&id) {
            ids.push(id);
        }
    } else {
        ids.retain(|&x| x != id);
    }
    save_running_agents(&ids)
}

pub struct RunningAgentGuard {
    id: usize,
}

impl RunningAgentGuard {
    pub fn new(id: usize) -> Self {
        let _ = set_agent_running(id, true);
        Self { id }
    }
}

impl Drop for RunningAgentGuard {
    fn drop(&mut self) {
        let _ = set_agent_running(self.id, false);
    }
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
    provider: Option<Option<String>>,
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
        if let Some(pv) = provider {
            agent.provider = pv;
        }
        save_agents(&agents)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{gemini::GeminiProvider, ModelProvider};
    use reqwest::Client;
    use serde_json::json;

    #[tokio::test(flavor = "current_thread")]
    async fn call_remote_api_returns_err_on_network_failure() {
        std::env::set_var("GEMINI_API_KEY", "dummy");
        // Build a client that does not consult system proxies to avoid sandbox issues.
        let client = Client::builder().no_proxy().build().expect("client");
        let agent = Agent {
            id: 1,
            system_prompt: String::new(),
            tools: vec![],
            model: "gemini-2.5-flash".into(),
            provider: Some("gemini".into()),
            schedule: None,
            repeat: false,
        };
        let provider = GeminiProvider;
        let history = provider.build_history(&agent, "hi");
        let result = provider.infer(&client, &agent, "dummy", &history).await;
        assert!(result.is_err());

        std::env::remove_var("GEMINI_API_KEY");
    }

    #[test]
    fn simulate_without_api_behaves() {
        let agent = Agent {
            id: 1,
            system_prompt: String::new(),
            tools: vec![],
            model: String::new(),
            provider: None,
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
            provider: None,
            schedule: None,
            repeat: false,
        };
        let provider = GeminiProvider;
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

        let action = provider.parse_response(&response_json).expect("tool call");
        match action {
            ModelAction::ToolCall {
                name,
                args,
                call_id: _,
            } => {
                let agent_ref = &agent;
                provider.append_tool_result(agent_ref, &mut history, &name, &args, "ok", None);
                assert_eq!(history.len(), 2);
            }
            _ => panic!("expected tool call"),
        }

        let response_json = json!({
            "candidates": [{
                "content": {"parts": [{"text": "done"}]}
            }]
        });

        let action = provider
            .parse_response(&response_json)
            .expect("text response");
        assert!(matches!(action, ModelAction::Text { content } if content == "done"));
    }
}
