#![allow(clippy::manual_string_new, clippy::used_underscore_binding)]

use serde_json::json;
use taskter::agent::{self, Agent, ExecutionResult, FunctionDeclaration};
use taskter::store::{self, Board, KeyResult, Okr, Task, TaskStatus};

mod common;
pub use common::with_temp_dir;

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
        store::save_okrs(std::slice::from_ref(&okr)).expect("failed to save okrs");
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

        board.tasks[0].comment = Some("note".to_string());
        store::save_board(&board).expect("failed to save board");

        let loaded = store::load_board().expect("failed to load board");
        assert_eq!(loaded.tasks[0].comment.as_deref(), Some("note"));
    });
}

#[tokio::test(flavor = "current_thread")]
async fn agent_executes_email_task_successfully() {
    std::env::remove_var("GEMINI_API_KEY");
    // Given
    let agent = Agent {
        id: 1,
        system_prompt: "You are an email sender".into(),
        tools: vec![FunctionDeclaration {
            name: "send_email".into(),
            description: Some("".into()),
            parameters: json!({}),
        }],
        model: "gemini-2.5-flash".into(),
        schedule: None,
        repeat: false,
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
    let result = agent::execute_task(&agent, Some(&task))
        .await
        .expect("execution failed");

    // Then
    assert!(matches!(result, ExecutionResult::Success { .. }));
}

#[tokio::test(flavor = "current_thread")]
async fn agent_execution_fails_without_tool() {
    // Given
    let agent = Agent {
        id: 1,
        system_prompt: "General agent".into(),
        tools: vec![],
        model: "gemini-2.5-flash".into(),
        schedule: None,
        repeat: false,
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
    let result = agent::execute_task(&agent, Some(&task))
        .await
        .expect("execution failed");

    // Then
    assert!(matches!(result, ExecutionResult::Failure { .. }));
}

#[tokio::test(flavor = "current_thread")]
async fn agent_execution_fails_on_network_error_without_tool() {
    std::env::set_var("GEMINI_API_KEY", "dummy");
    std::env::set_var("https_proxy", "http://127.0.0.1:9");

    let agent = Agent {
        id: 1,
        system_prompt: "General agent".into(),
        tools: vec![],
        model: "gemini-2.5-flash".into(),
        schedule: None,
        repeat: false,
    };

    let task = Task {
        id: 1,
        title: "Test".into(),
        description: None,
        status: TaskStatus::ToDo,
        agent_id: Some(1),
        comment: None,
    };

    let result = agent::execute_task(&agent, Some(&task))
        .await
        .expect("execution failed");

    assert!(matches!(result, ExecutionResult::Failure { .. }));

    std::env::remove_var("GEMINI_API_KEY");
    std::env::remove_var("https_proxy");
}

#[test]
fn run_python_tool_executes_code() {
    let result = taskter::tools::execute_tool("run_python", &json!({ "code": "print(40 + 2)" }))
        .expect("execution failed");
    assert_eq!(result.trim(), "42");
}
