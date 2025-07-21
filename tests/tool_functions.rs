use serde_json::json;
use std::fs;

use taskter::agent::{self, Agent};
use taskter::store::{self, Board, Task, TaskStatus};
use taskter::tools::{
    add_log, add_okr, assign_agent, create_task, get_description, list_agents, list_tasks,
    run_bash, run_python,
};

fn with_temp_dir<F: FnOnce() -> T, T>(test: F) -> T {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let original_dir = std::env::current_dir().expect("cannot read current dir");
    std::env::set_current_dir(tmp.path()).expect("cannot set current dir");

    fs::create_dir(".taskter").unwrap();

    let result = test();

    std::env::set_current_dir(original_dir).expect("cannot restore current dir");
    result
}

#[test]
fn create_task_adds_task() {
    with_temp_dir(|| {
        let msg = create_task::execute(&json!({"title": "Test", "description": "desc"})).unwrap();
        assert_eq!(msg, "Created task 1");

        let board = store::load_board().unwrap();
        assert_eq!(board.tasks.len(), 1);
        assert_eq!(board.tasks[0].title, "Test");
        assert_eq!(board.tasks[0].description.as_deref(), Some("desc"));
    });
}

#[test]
fn create_task_requires_title() {
    with_temp_dir(|| {
        let err = create_task::execute(&json!({"description": "d"})).unwrap_err();
        assert!(err.to_string().contains("title missing"));
    });
}

#[test]
fn assign_agent_assigns_task() {
    with_temp_dir(|| {
        let task = Task {
            id: 1,
            title: "t".into(),
            description: None,
            status: TaskStatus::ToDo,
            agent_id: None,
            comment: None,
        };
        store::save_board(&Board { tasks: vec![task] }).unwrap();
        let agent = Agent {
            id: 1,
            system_prompt: "p".into(),
            tools: vec![],
            model: "m".into(),
        };
        agent::save_agents(&[agent]).unwrap();

        let msg = assign_agent::execute(&json!({"task_id":1,"agent_id":1})).unwrap();
        assert_eq!(msg, "Agent 1 assigned to task 1");

        let board = store::load_board().unwrap();
        assert_eq!(board.tasks[0].agent_id, Some(1));
    });
}

#[test]
fn assign_agent_reports_missing_agent() {
    with_temp_dir(|| {
        let task = Task {
            id: 1,
            title: "t".into(),
            description: None,
            status: TaskStatus::ToDo,
            agent_id: None,
            comment: None,
        };
        store::save_board(&Board { tasks: vec![task] }).unwrap();
        let msg = assign_agent::execute(&json!({"task_id":1,"agent_id":1})).unwrap();
        assert_eq!(msg, "Agent 1 not found");
    });
}

#[test]
fn assign_agent_reports_missing_task() {
    with_temp_dir(|| {
        let agent = Agent {
            id: 1,
            system_prompt: "p".into(),
            tools: vec![],
            model: "m".into(),
        };
        agent::save_agents(&[agent]).unwrap();
        let msg = assign_agent::execute(&json!({"task_id":1,"agent_id":1})).unwrap();
        assert_eq!(msg, "Task 1 not found");
    });
}

#[test]
fn assign_agent_requires_fields() {
    with_temp_dir(|| {
        let err = assign_agent::execute(&json!({"task_id":1})).unwrap_err();
        assert!(err.to_string().contains("agent_id missing"));
        let err2 = assign_agent::execute(&json!({"agent_id":1})).unwrap_err();
        assert!(err2.to_string().contains("task_id missing"));
    });
}

#[test]
fn add_okr_adds_entry() {
    with_temp_dir(|| {
        let msg =
            add_okr::execute(&json!({"objective": "Improve", "key_results": ["speed"]})).unwrap();
        assert_eq!(msg, "Added OKR 'Improve'");
        let okrs = store::load_okrs().unwrap();
        assert_eq!(okrs.len(), 1);
        assert_eq!(okrs[0].objective, "Improve");
        assert_eq!(okrs[0].key_results[0].name, "speed");
    });
}

#[test]
fn add_okr_requires_fields() {
    with_temp_dir(|| {
        let err = add_okr::execute(&json!({"objective":"o"})).unwrap_err();
        assert!(err.to_string().contains("key_results missing"));
        let err2 = add_okr::execute(&json!({"key_results":[]})).unwrap_err();
        assert!(err2.to_string().contains("objective missing"));
    });
}

#[test]
fn add_log_appends_message() {
    with_temp_dir(|| {
        add_log::execute(&json!({"message":"hello"})).unwrap();
        let content = fs::read_to_string(".taskter/logs.log").unwrap();
        assert!(content.contains("hello"));
    });
}

#[test]
fn add_log_requires_message() {
    with_temp_dir(|| {
        let err = add_log::execute(&json!({})).unwrap_err();
        assert!(err.to_string().contains("message missing"));
    });
}

#[test]
fn list_agents_outputs_json() {
    with_temp_dir(|| {
        let agent = Agent {
            id: 1,
            system_prompt: "p".into(),
            tools: vec![],
            model: "m".into(),
        };
        agent::save_agents(&[agent.clone()]).unwrap();
        let out = list_agents::execute(&json!({})).unwrap();
        let parsed: Vec<Agent> = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed[0].id, agent.id);
    });
}

#[test]
fn list_agents_empty_when_none() {
    with_temp_dir(|| {
        let out = list_agents::execute(&json!({})).unwrap();
        assert_eq!(out.trim(), "[]");
    });
}

#[test]
fn list_tasks_outputs_json() {
    with_temp_dir(|| {
        let task = Task {
            id: 1,
            title: "t".into(),
            description: None,
            status: TaskStatus::ToDo,
            agent_id: None,
            comment: None,
        };
        store::save_board(&Board {
            tasks: vec![task.clone()],
        })
        .unwrap();
        let out = list_tasks::execute(&json!({})).unwrap();
        let parsed: Vec<Task> = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed[0].id, task.id);
    });
}

#[test]
fn list_tasks_empty_when_none() {
    with_temp_dir(|| {
        let out = list_tasks::execute(&json!({})).unwrap();
        assert_eq!(out.trim(), "[]");
    });
}

#[test]
fn get_description_reads_file() {
    with_temp_dir(|| {
        fs::write(".taskter/description.md", "desc").unwrap();
        let out = get_description::execute(&json!({})).unwrap();
        assert_eq!(out, "desc");
    });
}

#[test]
fn get_description_fails_missing_file() {
    with_temp_dir(|| {
        let err = get_description::execute(&json!({})).unwrap_err();
        assert!(err.to_string().contains("No such file"));
    });
}

#[test]
fn run_bash_requires_command() {
    with_temp_dir(|| {
        let err = run_bash::execute(&json!({})).unwrap_err();
        assert!(err.to_string().contains("command missing"));
    });
}

#[test]
fn run_bash_fails_on_error() {
    with_temp_dir(|| {
        let err = run_bash::execute(&json!({"command": "false"})).unwrap_err();
        assert!(err.to_string().contains("Command failed"));
    });
}

#[test]
fn run_python_requires_code() {
    with_temp_dir(|| {
        let err = run_python::execute(&json!({})).unwrap_err();
        assert!(err.to_string().contains("code missing"));
    });
}

#[test]
fn run_python_fails_on_error() {
    with_temp_dir(|| {
        let err = run_python::execute(&json!({"code": "raise Exception()"}))
            .unwrap_err();
        assert!(err.to_string().contains("Python execution failed"));
    });
}
