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
    /// Task management commands
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },
    /// Agent management commands
    Agent {
        #[command(subcommand)]
        action: AgentCommands,
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
    /// Lists all built-in agent tools
    Tools,
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// Adds a new agent
    Add {
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
    /// Lists all agents
    List,
    /// Removes an agent by id
    Remove {
        /// The id of the agent to delete
        #[arg(long)]
        id: usize,
    },
    /// Updates an agent's prompt and tools
    Update {
        /// The id of the agent to update
        #[arg(long)]
        id: usize,
        /// The new system prompt for the agent
        #[arg(short, long)]
        prompt: String,
        /// The new tools the agent can use
        #[arg(short, long, num_args = 1..)]
        tools: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum TaskCommands {
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
    /// Marks a task as complete
    Complete {
        /// The id of the task to mark as done
        #[arg(long)]
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
