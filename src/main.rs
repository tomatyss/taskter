use clap::Parser;

use taskter::cli::{Cli, Commands};
use taskter::commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
    }

    Ok(())
}
