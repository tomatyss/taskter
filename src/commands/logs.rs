use std::fs;
use std::io::Write;

use chrono::Local;

use crate::cli::LogCommands;
use crate::config;

pub fn handle(action: &LogCommands) -> anyhow::Result<()> {
    match action {
        LogCommands::Add { message } => {
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(config::log_path())?;
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(file, "[{timestamp}] {message}")?;
            println!("Log added successfully.");
        }
        LogCommands::List => {
            let logs = fs::read_to_string(config::log_path())?;
            println!("{logs}");
        }
    }
    Ok(())
}
