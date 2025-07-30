use crate::{agent, store};
use agent::ExecutionResult;
use chrono_tz::America::New_York;
use futures::future::join_all;
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
            let job = Job::new_async_tz(cron_expr, New_York, move |_id, l| {
                let a = job_agent.clone();
                Box::pin(async move {
                    if let Ok(mut board) = store::load_board() {
                        let tasks: Vec<usize> = board
                            .tasks
                            .iter()
                            .filter(|t| t.agent_id == Some(a.id) && t.status != TaskStatus::Done)
                            .map(|t| t.id)
                            .collect();

                        if tasks.is_empty() {
                            let _ = agent::execute_task(&a, None).await;
                        } else {
                            let task_data: Vec<(usize, store::Task)> = tasks
                                .iter()
                                .filter_map(|id| {
                                    board
                                        .tasks
                                        .iter()
                                        .find(|t| t.id == *id)
                                        .cloned()
                                        .map(|task| (*id, task))
                                })
                                .collect();

                            let handles = task_data.into_iter().map(|(id, task)| {
                                let agent_clone = a.clone();
                                tokio::spawn(async move {
                                    (id, agent::execute_task(&agent_clone, Some(&task)).await)
                                })
                            });

                            for (task_id, exec) in join_all(handles).await.into_iter().flatten() {
                                if let Ok(exec) = exec {
                                    if let Some(task_mut) =
                                        board.tasks.iter_mut().find(|t| t.id == task_id)
                                    {
                                        match exec {
                                            ExecutionResult::Success { comment } => {
                                                task_mut.status = TaskStatus::Done;
                                                task_mut.comment = Some(comment);
                                            }
                                            ExecutionResult::Failure { comment } => {
                                                task_mut.status = TaskStatus::ToDo;
                                                task_mut.comment = Some(comment);
                                                task_mut.agent_id = None;
                                            }
                                        }
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
