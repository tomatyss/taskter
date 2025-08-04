use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::config;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AgentState {
    Running,
    Idle,
}

pub type AgentStatusMap = HashMap<usize, AgentState>;

pub fn load_agent_status() -> anyhow::Result<AgentStatusMap> {
    let path = config::agent_status_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(path)?;
    let map = serde_json::from_str(&content)?;
    Ok(map)
}

pub fn save_agent_status(map: &AgentStatusMap) -> anyhow::Result<()> {
    let path = config::agent_status_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(map)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn set_agent_state(id: usize, state: AgentState) -> anyhow::Result<()> {
    let mut map = load_agent_status()?;
    map.insert(id, state);
    save_agent_status(&map)
}

pub struct StatusGuard {
    id: usize,
}

impl StatusGuard {
    pub fn new(id: usize) -> Self {
        let _ = set_agent_state(id, AgentState::Running);
        StatusGuard { id }
    }
}

impl Drop for StatusGuard {
    fn drop(&mut self) {
        let _ = set_agent_state(self.id, AgentState::Idle);
    }
}
