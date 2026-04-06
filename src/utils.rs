use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agent {
    pub name: String,
    pub model: String,
    pub api_key: String,
}

pub fn load_agents() -> Option<Vec<Agent>> {
    if Path::new("agents.json").exists() {
        let data = fs::read_to_string("agents.json").ok()?;
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

pub fn save_agents(agents: &Vec<Agent>) -> std::io::Result<()> {
    let data = serde_json::to_string_pretty(agents)?;
    fs::write("agents.json", data)
}

pub fn show_logs() {
    if Path::new("logs.txt").exists() {
        match fs::read_to_string("logs.txt") {
            Ok(content) => println!("{}", content),
            Err(_) => println!("Erreur lors de la lecture des logs."),
        }
    } else {
        println!("Aucun log disponible.");
    }
}

pub fn log(message: &str) {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs.txt")
        .unwrap();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    writeln!(file, "[{}] {}", timestamp, message).unwrap();
}
