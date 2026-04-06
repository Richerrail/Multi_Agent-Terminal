use crate::utils::{self, Agent};
use dialoguer::{Input, Select};

pub async fn add_agent() {
    let name: String = Input::new()
        .with_prompt("Nom de l'agent (ex: groq)")
        .interact_text()
        .unwrap();

    let model: String = Input::new()
        .with_prompt("Modèle (ex: llama-3.3-70b-versatile)")
        .interact_text()
        .unwrap();

    let api_key: String = Input::new()
        .with_prompt("Clé API")
        .interact_text()
        .unwrap();

    let agent = Agent { name: name.clone(), model, api_key };
    let mut agents = utils::load_agents().unwrap_or_else(Vec::new);
    agents.push(agent);
    utils::save_agents(&agents).expect("Échec de la sauvegarde");
    utils::log(&format!("Agent '{}' ajouté.", name));
    println!("\n✅ Agent '{}' ajouté avec succès.\n", name);
}

pub fn list_agents() {
    match utils::load_agents() {
        Some(agents) if !agents.is_empty() => {
            println!("\n📋 Liste des agents :");
            for (i, agent) in agents.iter().enumerate() {
                println!("  {}. {} (modèle: {})", i + 1, agent.name, agent.model);
            }
            println!();
        }
        _ => println!("\n⚠️  Aucun agent enregistré.\n"),
    }
}

pub async fn remove_agent() {
    let mut agents = match utils::load_agents() {
        Some(a) if !a.is_empty() => a,
        _ => { println!("\n⚠️  Aucun agent à supprimer.\n"); return; }
    };

    let names: Vec<String> = agents.iter().map(|a| a.name.clone()).collect();
    let selection = Select::new()
        .with_prompt("Choisir l'agent à supprimer")
        .items(&names)
        .default(0)
        .interact()
        .unwrap();

    let removed = agents.remove(selection);
    utils::save_agents(&agents).expect("Échec de la sauvegarde");
    utils::log(&format!("Agent '{}' supprimé.", removed.name));
    println!("\n🗑️  Agent '{}' supprimé.\n", removed.name);
}
