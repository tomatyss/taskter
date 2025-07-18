use std::fs;
use taskter::agent::{self, Agent};

fn with_temp_dir<F: FnOnce() -> T, T>(test: F) -> T {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let original_dir = std::env::current_dir().expect("cannot read current dir");
    std::env::set_current_dir(tmp.path()).expect("cannot set current dir");

    // Ensure workspace directory exists
    fs::create_dir(".taskter").unwrap();

    let result = test();

    std::env::set_current_dir(original_dir).expect("cannot restore current dir");
    result
}

#[test]
fn load_creates_empty_agents_file() {
    with_temp_dir(|| {
        let agents = agent::load_agents().expect("cannot load agents");
        assert!(agents.is_empty());

        let content = fs::read_to_string(".taskter/agents.json").expect("cannot read file");
        assert_eq!(content, "[]");
    });
}

#[test]
fn save_and_list_roundtrip() {
    with_temp_dir(|| {
        let agents = vec![
            Agent { id: 1, system_prompt: "a".into(), tools: vec![], model: "m".into() },
            Agent { id: 2, system_prompt: "b".into(), tools: vec![], model: "m".into() },
        ];

        agent::save_agents(&agents).expect("save failed");
        let loaded = agent::load_agents().expect("load failed");
        assert_eq!(loaded, agents);

        let listed = agent::list_agents().expect("list failed");
        assert_eq!(listed, agents);
    });
}

#[test]
fn delete_removes_agent() {
    with_temp_dir(|| {
        let agents = vec![
            Agent { id: 1, system_prompt: "a".into(), tools: vec![], model: "m".into() },
            Agent { id: 2, system_prompt: "b".into(), tools: vec![], model: "m".into() },
        ];

        agent::save_agents(&agents).expect("save failed");
        agent::delete_agent(1).expect("delete failed");

        let remaining = agent::load_agents().expect("load failed");
        assert_eq!(remaining, vec![agents[1].clone()]);

        let content = fs::read_to_string(".taskter/agents.json").expect("read failed");
        let on_disk: Vec<Agent> = serde_json::from_str(&content).unwrap();
        assert_eq!(on_disk, remaining);
    });
}
