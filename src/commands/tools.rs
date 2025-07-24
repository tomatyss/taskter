use crate::cli::ToolCommands;
use crate::tools;

pub fn handle(action: &ToolCommands) -> anyhow::Result<()> {
    match action {
        ToolCommands::List => {
            for t in tools::builtin_names() {
                println!("{t}");
            }
        }
    }
    Ok(())
}
