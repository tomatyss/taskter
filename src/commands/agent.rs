use std::fs;
use std::path::Path;

use crate::agent::FunctionDeclaration;
use crate::cli::{AgentCommands, ScheduleCommands};
use crate::{agent as agent_model, tools};

pub fn parse_tool_specs(specs: &[String]) -> anyhow::Result<Vec<FunctionDeclaration>> {
    let mut function_declarations = Vec::new();
    for spec in specs {
        let decl = if Path::new(spec).exists() {
            let tool_content = fs::read_to_string(spec)?;
            let tool_json: serde_json::Value = serde_json::from_str(&tool_content)?;
            serde_json::from_value(tool_json)?
        } else if let Some(built) = tools::builtin_declaration(spec) {
            built
        } else {
            return Err(anyhow::anyhow!(format!("Unknown tool: {spec}")));
        };
        function_declarations.push(decl);
    }
    Ok(function_declarations)
}

pub async fn handle(action: &AgentCommands) -> anyhow::Result<()> {
    match action {
        AgentCommands::Add {
            prompt,
            tools,
            model,
        } => {
            let mut agents = agent_model::load_agents()?;
            let function_declarations = parse_tool_specs(tools)?;
            let new_agent = agent_model::Agent {
                id: agents.len() + 1,
                system_prompt: prompt.clone(),
                tools: function_declarations,
                model: model.clone(),
                schedule: None,
                repeat: false,
            };
            agents.push(new_agent);
            agent_model::save_agents(&agents)?;
            println!("Agent added successfully.");
        }
        AgentCommands::List => {
            let agents = agent_model::list_agents()?;
            for a in agents {
                let tool_names = a
                    .tools
                    .iter()
                    .map(|t| t.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!(
                    "{}: {} (model: {}, tools: {})",
                    a.id, a.system_prompt, a.model, tool_names
                );
            }
        }
        AgentCommands::Remove { id } => {
            agent_model::delete_agent(*id)?;
            println!("Agent {id} deleted.");
        }
        AgentCommands::Update {
            id,
            prompt,
            tools,
            model,
        } => {
            let function_declarations = parse_tool_specs(tools)?;
            agent_model::update_agent(*id, prompt.clone(), function_declarations, model.clone())?;
            println!("Agent {id} updated.");
        }
        AgentCommands::Schedule { action } => match action {
            ScheduleCommands::Set { id, cron, once } => {
                if tokio_cron_scheduler::Job::new_async(cron, |_id, _| Box::pin(async {})).is_err()
                {
                    println!("Invalid cron expression");
                } else {
                    let mut agents = agent_model::load_agents()?;
                    if let Some(a) = agents.iter_mut().find(|a| a.id == *id) {
                        a.schedule = Some(cron.clone());
                        a.repeat = !*once;
                        agent_model::save_agents(&agents)?;
                        println!("Agent {id} scheduled.");
                    } else {
                        println!("Agent {id} not found.");
                    }
                }
            }
            ScheduleCommands::List => {
                let agents = agent_model::load_agents()?;
                for a in agents.into_iter().filter(|a| a.schedule.is_some()) {
                    println!(
                        "{}: {} (repeat: {})",
                        a.id,
                        a.schedule.as_deref().unwrap_or(""),
                        a.repeat
                    );
                }
            }
            ScheduleCommands::Remove { id } => {
                let mut agents = agent_model::load_agents()?;
                if let Some(a) = agents.iter_mut().find(|a| a.id == *id) {
                    a.schedule = None;
                    a.repeat = false;
                    agent_model::save_agents(&agents)?;
                    println!("Schedule removed for agent {id}.");
                } else {
                    println!("Agent {id} not found.");
                }
            }
        },
    }
    Ok(())
}
