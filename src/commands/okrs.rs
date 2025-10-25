use crate::cli::OkrCommands;
use crate::store;

pub fn handle(action: &OkrCommands) -> anyhow::Result<()> {
    match action {
        OkrCommands::Add {
            objective,
            key_results,
        } => {
            let mut okrs = store::load_okrs()?;
            let new_okr = store::Okr {
                objective: objective.clone(),
                key_results: key_results
                    .iter()
                    .map(|kr| store::KeyResult {
                        name: kr.clone(),
                        progress: 0.0,
                    })
                    .collect(),
            };
            okrs.push(new_okr);
            store::save_okrs(&okrs)?;
            println!("OKR added successfully.");
        }
        OkrCommands::List => {
            let okrs = store::load_okrs()?;
            println!("{}", serde_json::to_string_pretty(&okrs)?);
        }
    }
    Ok(())
}
