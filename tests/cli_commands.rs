use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;

fn with_temp_dir<F: FnOnce() -> T, T>(test: F) -> T {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let original_dir = std::env::current_dir().expect("cannot read current dir");
    std::env::set_current_dir(tmp.path()).expect("cannot set current dir");

    let result = test();

    std::env::set_current_dir(original_dir).expect("cannot restore current dir");
    result
}

#[test]
fn add_list_done_workflow() {
    with_temp_dir(|| {
        // Initialize board
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        // Add a task
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["add", "--title", "Test task"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Task added successfully"));

        // Verify list output contains the task
        let out = Command::cargo_bin("taskter")
            .unwrap()
            .arg("list")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("Test task"));

        // Mark the task as done
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["done", "1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("marked as done"));

        // Inspect board file
        let board: Value =
            serde_json::from_str(&fs::read_to_string(".taskter/board.json").unwrap()).unwrap();
        assert_eq!(board["tasks"][0]["status"], "Done");
    });
}

#[test]
fn add_agent_and_execute_task() {
    with_temp_dir(|| {
        // prepare board
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        // add a task
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["add", "--title", "Send email"])
            .assert()
            .success();

        // add agent with builtin tool
        Command::cargo_bin("taskter")
            .unwrap()
            .args([
                "add-agent",
                "--prompt",
                "email agent",
                "--tools",
                "email",
                "--model",
                "gpt-4o",
            ])
            .assert()
            .success();

        // assign agent to task
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["assign", "--task-id", "1", "--agent-id", "1"])
            .assert()
            .success();

        // execute the task
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["execute", "--task-id", "1"])
            .assert()
            .success();

        let board: Value =
            serde_json::from_str(&fs::read_to_string(".taskter/board.json").unwrap()).unwrap();
        assert_eq!(board["tasks"][0]["status"], "Done");
    });
}

#[test]
fn show_lists_builtin_tools() {
    with_temp_dir(|| {
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        let out = Command::cargo_bin("taskter")
            .unwrap()
            .args(["show", "tools"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("send_email"));
        assert!(output.contains("create_task"));
    });
}
