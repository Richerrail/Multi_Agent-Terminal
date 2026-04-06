use crate::utils;
use dialoguer::{Input, Select};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn load_gateways() -> HashMap<String, String> {
    if Path::new("gateways.json").exists() {
        let data = fs::read_to_string("gateways.json").unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_gateways(gateways: &HashMap<String, String>) {
    let data = serde_json::to_string_pretty(gateways).unwrap();
    fs::write("gateways.json", data).expect("Échec sauvegarde gateways");
}

pub async fn configure_gateway() {
    let options = vec!["Discord", "Telegram"];
    let selection = Select::new()
        .with_prompt("Choisir le gateway")
        .items(&options)
        .default(0)
        .interact()
        .unwrap();
    match selection {
        0 => configure_discord().await,
        1 => configure_telegram().await,
        _ => unreachable!(),
    }
}

async fn configure_discord() {
    let token: String = Input::new().with_prompt("Token du bot Discord").interact_text().unwrap();
    let channel_id: String = Input::new().with_prompt("ID du canal Discord").interact_text().unwrap();
    let mut gw = load_gateways();
    gw.insert("discord_token".to_string(), token);
    gw.insert("discord_channel".to_string(), channel_id);
    save_gateways(&gw);
    utils::log("Gateway Discord configuré.");
    println!("\n✅ Gateway Discord configuré.\n");
}

async fn configure_telegram() {
    let token: String = Input::new().with_prompt("Token du bot Telegram").interact_text().unwrap();
    let chat_id: String = Input::new().with_prompt("Chat ID Telegram").interact_text().unwrap();
    let mut gw = load_gateways();
    gw.insert("telegram_token".to_string(), token);
    gw.insert("telegram_chat_id".to_string(), chat_id);
    save_gateways(&gw);
    utils::log("Gateway Telegram configuré.");
    println!("\n✅ Gateway Telegram configuré.\n");
}

pub async fn send_message() {
    let gw = load_gateways();
    let options = vec!["Discord", "Telegram"];
    let selection = Select::new().with_prompt("Envoyer via").items(&options).default(0).interact().unwrap();
    let message: String = Input::new().with_prompt("Message").interact_text().unwrap();
    match selection {
        0 => match (gw.get("discord_token"), gw.get("discord_channel")) {
            (Some(t), Some(c)) => send_discord(t, c, &message).await,
            _ => println!("\n⚠️  Discord non configuré.\n"),
        },
        1 => match (gw.get("telegram_token"), gw.get("telegram_chat_id")) {
            (Some(t), Some(c)) => send_telegram(t, c, &message).await,
            _ => println!("\n⚠️  Telegram non configuré.\n"),
        },
        _ => unreachable!(),
    }
}

async fn send_discord(token: &str, channel_id: &str, message: &str) {
    let client = reqwest::Client::new();
    let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
    match client.post(&url)
        .header("Authorization", format!("Bot {}", token))
        .json(&serde_json::json!({ "content": message }))
        .send().await
    {
        Ok(r) if r.status().is_success() => { utils::log(&format!("Discord: {}", message)); println!("\n✅ Envoyé sur Discord!\n"); }
        Ok(r) => println!("\n❌ Erreur Discord: HTTP {}\n", r.status()),
        Err(e) => println!("\n❌ Erreur réseau: {}\n", e),
    }
}

async fn send_telegram(token: &str, chat_id: &str, message: &str) {
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    match client.post(&url)
        .json(&serde_json::json!({ "chat_id": chat_id, "text": message }))
        .send().await
    {
        Ok(r) if r.status().is_success() => { utils::log(&format!("Telegram: {}", message)); println!("\n✅ Envoyé sur Telegram!\n"); }
        Ok(r) => println!("\n❌ Erreur Telegram: HTTP {}\n", r.status()),
        Err(e) => println!("\n❌ Erreur réseau: {}\n", e),
    }
}
