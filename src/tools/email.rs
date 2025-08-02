use anyhow::{anyhow, Result};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

use crate::agent::FunctionDeclaration;
use crate::config;
use crate::tools::Tool;
use std::collections::HashMap;

#[derive(Deserialize)]
struct EmailConfig {
    smtp_server: String,
    smtp_port: u16,
    username: String,
    password: String,
}

const DECL_JSON: &str = include_str!("../../tools/send_email.json");

/// Returns the function declaration for this tool.
pub fn declaration() -> FunctionDeclaration {
    serde_json::from_str(DECL_JSON).expect("invalid send_email.json")
}

/// Sends an email using `.taskter/email_config.json` for credentials.
pub fn execute(args: &Value) -> Result<String> {
    let to = args["to"].as_str().ok_or_else(|| anyhow!("to missing"))?;
    let subject = args["subject"]
        .as_str()
        .ok_or_else(|| anyhow!("subject missing"))?;
    let body = args["body"]
        .as_str()
        .ok_or_else(|| anyhow!("body missing"))?;
    match send_email(to, subject, body) {
        Ok(_) => Ok(format!(
            "Email sent to {to} with subject '{subject}' and body '{body}'"
        )),
        Err(e) => Ok(format!("Failed to send email: {e}")),
    }
}

fn send_email(to: &str, subject: &str, body: &str) -> Result<()> {
    let config_path = config::email_config_path();
    let config_str = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(_) => return Err(anyhow::anyhow!("Email configuration not found")),
    };

    let config: EmailConfig = serde_json::from_str(&config_str)?;

    let email = Message::builder()
        .from(config.username.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(body.to_string())?;

    let creds = Credentials::new(config.username, config.password);

    let mailer = SmtpTransport::relay(&config.smtp_server)?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    mailer.send(&email).map(|_| ()).map_err(|e| e.into())
}

/// Registers the tool in the provided map.
pub fn register(map: &mut HashMap<&'static str, Tool>) {
    let decl = declaration();
    map.insert(
        "send_email",
        Tool {
            declaration: decl.clone(),
            execute,
        },
    );
    map.insert(
        "email",
        Tool {
            declaration: decl,
            execute,
        },
    );
}
