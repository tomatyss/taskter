use std::time::Duration;
use taskter::agent::Agent;
use taskter::store::{Board, Task, TaskStatus};
use taskter::{agent, scheduler, store, tools};

#[tokio::test]
async fn scheduler_executes_agent_tasks() {
    let tmp = tempfile::tempdir().expect("tmp");
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    std::fs::create_dir(taskter::config::DIR).unwrap();

    let send_email = tools::builtin_declaration("send_email").unwrap();
    let agent = Agent {
        id: 1,
        system_prompt: "helper".into(),
        tools: vec![send_email],
        model: "gemini-2.5-flash".into(),
        schedule: Some("*/1 * * * * *".into()),
        repeat: false,
    };
    agent::save_agents(&[agent.clone()]).unwrap();

    let tasks = vec![
        Task {
            id: 1,
            title: "t1".into(),
            description: None,
            status: TaskStatus::ToDo,
            agent_id: Some(1),
            comment: None,
        },
        Task {
            id: 2,
            title: "t2".into(),
            description: None,
            status: TaskStatus::ToDo,
            agent_id: Some(1),
            comment: None,
        },
    ];
    store::save_board(&Board { tasks }).unwrap();

    let handle = tokio::spawn(async { scheduler::run().await.unwrap() });
    tokio::time::sleep(Duration::from_secs(3)).await;
    handle.abort();
    let _ = handle.await;

    let board = store::load_board().unwrap();
    for task in board.tasks {
        assert_eq!(task.status, TaskStatus::Done);
    }

    std::env::set_current_dir(orig).unwrap();
}
