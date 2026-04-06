# Terminal Multi-Agent 🤖

Terminal interactif en Rust pour gérer des agents IA et envoyer des messages via Discord/Telegram.

## Prérequis

- Rust 1.80+ : https://rustup.rs

## Installation & Lancement

```bash
git clone <ton-repo>
cd multi_agent_terminal
cargo run
```

Ou directement :
```bash
cargo build --release
./target/release/multi_agent_terminal
```

## Fonctionnalités

| Option | Description |
|--------|-------------|
| 1. Ajouter un agent | Enregistre un agent IA (nom, modèle, clé API) |
| 2. Lister les agents | Affiche tous les agents configurés |
| 3. Supprimer un agent | Retire un agent de la liste |
| 4. Configurer gateway | Configure Discord ou Telegram (token + channel/chat ID) |
| 5. Chatter | Conversation directe avec un agent via son API |
| 6. Envoyer un message | Envoie un message sur Discord ou Telegram |
| 7. Voir les logs | Affiche l'historique des actions |

## APIs supportées pour le chat

| Modèle | URL API | Clé API |
|--------|---------|---------|
| `gpt-4`, `gpt-3.5-turbo` | OpenAI | https://platform.openai.com |
| `mistral-small`, `mistral-large` | Mistral | https://console.mistral.ai |
| `llama3-*`, `mixtral-*` | Groq | https://console.groq.com |
| Tout modèle OpenAI-compatible | Défaut OpenAI endpoint | - |

## Configuration Discord

1. Crée un bot sur https://discord.com/developers/applications
2. Active "Message Content Intent" dans Bot > Privileged Gateway Intents
3. Copie le token
4. Récupère l'ID du canal (clic droit sur le canal > Copier l'identifiant)

## Configuration Telegram

1. Parle à @BotFather sur Telegram : `/newbot`
2. Copie le token
3. Récupère ton Chat ID via @userinfobot

## Fichiers de données

- `agents.json` — agents enregistrés (créé automatiquement)
- `gateways.json` — tokens Discord/Telegram (créé automatiquement)
- `logs.txt` — historique des actions

⚠️ **Ne commite pas ces fichiers** — ils contiennent tes clés API !

## Structure du projet

```
multi_agent_terminal/
├── Cargo.toml
├── README.md
├── .gitignore
└── src/
    ├── main.rs        # Menu principal
    ├── agents.rs      # Gestion des agents IA
    ├── gateways.rs    # Discord / Telegram
    ├── chat.rs        # Chat avec les agents
    └── utils.rs       # Persistance & logs
```
