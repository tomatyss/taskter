use crate::cli::TaskCommands;
use crate::{agent, store};

pub async fn handle(action: &TaskCommands) -> anyhow::Result<()> {
    match action {
        TaskCommands::Add { title, description } => {
            let mut board = store::load_board()?;
            let new_task = store::Task {
                id: board.next_task_id(),
                title: title.clone(),
                description: description.clone(),
                status: store::TaskStatus::ToDo,
                agent_id: None,
                comment: None,
            };
            board.tasks.push(new_task);
            store::save_board(&board)?;
            println!("Task added successfully.");
        }
        TaskCommands::List => {
            let board = store::load_board()?;
            for task in board.tasks {
                println!(
                    "[{}] {} - {:?} - {:?}",
                    task.id,
                    task.title,
                    task.status,
                    task.description.unwrap_or_default()
                );
            }
        }
        TaskCommands::Complete { id } => {
            let mut board = store::load_board()?;
            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == *id) {
                task.status = store::TaskStatus::Done;
                store::save_board(&board)?;
                println!("Task {id} marked as done.");
            } else {
                println!("Task with id {id} not found.");
            }
        }
        TaskCommands::Comment { task_id, comment } => {
            let mut board = store::load_board()?;
            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == *task_id) {
                task.comment = Some(comment.clone());
                store::save_board(&board)?;
                println!("Comment added to task {task_id}.");
            } else {
                println!("Task with id {task_id} not found.");
            }
        }
        TaskCommands::Execute { task_id } => {
            let mut board = store::load_board()?;
            let agents = agent::load_agents()?;

            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == *task_id) {
                if let Some(agent_id) = task.agent_id {
                    if let Some(a) = agents.iter().find(|a| a.id == agent_id) {
                        match agent::execute_task(a, Some(task)).await {
                            Ok(result) => match result {
                                agent::ExecutionResult::Success { comment } => {
                                    task.status = store::TaskStatus::Done;
                                    task.comment = Some(comment);
                                    println!("Task {task_id} executed successfully.");
                                }
                                agent::ExecutionResult::Failure { comment } => {
                                    task.status = store::TaskStatus::ToDo;
                                    task.comment = Some(comment);
                                    task.agent_id = None;
                                    println!("Task {task_id} failed to execute.");
                                }
                            },
                            Err(e) => {
                                println!("Error executing task {task_id}: {e}");
                            }
                        }
                    } else {
                        println!("Agent with id {agent_id} not found.");
                    }
                } else {
                    println!("Task {task_id} is not assigned to an agent.");
                }
            } else {
                println!("Task with id {task_id} not found.");
            }

            store::save_board(&board)?;
        }
        TaskCommands::Assign { task_id, agent_id } => {
            let mut board = store::load_board()?;
            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == *task_id) {
                task.agent_id = Some(*agent_id);
                store::save_board(&board)?;
                println!("Agent {agent_id} assigned to task {task_id}.");
            } else {
                println!("Task with id {task_id} not found.");
            }
        }
        TaskCommands::Unassign { task_id } => {
            let mut board = store::load_board()?;
            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == *task_id) {
                task.agent_id = None;
                store::save_board(&board)?;
                println!("Agent unassigned from task {task_id}.");
            } else {
                println!("Task with id {task_id} not found.");
            }
        }
    }
    Ok(())
}
