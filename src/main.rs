use chrono::Local;
use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::Path;

use taskter::cli::*;
mod agent;
mod store;
mod tools;
mod tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            let path = Path::new(".taskter");
            if path.exists() {
                println!("Taskter board already initialized.");
            } else {
                fs::create_dir(path)?;
                fs::write(path.join("description.md"), "# Project Description")?;
                fs::write(path.join("okrs.json"), "[]")?;
                fs::write(path.join("logs.log"), "")?;
                fs::write(path.join("board.json"), r#"{ "tasks": [] }"#)?;
                fs::write(path.join("agents.json"), "[]")?;
                println!("Taskter board initialized.");
            }
        }
        Commands::Task { action } => match action {
            TaskCommands::Add { title, description } => {
                let mut board = store::load_board()?;
                let new_task = store::Task {
                    id: board.tasks.len() + 1,
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
                        if let Some(agent) = agents.iter().find(|a| a.id == agent_id) {
                            match agent::execute_task(agent, task).await {
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
        },
        Commands::Agent { action } => match action {
            AgentCommands::Add {
                prompt,
                tools,
                model,
            } => {
                let mut agents = agent::load_agents()?;
                let mut function_declarations = Vec::new();
                for spec in tools {
                    let decl = if Path::new(spec).exists() {
                        let tool_content = fs::read_to_string(spec)?;
                        let tool_json: serde_json::Value = serde_json::from_str(&tool_content)?;
                        serde_json::from_value(tool_json)?
                    } else if let Some(built) = tools::builtin_declaration(spec) {
                        built
                    } else {
                        return Err(anyhow::anyhow!(format!("Unknown tool: {spec}")));
                    };
                    function_declarations.push(decl);
                }

                let new_agent = agent::Agent {
                    id: agents.len() + 1,
                    system_prompt: prompt.clone(),
                    tools: function_declarations,
                    model: model.clone(),
                };
                agents.push(new_agent);
                agent::save_agents(&agents)?;
                println!("Agent added successfully.");
            }
            AgentCommands::List => {
                let agents = agent::list_agents()?;
                for a in agents {
                    println!("{}: {}", a.id, a.system_prompt);
                }
            }
            AgentCommands::Remove { id } => {
                agent::delete_agent(*id)?;
                println!("Agent {id} deleted.");
            }
            AgentCommands::Update { id, prompt, tools } => {
                let mut function_declarations = Vec::new();
                for spec in tools {
                    let decl = if Path::new(spec).exists() {
                        let tool_content = fs::read_to_string(spec)?;
                        let tool_json: serde_json::Value = serde_json::from_str(&tool_content)?;
                        serde_json::from_value(tool_json)?
                    } else if let Some(built) = tools::builtin_declaration(spec) {
                        built
                    } else {
                        return Err(anyhow::anyhow!(format!("Unknown tool: {spec}")));
                    };
                    function_declarations.push(decl);
                }

                agent::update_agent(*id, prompt.clone(), function_declarations)?;
                println!("Agent {id} updated.");
            }
        },
        Commands::Show { what } => match what {
            ShowCommands::Description => {
                let description = fs::read_to_string(".taskter/description.md")?;
                println!("{description}");
            }
        },
        Commands::Okrs { action } => match action {
            OkrCommands::Add {
                objective,
                key_results,
            } => {
                let mut okrs = store::load_okrs()?;
                let new_okr = store::Okr {
                    objective: objective.clone(),
                    key_results: key_results
                        .iter()
                        .map(|kr| store::KeyResult {
                            name: kr.clone(),
                            progress: 0.0,
                        })
                        .collect(),
                };
                okrs.push(new_okr);
                store::save_okrs(&okrs)?;
                println!("OKR added successfully.");
            }
            OkrCommands::List => {
                let okrs = fs::read_to_string(".taskter/okrs.json")?;
                println!("{okrs}");
            }
        },
        Commands::Logs { action } => match action {
            LogCommands::Add { message } => {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(".taskter/logs.log")?;
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                writeln!(file, "[{timestamp}] {message}")?;
                println!("Log added successfully.");
            }
            LogCommands::List => {
                let logs = fs::read_to_string(".taskter/logs.log")?;
                println!("{logs}");
            }
        },
        Commands::Tools { action } => match action {
            ToolCommands::List => {
                for t in tools::builtin_names() {
                    println!("{t}");
                }
            }
        },
        Commands::Board => {
            tui::run_tui()?;
        }
        Commands::Description { description } => {
            fs::write(".taskter/description.md", description)?;
            println!("Project description updated successfully.");
        }
    }

    Ok(())
}
