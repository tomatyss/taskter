//! Command-line interface definitions for Taskter.

use clap::{Parser, Subcommand};

use crate::config::ConfigOverrides;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub config: ConfigOverrides,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
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
    /// Manage OKRs
    #[command(visible_alias = "okr")]
    Okrs {
        #[command(subcommand)]
        action: OkrCommands,
    },
    /// Manage logs
    #[command(visible_alias = "log")]
    Logs {
        #[command(subcommand)]
        action: LogCommands,
    },
    /// Manage built-in tools
    Tools {
        #[command(subcommand)]
        action: ToolCommands,
    },
    /// Run the agent scheduler
    Scheduler {
        #[command(subcommand)]
        action: SchedulerCommands,
    },
    /// Opens the interactive board
    #[command(visible_alias = "ui")]
    Board,
    /// Sets the project description
    #[command(visible_alias = "set-desc")]
    Description {
        /// The project description
        description: String,
    },
    /// Run the MCP (Model Context Protocol) server
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum ShowCommands {
    /// Shows the project description
    Description,
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum OkrCommands {
    /// Adds a new OKR
    Add {
        /// The objective
        #[arg(short, long)]
        objective: String,
        /// The key results
        #[arg(short, long, num_args = 1..)]
        key_results: Vec<String>,
    },
    /// Lists all OKRs
    List,
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum LogCommands {
    /// Adds a log entry
    Add {
        /// The log message
        message: String,
    },
    /// Lists log entries
    List,
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum ToolCommands {
    /// Lists built-in tools
    List,
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
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
        /// The provider to use for the agent (e.g. openai, gemini, ollama)
        #[arg(long)]
        provider: Option<String>,
    },
    /// Lists all agents
    List,
    /// Lists running agents
    Running,
    /// Removes an agent by id
    Remove {
        /// The id of the agent to delete
        #[arg(long)]
        id: usize,
    },
    /// Updates an agent's configuration. Each field is optional.
    Update {
        /// The id of the agent to update
        #[arg(long)]
        id: usize,
        /// The new system prompt for the agent
        #[arg(short, long)]
        prompt: Option<String>,
        /// The new tools the agent can use
        #[arg(short, long, num_args = 1..)]
        tools: Option<Vec<String>>,
        /// The new model for the agent
        #[arg(short, long)]
        model: Option<String>,
        /// The new provider for the agent
        #[arg(long)]
        provider: Option<String>,
    },
    /// Schedule operations for an agent
    Schedule {
        #[command(subcommand)]
        action: ScheduleCommands,
    },
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum ScheduleCommands {
    /// Set a cron expression for an agent
    Set {
        #[arg(long)]
        id: usize,
        #[arg(long)]
        cron: String,
        #[arg(long)]
        once: bool,
    },
    /// List scheduled agents
    List,
    /// Remove a schedule from an agent
    Remove {
        #[arg(long)]
        id: usize,
    },
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum SchedulerCommands {
    /// Run the scheduler loop
    Run,
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
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
    /// Unassigns any agent from a task
    Unassign {
        /// The id of the task to unassign
        #[arg(short, long)]
        task_id: usize,
    },
}

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum McpCommands {
    /// Serve MCP over stdio
    Serve,
}
