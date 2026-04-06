use crate::workspace::{ChatMessage, MessageRole};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, Write};
use std::path::PathBuf;

/// Enregistrement d'un message pour la persistance
#[derive(Serialize, Deserialize, Debug, Clone)]
struct HistoryEntry {
    timestamp: String,
    role: String, // "user", "agent", "system"
    content: String,
    session_id: String,
}

/// Gestionnaire d'historique
pub struct HistoryManager {
    history_dir: PathBuf,
}

impl HistoryManager {
    pub fn new() -> Self {
        let history_dir = PathBuf::from("history");
        if !history_dir.exists() {
            fs::create_dir_all(&history_dir).unwrap_or_default();
        }
        Self { history_dir }
    }

    fn get_history_file(&self, agent_name: &str) -> PathBuf {
        // Nom de fichier sécurisé (remplace les caractères spéciaux)
        let safe_name: String = agent_name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        self.history_dir.join(format!("{}.jsonl", safe_name))
    }

    /// Sauvegarde un message dans l'historique
    pub fn save_message(&self, agent_name: &str, msg: &ChatMessage, session_id: &str) {
        let entry = HistoryEntry {
            timestamp: msg.timestamp.clone(),
            role: match msg.role {
                MessageRole::User => "user",
                MessageRole::Agent => "agent",
                MessageRole::System => "system",
            }.to_string(),
            content: msg.content.clone(),
            session_id: session_id.to_string(),
        };

        let file_path = self.get_history_file(agent_name);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .unwrap();

        if let Ok(json) = serde_json::to_string(&entry) {
            writeln!(file, "{}", json).unwrap_or_default();
        }
    }

    /// Charge l'historique d'un agent (dernières N entrées)
    pub fn load_history(&self, agent_name: &str, limit: usize) -> Vec<ChatMessage> {
        let file_path = self.get_history_file(agent_name);
        
        if !file_path.exists() {
            return Vec::new();
        }

        let file = match fs::File::open(&file_path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = std::io::BufReader::new(file);
        let mut entries: Vec<HistoryEntry> = Vec::new();

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
                    entries.push(entry);
                }
            }
        }

        // Garde seulement les N dernières entrées
        let start = entries.len().saturating_sub(limit);
        let recent_entries = &entries[start..];

        recent_entries
            .iter()
            .map(|e| ChatMessage {
                timestamp: e.timestamp.clone(),
                role: match e.role.as_str() {
                    "user" => MessageRole::User,
                    "agent" => MessageRole::Agent,
                    _ => MessageRole::System,
                },
                content: e.content.clone(),
            })
            .collect()
    }

    /// Liste toutes les sessions disponibles pour un agent
    pub fn list_sessions(&self, agent_name: &str) -> Vec<(String, usize)> {
        let file_path = self.get_history_file(agent_name);
        
        if !file_path.exists() {
            return Vec::new();
        }

        let file = match fs::File::open(&file_path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = std::io::BufReader::new(file);
        let mut sessions: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
                    *sessions.entry(entry.session_id).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<(String, usize)> = sessions.into_iter().collect();
        result.sort_by(|a, b| b.0.cmp(&a.0)); // Plus récent d'abord
        result
    }

    /// Charge une session spécifique
    pub fn load_session(&self, agent_name: &str, session_id: &str) -> Vec<ChatMessage> {
        let file_path = self.get_history_file(agent_name);
        
        if !file_path.exists() {
            return Vec::new();
        }

        let file = match fs::File::open(&file_path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = std::io::BufReader::new(file);
        let mut messages: Vec<ChatMessage> = Vec::new();

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
                    if entry.session_id == session_id {
                        messages.push(ChatMessage {
                            timestamp: entry.timestamp,
                            role: match entry.role.as_str() {
                                "user" => MessageRole::User,
                                "agent" => MessageRole::Agent,
                                _ => MessageRole::System,
                            },
                            content: entry.content,
                        });
                    }
                }
            }
        }

        messages
    }

    /// Supprime l'historique d'un agent
    pub fn clear_history(&self, agent_name: &str) -> Result<(), String> {
        let file_path = self.get_history_file(agent_name);
        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Taille totale de l'historique (en messages)
    pub fn history_size(&self, agent_name: &str) -> usize {
        let file_path = self.get_history_file(agent_name);
        
        if !file_path.exists() {
            return 0;
        }

        match fs::read_to_string(&file_path) {
            Ok(content) => content.lines().count(),
            Err(_) => 0,
        }
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}
