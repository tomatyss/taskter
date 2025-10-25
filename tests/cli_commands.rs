use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;

mod common;
pub use common::with_temp_dir;

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
            .args(["task", "add", "--title", "Test task"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Task added successfully"));

        // Verify list output contains the task
        let out = Command::cargo_bin("taskter")
            .unwrap()
            .args(["task", "list"])
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
            .args(["task", "complete", "--id", "1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("marked as done"));

        // Inspect board file
        let board: Value =
            serde_json::from_str(&fs::read_to_string(taskter::config::BOARD_FILE).unwrap())
                .unwrap();
        assert_eq!(board["tasks"][0]["status"], "Done");
    });
}

#[test]
fn board_command_starts_and_quits() {
    use expectrl::{Eof, Expect, Session};
    use std::process::Command as PCommand;
    use std::thread;
    use std::time::Duration;

    with_temp_dir(|| {
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        let bin = Command::cargo_bin("taskter").unwrap();
        let path = bin.get_program().to_owned();

        let mut cmd = PCommand::new(path);
        cmd.arg("board");
        let mut session = Session::spawn(cmd).unwrap();

        thread::sleep(Duration::from_millis(200));
        session.send("q").unwrap();
        session.expect(Eof).unwrap();
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
            .args(["task", "add", "--title", "Send email"])
            .assert()
            .success();

        // add agent with builtin tool
        Command::cargo_bin("taskter")
            .unwrap()
            .args([
                "agent",
                "add",
                "--prompt",
                "email agent",
                "--tools",
                "email",
                "--model",
                "gemini-2.5-flash",
            ])
            .assert()
            .success();

        // assign agent to task
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["task", "assign", "--task-id", "1", "--agent-id", "1"])
            .assert()
            .success();

        // execute the task
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["task", "execute", "--task-id", "1"])
            .assert()
            .success();

        let board: Value =
            serde_json::from_str(&fs::read_to_string(taskter::config::BOARD_FILE).unwrap())
                .unwrap();
        assert_eq!(board["tasks"][0]["status"], "Done");
    });
}

#[test]
fn unassign_removes_agent() {
    with_temp_dir(|| {
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        Command::cargo_bin("taskter")
            .unwrap()
            .args(["task", "add", "--title", "Test task"])
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
                "email",
                "--model",
                "gemini-2.5-flash",
            ])
            .assert()
            .success();

        Command::cargo_bin("taskter")
            .unwrap()
            .args(["task", "assign", "--task-id", "1", "--agent-id", "1"])
            .assert()
            .success();

        Command::cargo_bin("taskter")
            .unwrap()
            .args(["task", "unassign", "--task-id", "1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("unassigned"));

        let board: Value =
            serde_json::from_str(&fs::read_to_string(taskter::config::BOARD_FILE).unwrap())
                .unwrap();
        assert!(board["tasks"][0]["agent_id"].is_null());
    });
}
#[test]
fn list_and_delete_agents() {
    with_temp_dir(|| {
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        // add an agent
        Command::cargo_bin("taskter")
            .unwrap()
            .args([
                "agent",
                "add",
                "--prompt",
                "helper",
                "--tools",
                "email",
                "--model",
                "gemini-2.5-flash",
            ])
            .assert()
            .success();

        // list agents
        let out = Command::cargo_bin("taskter")
            .unwrap()
            .args(["agent", "list"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("1: helper (model: gemini-2.5-flash, tools: send_email)"));

        // delete agent
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["agent", "remove", "--id", "1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Agent 1 deleted."));

        let agents: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(taskter::config::AGENTS_FILE).unwrap())
                .unwrap();
        assert!(agents.is_empty());
    });
}

#[test]
fn update_agent_changes_configuration() {
    with_temp_dir(|| {
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        // add an agent
        Command::cargo_bin("taskter")
            .unwrap()
            .args([
                "agent",
                "add",
                "--prompt",
                "helper",
                "--tools",
                "email",
                "--model",
                "gemini-2.5-flash",
            ])
            .assert()
            .success();

        // update the agent's tools and model only
        Command::cargo_bin("taskter")
            .unwrap()
            .args([
                "agent",
                "update",
                "--id",
                "1",
                "--tools",
                "taskter_task",
                "--model",
                "gemini-2.5-pro",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Agent 1 updated."));

        let agents: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(taskter::config::AGENTS_FILE).unwrap())
                .unwrap();
        assert_eq!(agents[0]["system_prompt"], "helper");
        assert_eq!(agents[0]["tools"][0]["name"], "taskter_task");
        assert_eq!(agents[0]["model"], "gemini-2.5-pro");

        // update the agent's prompt only
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["agent", "update", "--id", "1", "--prompt", "new helper"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Agent 1 updated."));

        let agents: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(taskter::config::AGENTS_FILE).unwrap())
                .unwrap();
        assert_eq!(agents[0]["system_prompt"], "new helper");
        assert_eq!(agents[0]["tools"][0]["name"], "taskter_task");
        assert_eq!(agents[0]["model"], "gemini-2.5-pro");
    });
}

#[test]
fn add_okr_log_and_description() {
    with_temp_dir(|| {
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        // add okr
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["okrs", "add", "-o", "Improve UI", "-k", "Faster", "Better"])
            .assert()
            .success()
            .stdout(predicate::str::contains("OKR added successfully"));

        let okrs: Value =
            serde_json::from_str(&fs::read_to_string(taskter::config::OKRS_FILE).unwrap()).unwrap();
        assert_eq!(okrs.as_array().unwrap().len(), 1);
        assert_eq!(okrs[0]["objective"], "Improve UI");

        // add log entry
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["logs", "add", "Initial commit"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Log added successfully"));

        let logs = fs::read_to_string(taskter::config::LOG_FILE).unwrap();
        assert!(logs.contains("Initial commit"));

        // update description
        Command::cargo_bin("taskter")
            .unwrap()
            .args(["description", "A great project"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Project description updated successfully",
            ));

        let desc = fs::read_to_string(taskter::config::DESCRIPTION_FILE).unwrap();
        assert_eq!(desc, "A great project");
    });
}

#[test]
fn show_tools_lists_builtins() {
    with_temp_dir(|| {
        Command::cargo_bin("taskter")
            .unwrap()
            .arg("init")
            .assert()
            .success();

        let out = Command::cargo_bin("taskter")
            .unwrap()
            .args(["tools", "list"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("taskter_task"));
        assert!(output.contains("run_bash"));
        assert!(output.contains("web_search"));
    });
}

#[test]
fn schedule_agent_updates_file() {
    with_temp_dir(|| {
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
                "email",
                "--model",
                "gemini-2.5-flash",
            ])
            .assert()
            .success();

        Command::cargo_bin("taskter")
            .unwrap()
            .args([
                "agent",
                "schedule",
                "set",
                "--id",
                "1",
                "--cron",
                "*/5 * * * * *",
            ])
            .assert()
            .success();

        let agents: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(taskter::config::AGENTS_FILE).unwrap())
                .unwrap();
        assert_eq!(agents[0]["schedule"], "*/5 * * * * *");

        let out = Command::cargo_bin("taskter")
            .unwrap()
            .args(["agent", "schedule", "list"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let output = String::from_utf8(out).unwrap();
        assert!(output.contains("*/5 * * * * *"));

        Command::cargo_bin("taskter")
            .unwrap()
            .args(["agent", "schedule", "remove", "--id", "1"])
            .assert()
            .success();

        let agents: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(taskter::config::AGENTS_FILE).unwrap())
                .unwrap();
        assert!(agents[0]["schedule"].is_null());
    });
}
