use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Adds a comment to a task
    Comment {
        /// The id of the task to comment on
        #[arg(short, long)]
        task_id: usize,
        /// The comment text
        #[arg(short, long)]
        comment: String,
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
    /// Deletes an agent by id
    #[command(name = "delete-agent")]
    DeleteAgent {
        /// The id of the agent to delete
        #[arg(short, long)]
        agent_id: usize,
    },
}

#[derive(Subcommand)]
pub enum ShowCommands {
    /// Shows the project description
    Description,
    /// Shows the project OKRs
    Okrs,
    /// Shows the operation logs
    Logs,
    /// Lists all agents
    Agents,
}
