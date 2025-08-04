use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentState {
    Running,
    Idle,
}

pub fn load_status() -> Result<HashMap<usize, AgentState>> {
    let path = config::agent_status_path();
    if !path.exists() {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, "{}")?;
    }
    // Avoid attempting to deserialize arbitrarily large files which could OOM the
    // process. If the status file exceeds 1MB, reset it to an empty map.
    const MAX_BYTES: u64 = 1_048_576; // 1MB
    if fs::metadata(path)?.len() > MAX_BYTES {
        fs::write(path, "{}")?;
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(path)?;
    let map: HashMap<usize, AgentState> = serde_json::from_str(&content).unwrap_or_default();
    Ok(map)
}

pub fn save_status(status: &HashMap<usize, AgentState>) -> Result<()> {
    let path = config::agent_status_path();
    let content = serde_json::to_string_pretty(status)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn set_status(agent_id: usize, state: AgentState) -> Result<()> {
    let mut statuses = load_status()?;
    statuses.insert(agent_id, state);
    save_status(&statuses)
}

pub struct StatusGuard {
    agent_id: usize,
}

impl StatusGuard {
    #[must_use]
    pub fn new(agent_id: usize) -> Self {
        let _ = set_status(agent_id, AgentState::Running);
        Self { agent_id }
    }
}

impl Drop for StatusGuard {
    fn drop(&mut self) {
        let _ = set_status(self.agent_id, AgentState::Idle);
    }
}
