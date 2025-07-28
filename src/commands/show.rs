use std::fs;

use crate::cli::ShowCommands;
use crate::config;

pub fn handle(what: &ShowCommands) -> anyhow::Result<()> {
    match what {
        ShowCommands::Description => {
            let description = fs::read_to_string(config::description_path())?;
            println!("{description}");
        }
    }
    Ok(())
}
