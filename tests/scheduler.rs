use std::time::Duration;
use taskter::agent::Agent;
use taskter::config::{self, ConfigOverrides};
use taskter::store::{Board, Task, TaskStatus};
use taskter::{agent, scheduler, store, tools};

mod common;
use common::disable_host_config_guard;

#[tokio::test]
async fn scheduler_executes_agent_tasks() {
    let _host_config_guard = disable_host_config_guard();
    let tmp = tempfile::tempdir().expect("tmp");
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let data_dir = tmp.path().join(taskter::config::DIR);
    std::fs::create_dir(&data_dir).unwrap();

    let config_path = tmp.path().join("config.toml");
    std::fs::write(&config_path, b"").unwrap();
    let mut overrides = ConfigOverrides::default();
    overrides.config_file = Some(config_path);
    overrides.data_dir = Some(data_dir);
    config::init(&overrides).expect("config init");

    let send_email = tools::builtin_declaration("send_email").unwrap();
    let agent = Agent {
        id: 1,
        system_prompt: "helper".into(),
        tools: vec![send_email],
        model: "gemini-2.5-flash".into(),
        provider: Some("gemini".into()),
        schedule: Some("*/1 * * * * *".into()),
        repeat: false,
    };
    agent::save_agents(std::slice::from_ref(&agent)).unwrap();

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
    config::init(&ConfigOverrides::default()).expect("reset config state");
}
