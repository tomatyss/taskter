use std::fs;

use serde_json::json;
use taskter::agent::{self, Agent, ExecutionResult, FunctionDeclaration};
use taskter::store::{self, Board, KeyResult, Okr, Task, TaskStatus};

// Helper that creates a temporary workspace and changes the current directory to it.
fn with_temp_dir<F: FnOnce() -> T, T>(test: F) -> T {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let original_dir = std::env::current_dir().expect("cannot read current dir");
    std::env::set_current_dir(tmp.path()).expect("cannot set current dir");

    // Each Taskter invocation expects the .taskter directory to exist. We'll ensure it's present.
    fs::create_dir(".taskter").unwrap();

    let result = test();

    std::env::set_current_dir(original_dir).expect("cannot restore current dir");
    result
}

#[test]
fn board_roundtrip_persists_tasks() {
    with_temp_dir(|| {
        // Given
        let task = Task {
            id: 1,
            title: "Write docs".to_string(),
            description: Some("Write the project documentation".to_string()),
            status: TaskStatus::ToDo,
            agent_id: None,
            comment: None,
        };

        let board = Board {
            tasks: vec![task.clone()],
        };

        // When
        store::save_board(&board).expect("failed to save board");
        let loaded = store::load_board().expect("failed to load board");

        // Then
        assert_eq!(loaded, board);
        assert_eq!(loaded.tasks[0], task);
    });
}

#[test]
fn okr_roundtrip_persists_data() {
    with_temp_dir(|| {
        // Given
        let okr = Okr {
            objective: "Improve UX".to_string(),
            key_results: vec![KeyResult {
                name: "Reduce load time".to_string(),
                progress: 0.2,
            }],
        };

        // When
        store::save_okrs(&[okr.clone()]).expect("failed to save okrs");
        let loaded = store::load_okrs().expect("failed to load okrs");

        // Then
        assert_eq!(loaded, vec![okr]);
    });
}

#[test]
fn comment_roundtrip_persists_changes() {
    with_temp_dir(|| {
        let mut board = Board {
            tasks: vec![Task {
                id: 1,
                title: "Test".to_string(),
                description: None,
                status: TaskStatus::ToDo,
                agent_id: None,
                comment: None,
            }],
        };

        store::save_board(&board).expect("failed to save board");

        board.add_comment(1, "note".to_string());
        store::save_board(&board).expect("failed to save board");

        let loaded = store::load_board().expect("failed to load board");
        assert_eq!(loaded.tasks[0].comment.as_deref(), Some("note"));
    });
}

#[tokio::test]
async fn agent_executes_email_task_successfully() {
    // Given
    let agent = Agent {
        id: 1,
        system_prompt: "You are an email sender".into(),
        tools: vec![FunctionDeclaration {
            name: "send_email".into(),
            description: Some("".into()),
            parameters: json!({}),
        }],
        model: "gpt-4o".into(),
    };

    let task = Task {
        id: 1,
        title: "Send email to John".into(),
        description: None,
        status: TaskStatus::ToDo,
        agent_id: Some(1),
        comment: None,
    };

    // When
    let result = agent::execute_task(&agent, &task)
        .await
        .expect("execution failed");

    // Then
    assert!(matches!(result, ExecutionResult::Success { .. }));
}

#[tokio::test]
async fn agent_execution_fails_without_tool() {
    // Given
    let agent = Agent {
        id: 1,
        system_prompt: "General agent".into(),
        tools: vec![],
        model: "gpt-4o".into(),
    };

    let task = Task {
        id: 1,
        title: "Send email to John".into(),
        description: None,
        status: TaskStatus::ToDo,
        agent_id: Some(1),
        comment: None,
    };

    // When
    let result = agent::execute_task(&agent, &task)
        .await
        .expect("execution failed");

    // Then
    assert!(matches!(result, ExecutionResult::Failure { .. }));
}

#[test]
fn run_python_tool_executes_code() {
    let result = taskter::tools::execute_tool("run_python", &json!({ "code": "print(40 + 2)" }))
        .expect("execution failed");
    assert_eq!(result.trim(), "42");
}
