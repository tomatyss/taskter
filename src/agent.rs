use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::store::Task;
use anyhow::Result;

#[derive(Debug, PartialEq)]
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
 
#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{Task, TaskStatus};

    fn make_task(title: &str) -> Task {
        Task {
            id: 0,
            title: title.to_string(),
            description: None,
            status: TaskStatus::ToDo,
            agent_id: None,
            comment: None,
        }
    }

    #[test]
    fn execute_email_success() {
        let agent = Agent {
            id: 0,
            system_prompt: String::new(),
            tools: vec![Tool { name: "email".into() }],
            model: String::new(),
        };
        let task = make_task("send email to user");
        let result = execute_task(&agent, &task).unwrap();
        assert_eq!(result, ExecutionResult::Success);
    }

    #[test]
    fn execute_email_failure_send() {
        let agent = Agent {
            id: 0,
            system_prompt: String::new(),
            tools: vec![Tool { name: "email".into() }],
            model: String::new(),
        };
        let task = make_task("send email fail");
        let result = execute_task(&agent, &task).unwrap();
        assert!(matches!(result, ExecutionResult::Failure { comment } if comment == "The agent failed to send the email."));
    }

    #[test]
    fn execute_without_tool() {
        let agent = Agent {
            id: 0,
            system_prompt: String::new(),
            tools: vec![],
            model: String::new(),
        };
        let task = make_task("send email to user");
        let result = execute_task(&agent, &task).unwrap();
        assert!(matches!(result, ExecutionResult::Failure { comment } if comment.contains("does not have")));
    }

    #[test]
    fn execute_unknown_task() {
        let agent = Agent {
            id: 0,
            system_prompt: String::new(),
            tools: vec![Tool { name: "email".into() }],
            model: String::new(),
        };
        let task = make_task("perform other action");
        let result = execute_task(&agent, &task).unwrap();
        assert!(matches!(result, ExecutionResult::Failure { comment } if comment.contains("does not have")));
    }
}
