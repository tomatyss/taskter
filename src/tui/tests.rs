#[cfg(feature = "tui")]
use super::app::{App, View};
#[cfg(feature = "tui")]
use crate::agent::{self, Agent, ExecutionResult};
#[cfg(feature = "tui")]
use crate::store::{Board, Task, TaskStatus};
#[cfg(feature = "tui")]
use std::sync::Arc;

#[cfg(feature = "tui")]
fn sample_task() -> Task {
    Task {
        id: 1,
        title: "sample".into(),
        description: None,
        status: TaskStatus::ToDo,
        agent_id: None,
        comment: None,
    }
}

#[cfg(feature = "tui")]
#[test]
fn navigation_and_move_task() {
    let board = Board {
        tasks: vec![sample_task()],
    };
    let mut app = App::new(board, Vec::new());

    assert_eq!(app.selected_column, 0);
    assert_eq!(app.get_selected_task().unwrap().id, 1);

    app.move_task_to_next_column();
    assert_eq!(
        app.board.lock().unwrap().tasks[0].status,
        TaskStatus::InProgress
    );

    app.next_column();
    assert_eq!(app.selected_column, 1);
    app.move_task_to_prev_column();
    assert_eq!(app.board.lock().unwrap().tasks[0].status, TaskStatus::ToDo);
}

#[cfg(feature = "tui")]
#[test]
fn add_comment_flow() {
    let board = Board {
        tasks: vec![sample_task()],
    };
    let mut app = App::new(board, Vec::new());
    app.current_view = View::AddComment;
    app.comment_input = "note".to_string();

    if let Some(task_id) = app.get_selected_task().map(|t| t.id) {
        if let Some(task) = app
            .board
            .lock()
            .unwrap()
            .tasks
            .iter_mut()
            .find(|t| t.id == task_id)
        {
            task.comment = Some(app.comment_input.clone());
        }
    }

    assert_eq!(
        app.board.lock().unwrap().tasks[0].comment.as_deref(),
        Some("note")
    );
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn assign_agent_failure_updates_task() {
    let mut task = sample_task();
    task.agent_id = Some(1);
    let board = Board { tasks: vec![task] };
    let agent = Agent {
        id: 1,
        system_prompt: "helper".into(),
        tools: Vec::new(),
        model: "gpt-4o".into(),
    };
    let app = App::new(board, vec![agent.clone()]);
    let agent_clone = agent.clone();
    let task_clone = app.get_selected_task().unwrap();
    let board_clone = Arc::clone(&app.board);

    let result = agent::execute_task(&agent_clone, &task_clone)
        .await
        .unwrap();
    let mut board = board_clone.lock().unwrap();
    if let Some(t) = board.tasks.iter_mut().find(|t| t.id == task_clone.id) {
        match result {
            ExecutionResult::Success { comment } => {
                t.status = TaskStatus::Done;
                t.comment = Some(comment);
            }
            ExecutionResult::Failure { comment } => {
                t.status = TaskStatus::ToDo;
                t.comment = Some(comment);
                t.agent_id = None;
            }
        }
    }

    assert!(board.tasks[0].comment.is_some());
    assert_eq!(board.tasks[0].status, TaskStatus::ToDo);
    assert_eq!(board.tasks[0].agent_id, None);
}
