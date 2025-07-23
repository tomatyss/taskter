use crate::{agent, store};
use agent::ExecutionResult;
use chrono_tz::America::New_York;
use std::time::Duration;
use store::TaskStatus;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn run() -> anyhow::Result<()> {
    let agents = agent::load_agents()?;
    let sched = JobScheduler::new().await?;

    for ag in agents {
        if let Some(expr) = &ag.schedule {
            let job_agent = ag.clone();
            let cron_expr = expr.clone();
            let job =
                Job::new_async_tz(cron_expr, New_York, move |_id, l| {
                    let a = job_agent.clone();
                    Box::pin(async move {
                        if let Ok(mut board) = store::load_board() {
                            for task in board.tasks.iter_mut().filter(|t| {
                                t.agent_id == Some(a.id) && t.status != TaskStatus::Done
                            }) {
                                if let Ok(res) = agent::execute_task(&a, task).await {
                                    match res {
                                        ExecutionResult::Success { comment } => {
                                            task.status = TaskStatus::Done;
                                            task.comment = Some(comment);
                                        }
                                        ExecutionResult::Failure { comment } => {
                                            task.status = TaskStatus::ToDo;
                                            task.comment = Some(comment);
                                            task.agent_id = None;
                                        }
                                    }
                                }
                            }
                            let _ = store::save_board(&board);
                        }
                        if !a.repeat {
                            let _ = l.remove(&_id).await;
                            if let Ok(mut agents) = agent::load_agents() {
                                if let Some(mut_a) = agents.iter_mut().find(|x| x.id == a.id) {
                                    mut_a.schedule = None;
                                    mut_a.repeat = false;
                                    let _ = agent::save_agents(&agents);
                                }
                            }
                        }
                    })
                })?;
            sched.add(job).await?;
        }
    }

    sched.start().await?;

    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
