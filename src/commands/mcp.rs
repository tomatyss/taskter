use anyhow::Result;

use crate::cli::McpCommands;
use crate::mcp;

pub async fn handle(action: &McpCommands) -> Result<()> {
    match action {
        McpCommands::Serve => mcp::serve_stdio().await,
    }
}
