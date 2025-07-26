#![allow(clippy::pedantic)]

use taskter::agent::{self, Agent, FunctionDeclaration};

mod common;
pub use common::with_temp_dir;

#[test]
fn load_agents_creates_file_when_missing() {
    with_temp_dir(|| {
        // Ensure file does not exist
        assert!(!std::path::Path::new(".taskter/agents.json").exists());
        let agents = agent::load_agents().expect("load failed");
        assert!(agents.is_empty());
        let content = std::fs::read_to_string(".taskter/agents.json").expect("file missing");
        assert_eq!(content.trim(), "[]");
    });
}

#[test]
fn save_agents_persists_to_disk() {
    with_temp_dir(|| {
        let agent = Agent {
            id: 1,
            system_prompt: "p".into(),
            tools: vec![],
            model: "m".into(),
            schedule: None,
            repeat: false,
        };
        agent::save_agents(&[agent.clone()]).expect("save failed");
        let stored: Vec<Agent> =
            serde_json::from_str(&std::fs::read_to_string(".taskter/agents.json").unwrap())
                .unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id, agent.id);
        assert_eq!(stored[0].system_prompt, agent.system_prompt);
    });
}

#[test]
fn list_agents_returns_saved_agents() {
    with_temp_dir(|| {
        let agent = Agent {
            id: 1,
            system_prompt: "p".into(),
            tools: vec![FunctionDeclaration {
                name: "t".into(),
                description: None,
                parameters: serde_json::json!({}),
            }],
            model: "m".into(),
            schedule: None,
            repeat: false,
        };
        agent::save_agents(&[agent.clone()]).unwrap();
        let listed = agent::list_agents().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, agent.id);
        assert_eq!(listed[0].tools[0].name, "t");
    });
}

#[test]
fn delete_agent_removes_entry() {
    with_temp_dir(|| {
        let a1 = Agent {
            id: 1,
            system_prompt: "p1".into(),
            tools: vec![],
            model: "m".into(),
            schedule: None,
            repeat: false,
        };
        let a2 = Agent {
            id: 2,
            system_prompt: "p2".into(),
            tools: vec![],
            model: "m".into(),
            schedule: None,
            repeat: false,
        };
        agent::save_agents(&[a1.clone(), a2.clone()]).unwrap();
        agent::delete_agent(1).unwrap();
        let remaining = agent::load_agents().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, a2.id);
    });
}
