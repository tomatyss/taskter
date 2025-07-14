use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::io::Write;

mod store;
mod tui;
mod agent;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a new Taskter board
    Init,
    /// Adds a new task
    Add {
        /// The title of the task
        #[arg(short, long)]
        title: String,
        /// The description of the task
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Lists all tasks
    List,
    /// Marks a task as done
    Done {
        /// The id of the task to mark as done
        id: usize,
    },
    /// Show project information
    Show {
        #[command(subcommand)]
        what: ShowCommands,
    },
    /// Adds a new OKR
    AddOkr {
        /// The objective
        #[arg(short, long)]
        objective: String,
        /// The key results
        #[arg(short, long, num_args = 1..)]
        key_results: Vec<String>,
    },
    /// Adds a log entry
    Log {
        /// The log message
        message: String,
    },
    /// Opens the interactive board
    Board,
    /// Sets the project description
    Description {
        /// The project description
        description: String,
    },
    /// Adds a new agent
    #[command(name = "add-agent")]
    AddAgent {
        /// The system prompt for the agent
        #[arg(short, long)]
        prompt: String,
        /// The tools the agent can use
        #[arg(short, long, num_args = 1..)]
        tools: Vec<String>,
        /// The model to use for the agent
        #[arg(short, long)]
        model: String,
    },
    /// Executes a task with an agent
    Execute {
        /// The id of the task to execute
        #[arg(short, long)]
        task_id: usize,
    },
    /// Assigns an agent to a task
    Assign {
        /// The id of the task to assign
        #[arg(short, long)]
        task_id: usize,
        /// The id of the agent to assign
        #[arg(short, long)]
        agent_id: usize,
    },
}

#[derive(Subcommand)]
enum ShowCommands {
    /// Shows the project description
    Description,
    /// Shows the project OKRs
    Okrs,
    /// Shows the operation logs
    Logs,
}

fn main() -> anyhow::Result<()> {
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
        Commands::Add { title, description } => {
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
        Commands::List => {
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
        Commands::Done { id } => {
            let mut board = store::load_board()?;
            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == *id) {
                task.status = store::TaskStatus::Done;
                store::save_board(&board)?;
                println!("Task {} marked as done.", id);
            } else {
                println!("Task with id {} not found.", id);
            }
        }
        Commands::Show { what } => match what {
            ShowCommands::Description => {
                let description = fs::read_to_string(".taskter/description.md")?;
                println!("{}", description);
            }
            ShowCommands::Okrs => {
                let okrs = fs::read_to_string(".taskter/okrs.json")?;
                println!("{}", okrs);
            }
            ShowCommands::Logs => {
                let logs = fs::read_to_string(".taskter/logs.log")?;
                println!("{}", logs);
            }
        },
        Commands::AddOkr {
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
        Commands::Log { message } => {
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(".taskter/logs.log")?;
            writeln!(file, "{}", message)?;
            println!("Log added successfully.");
        }
        Commands::Board => {
            tui::run_tui()?;
        }
        Commands::Description { description } => {
            fs::write(".taskter/description.md", description)?;
            println!("Project description updated successfully.");
        }
        Commands::AddAgent {
            prompt,
            tools,
            model,
        } => {
            let mut agents = agent::load_agents()?;
            let new_agent = agent::Agent {
                id: agents.len() + 1,
                system_prompt: prompt.clone(),
                tools: tools
                    .iter()
                    .map(|t| agent::Tool { name: t.clone() })
                    .collect(),
                model: model.clone(),
            };
            agents.push(new_agent);
            agent::save_agents(&agents)?;
            println!("Agent added successfully.");
        }
        Commands::Execute { task_id } => {
            let mut board = store::load_board()?;
            let agents = agent::load_agents()?;

            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == *task_id) {
                if let Some(agent_id) = task.agent_id {
                    if let Some(agent) = agents.iter().find(|a| a.id == agent_id) {
                        match agent::execute_task(agent, task) {
                            Ok(result) => match result {
                                agent::ExecutionResult::Success => {
                                    task.status = store::TaskStatus::Done;
                                    task.comment = Some("Task completed successfully.".to_string());
                                    println!("Task {} executed successfully.", task_id);
                                }
                                agent::ExecutionResult::Failure { comment } => {
                                    task.status = store::TaskStatus::ToDo;
                                    task.comment = Some(comment);
                                    task.agent_id = None;
                                    println!("Task {} failed to execute.", task_id);
                                }
                            },
                            Err(e) => {
                                println!("Error executing task {}: {}", task_id, e);
                            }
                        }
                    } else {
                        println!("Agent with id {} not found.", agent_id);
                    }
                } else {
                    println!("Task {} is not assigned to an agent.", task_id);
                }
            } else {
                println!("Task with id {} not found.", task_id);
            }

            store::save_board(&board)?;
        }
        Commands::Assign { task_id, agent_id } => {
            let mut board = store::load_board()?;
            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == *task_id) {
                task.agent_id = Some(*agent_id);
                store::save_board(&board)?;
                println!("Agent {} assigned to task {}.", agent_id, task_id);
            } else {
                println!("Task with id {} not found.", task_id);
            }
        }
    }

    Ok(())
}