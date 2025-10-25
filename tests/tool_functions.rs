#![allow(clippy::used_underscore_binding)]

use assert_cmd::Command;
use serde_json::json;
use std::fs;

use taskter::tools::get_description;

use mockito::{Matcher, Server};

mod common;
pub use common::with_temp_dir;

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
fn send_email_requires_arguments() {
    with_temp_dir(|| {
        let result = taskter::tools::execute_tool("send_email", &json!({}));
        assert!(result.is_err());
    });
}

#[test]
fn send_email_reports_missing_configuration() {
    with_temp_dir(|| {
        let err = taskter::tools::execute_tool(
            "send_email",
            &json!({"to": "user@example.com", "subject": "hi", "body": "hello"}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("Email configuration not found"));
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
fn file_ops_requires_action() {
    with_temp_dir(|| {
        let err = taskter::tools::execute_tool("file_ops", &json!({})).unwrap_err();
        assert!(err.to_string().contains("action missing"));
    });
}

#[test]
fn file_ops_unknown_action() {
    with_temp_dir(|| {
        let err = taskter::tools::execute_tool("file_ops", &json!({"action": "foo"})).unwrap_err();
        assert!(err.to_string().contains("unknown action"));
    });
}

#[test]
fn file_ops_create_read_update() {
    with_temp_dir(|| {
        // Create a new file
        let out = taskter::tools::execute_tool(
            "file_ops",
            &json!({"action": "create", "path": "a.txt", "content": "hello"}),
        )
        .unwrap();
        assert_eq!(out, "Created a.txt");
        // Read the file
        let out =
            taskter::tools::execute_tool("file_ops", &json!({"action": "read", "path": "a.txt"}))
                .unwrap();
        assert_eq!(out, "hello");
        // Update the file
        let out = taskter::tools::execute_tool(
            "file_ops",
            &json!({"action": "update", "path": "a.txt", "content": "world"}),
        )
        .unwrap();
        assert_eq!(out, "Updated a.txt");
        let content = fs::read_to_string("a.txt").unwrap();
        assert_eq!(content, "world");
    });
}

#[test]
fn file_ops_search() {
    with_temp_dir(|| {
        // Create files with and without the query
        fs::write("match.txt", "find me").unwrap();
        fs::write("other.txt", "nothing").unwrap();
        let out =
            taskter::tools::execute_tool("file_ops", &json!({"action": "search", "query": "find"}))
                .unwrap();
        assert!(out.contains("match.txt"));
        assert!(!out.contains("other.txt"));
        // Search for non-existing string
        let out = taskter::tools::execute_tool(
            "file_ops",
            &json!({"action": "search", "query": "absent"}),
        )
        .unwrap();
        assert_eq!(out, "No matches found");
    });
}
