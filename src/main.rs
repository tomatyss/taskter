use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::io::Write;

mod store;
mod tui;

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
    }

    Ok(())
}

