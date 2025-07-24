use crate::cli::SchedulerCommands;
use crate::scheduler;

pub async fn handle(action: &SchedulerCommands) -> anyhow::Result<()> {
    match action {
        SchedulerCommands::Run => {
            scheduler::run().await?;
        }
    }
    Ok(())
}
