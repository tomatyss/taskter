use assert_cmd::Command;
use serde_json::json;
use std::fs;

use taskter::agent::{self, Agent};
use taskter::store::{self, Board, Task, TaskStatus};
use taskter::tools::{
    add_log, add_okr, assign_agent, create_task, get_description, list_agents, list_tasks, file_ops,
};

use mockito::{Matcher, Server};

mod common;
pub use common::with_temp_dir;

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
            schedule: None,
            repeat: false,
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
            schedule: None,
            repeat: false,
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
        let content = fs::read_to_string(taskter::config::LOG_FILE).unwrap();
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
            schedule: None,
            repeat: false,
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
        fs::write(taskter::config::DESCRIPTION_FILE, "desc").unwrap();
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
fn web_search_fetches_result() {
    with_temp_dir(|| {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("q".into(), "rust".into()),
                Matcher::UrlEncoded("format".into(), "json".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"AbstractText":"Rust lang","RelatedTopics":[]}"#)
            .create();
        std::env::set_var("SEARCH_API_ENDPOINT", server.url());
        let out = taskter::tools::execute_tool("web_search", &json!({"query":"rust"})).unwrap();
        assert_eq!(out, "Rust lang");
        std::env::remove_var("SEARCH_API_ENDPOINT");
        _m.assert();
    });
}

#[test]
fn taskter_task_tool_lists_tasks() {
    with_temp_dir(|| {
        let cmd = Command::cargo_bin("taskter").unwrap();
        let bin = cmd.get_program().to_owned();
        std::env::set_var("TASKTER_BIN", &bin);

        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        Command::cargo_bin("taskter")
            .unwrap()
            .args(["task", "add", "--title", "Demo"])
            .assert()
            .success();

        let out = taskter::tools::execute_tool("taskter_task", &json!({"args": ["list"]})).unwrap();
        assert!(out.contains("Demo"));
        std::env::remove_var("TASKTER_BIN");
    });
}

#[test]
fn taskter_agent_tool_lists_agents() {
    with_temp_dir(|| {
        let cmd = Command::cargo_bin("taskter").unwrap();
        let bin = cmd.get_program().to_owned();
        std::env::set_var("TASKTER_BIN", &bin);

        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        Command::cargo_bin("taskter")
            .unwrap()
            .args([
                "agent",
                "add",
                "--prompt",
                "helper",
                "--tools",
                "run_bash",
                "--model",
                "gemini-2.5-flash",
            ])
            .assert()
            .success();

        let out =
            taskter::tools::execute_tool("taskter_agent", &json!({"args": ["list"]})).unwrap();
        assert!(out.contains("helper"));
        std::env::remove_var("TASKTER_BIN");
    });
}

#[test]
fn taskter_okrs_tool_lists_okrs() {
    with_temp_dir(|| {
        let cmd = Command::cargo_bin("taskter").unwrap();
        let bin = cmd.get_program().to_owned();
        std::env::set_var("TASKTER_BIN", &bin);

        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        Command::cargo_bin("taskter")
            .unwrap()
            .args(["okrs", "add", "-o", "Improve", "-k", "Speed"])
            .assert()
            .success();

        let out = taskter::tools::execute_tool("taskter_okrs", &json!({"args": ["list"]})).unwrap();
        assert!(out.contains("Improve"));
        std::env::remove_var("TASKTER_BIN");
    });
}

#[test]
fn taskter_tools_tool_lists_builtins() {
    with_temp_dir(|| {
        let cmd = Command::cargo_bin("taskter").unwrap();
        let bin = cmd.get_program().to_owned();
        std::env::set_var("TASKTER_BIN", &bin);

        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        let out =
            taskter::tools::execute_tool("taskter_tools", &json!({"args": ["list"]})).unwrap();
        assert!(out.contains("run_bash"));
        std::env::remove_var("TASKTER_BIN");
    });
}

#[test]
fn run_bash_requires_command_argument() {
    with_temp_dir(|| {
        let err = taskter::tools::execute_tool("run_bash", &json!({})).unwrap_err();
        assert!(err.to_string().contains("command missing"));
    });
}

#[test]
fn run_bash_reports_command_failure() {
    with_temp_dir(|| {
        let err =
            taskter::tools::execute_tool("run_bash", &json!({"command": "exit 1"})).unwrap_err();
        assert!(err.to_string().contains("Command failed"));
    });
}

#[test]
fn run_python_requires_code_argument() {
    with_temp_dir(|| {
        let err = taskter::tools::execute_tool("run_python", &json!({})).unwrap_err();
        assert!(err.to_string().contains("code missing"));
    });
}

#[test]
fn run_python_reports_execution_error() {
    with_temp_dir(|| {
        let err =
            taskter::tools::execute_tool("run_python", &json!({"code": "import sys; sys.exit(1)"}))
                .unwrap_err();
        assert!(err.to_string().contains("Python execution failed"));
    });
}

#[test]
fn unknown_tool_returns_error() {
    with_temp_dir(|| {
        let err = taskter::tools::execute_tool("no_such_tool", &json!({})).unwrap_err();
        assert!(err.to_string().contains("Unknown tool"));
    });
}

#[test]
fn file_ops_create_and_read() {
    with_temp_dir(|| {
        file_ops::execute(&json!({
            "action": "create",
            "path": "note.txt",
            "content": "hello"
        }))
        .unwrap();

        let out = file_ops::execute(&json!({"action": "read", "path": "note.txt"})).unwrap();
        assert_eq!(out, "hello");
    });
}

#[test]
fn file_ops_search_returns_lines() {
    with_temp_dir(|| {
        fs::write("note.txt", "hello\nworld\nhello world").unwrap();
        let out = file_ops::execute(&json!({
            "action": "search",
            "path": "note.txt",
            "query": "world"
        }))
        .unwrap();
        assert!(out.contains("2:world"));
    });
}

#[test]
fn file_ops_update_overwrites_file() {
    with_temp_dir(|| {
        fs::write("note.txt", "old").unwrap();
        file_ops::execute(&json!({
            "action": "update",
            "path": "note.txt",
            "content": "new"
        }))
        .unwrap();
        let content = fs::read_to_string("note.txt").unwrap();
        assert_eq!(content, "new");
    });
}
