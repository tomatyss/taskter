use clap::Parser;

use taskter::cli::{Cli, Commands};
use taskter::commands;
use taskter::config;

// A multi-threaded Tokio runtime is unnecessary for a command-line application that
// mainly executes short-lived, mostly single-threaded tasks.  Using the default
// runtime spawns a worker thread per core which noticeably increases the memory
// footprint of the binary and might lead to the process being killed on
// constrained environments when users only want to display `--help` or `--version`.
//
// Switching to the `current_thread` flavour keeps the runtime single-threaded and
// lightweight while still allowing us to `.await` asynchronous operations when
// they are actually requested by the user.
//
// If in the future true parallel execution becomes necessary we can either
// create a dedicated multi-threaded runtime only for that specific operation or
// revisit this decision.
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    config::init(&cli.config)?;

    match &cli.command {
        Commands::Init => commands::init::run()?,
        Commands::Task { action } => commands::task::handle(action).await?,
        Commands::Agent { action } => commands::agent::handle(action).await?,
        Commands::Show { what } => commands::show::handle(what)?,
        Commands::Okrs { action } => commands::okrs::handle(action)?,
        Commands::Logs { action } => commands::logs::handle(action)?,
        Commands::Tools { action } => commands::tools::handle(action)?,
        Commands::Scheduler { action } => commands::scheduler::handle(action).await?,
        Commands::Board => commands::board::run()?,
        Commands::Description { description } => commands::description::set(description)?,
        Commands::Mcp { action } => commands::mcp::handle(action).await?,
    }

    Ok(())
}
