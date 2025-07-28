#![cfg(feature = "tui")]

use taskter::agent::Agent;
use taskter::store::{Board, Task, TaskStatus};
use taskter::tui::app::App;

mod common;
use common::with_temp_dir;

#[test]
fn navigation_cycles_through_columns_and_tasks() {
    with_temp_dir(|| {
        let board = Board {
            tasks: vec![
                Task {
                    id: 1,
                    title: "A".into(),
                    description: None,
                    status: TaskStatus::ToDo,
                    agent_id: None,
                    comment: None,
                },
                Task {
                    id: 2,
                    title: "B".into(),
                    description: None,
                    status: TaskStatus::InProgress,
                    agent_id: None,
                    comment: None,
                },
                Task {
                    id: 3,
                    title: "C".into(),
                    description: None,
                    status: TaskStatus::Done,
                    agent_id: None,
                    comment: None,
                },
            ],
        };
        let mut app = App::new(board, Vec::<Agent>::new());
        assert_eq!(app.selected_column, 0);
        assert_eq!(app.get_selected_task().unwrap().id, 1);
        app.next_column();
        assert_eq!(app.selected_column, 1);
        assert_eq!(app.get_selected_task().unwrap().id, 2);
        app.next_column();
        assert_eq!(app.selected_column, 2);
        assert_eq!(app.get_selected_task().unwrap().id, 3);
        app.next_column();
        assert_eq!(app.selected_column, 0);
    });
}

#[test]
fn moving_task_updates_status() {
    with_temp_dir(|| {
        let board = Board {
            tasks: vec![Task {
                id: 1,
                title: "T".into(),
                description: None,
                status: TaskStatus::ToDo,
                agent_id: None,
                comment: None,
            }],
        };
        let mut app = App::new(board, Vec::<Agent>::new());
        app.move_task_to_next_column();
        assert_eq!(
            app.board.lock().unwrap().tasks[0].status,
            TaskStatus::InProgress
        );
        app.next_column();
        app.move_task_to_next_column();
        assert_eq!(app.board.lock().unwrap().tasks[0].status, TaskStatus::Done);
        app.next_column();
        app.move_task_to_prev_column();
        app.prev_column();
        assert_eq!(
            app.board.lock().unwrap().tasks[0].status,
            TaskStatus::InProgress
        );
    });
}

#[test]
fn unassign_selected_task_clears_agent() {
    with_temp_dir(|| {
        let board = Board {
            tasks: vec![Task {
                id: 1,
                title: "T".into(),
                description: None,
                status: TaskStatus::ToDo,
                agent_id: Some(1),
                comment: None,
            }],
        };
        let mut app = App::new(board, Vec::<Agent>::new());
        assert_eq!(app.board.lock().unwrap().tasks[0].agent_id, Some(1));
        app.unassign_selected_task();
        assert!(app.board.lock().unwrap().tasks[0].agent_id.is_none());
    });
}
