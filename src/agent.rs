use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::store::Task;
use anyhow::Result;

pub enum ExecutionResult {
    Success,
    Failure { comment: String },
}

pub fn execute_task(agent: &Agent, task: &Task) -> Result<ExecutionResult> {
    // Mock Gemini API call
    let has_email_tool = agent.tools.iter().any(|t| t.name == "email");
    let is_email_task = task.title.to_lowercase().contains("send email");

    if has_email_tool && is_email_task {
        if task.title.contains("fail") {
            Ok(ExecutionResult::Failure {
                comment: "The agent failed to send the email.".to_string(),
            })
        } else {
            Ok(ExecutionResult::Success)
        }
    } else {
        Ok(ExecutionResult::Failure {
            comment: "The agent does not have the required tool or the task is not an email task.".to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tool {
    pub name: String,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Agent {
    pub id: usize,
    pub system_prompt: String,
    pub tools: Vec<Tool>,
    pub model: String,
}

pub fn load_agents() -> anyhow::Result<Vec<Agent>> {
    let path = Path::new(".taskter/agents.json");
    if !path.exists() {
        // Create the file with an empty list if it doesn't exist
        fs::write(path, "[]")?;
    }

    let content = fs::read_to_string(path)?;
    let agents: Vec<Agent> = serde_json::from_str(&content)?;
    Ok(agents)
}

pub fn save_agents(agents: &[Agent]) -> anyhow::Result<()> {
    let path = Path::new(".taskter/agents.json");
    let content = serde_json::to_string_pretty(agents)?;
    fs::write(path, content)?;
    Ok(())
}
