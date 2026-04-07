# Terminal Multi-Agent 🤖

Terminal interactif en Rust pour gérer des agents IA, chatter via une interface TUI moderne (Chat + Éditeur de code), et envoyer des messages via Discord/Telegram.

## Prérequis

- Rust 1.80+ : https://rustup.rs
- OpenSSL (Linux) : `sudo apt install libssl-dev pkg-config`

## Installation & Lancement

```bash
git clone https://github.com/Richerrail/Multi_Agent-Terminal
cd multi_agent_terminal
cargo run
```

Ou en mode release (recommandé) :
```bash
cargo build --release
./target/release/multi_agent_terminal
```

---

## Interface principale

Au lancement, tu arrives sur le **menu TUI** avec navigation clavier :

| Touche | Action |
|--------|--------|
| `↑` / `↓` ou `k` / `j` | Naviguer dans le menu |
| `↵ Entrée` | Sélectionner une option |
| `q` | Quitter |

---

## Options du menu

| Option | Description |
|--------|-------------|
| `+` Ajouter un agent | Enregistre un agent IA (nom, modèle, clé API) |
| `≡` Lister les agents | Affiche tous les agents configurés |
| `✕` Supprimer un agent | Retire un agent de la liste |
| `⚙` Configurer un gateway | Configure Discord ou Telegram |
| `◎` Ouvrir Workspace | Interface Chat + Éditeur de code côte à côte |
| `↑` Envoyer (Discord/TG) | Envoie un message sur Discord ou Telegram |
| `▤` Voir les logs | Affiche l'historique des actions |
| `⏻` Quitter | Ferme l'application |

---

## Workspace (Chat + Éditeur de code)

Le workspace est l'interface principale de conversation. Il se divise en deux panneaux :

- **Gauche (45%)** : Chat avec l'agent IA + zone de saisie
- **Droite (55%)** : Éditeur de code avec numéros de ligne

### Raccourcis dans le workspace

| Touche | Action |
|--------|--------|
| `Tab` | Switcher le focus entre Chat et Éditeur |
| `↵ Entrée` (focus Chat) | Envoyer le message à l'IA |
| `↵ Entrée` (focus Éditeur) | Nouvelle ligne dans l'éditeur |
| `↑` `↓` `←` `→` | Déplacer le curseur (Éditeur) |
| `Ctrl+S` | Sauvegarder le code dans un fichier |
| `Ctrl+R` | Exécuter le code *(à venir)* |
| `Ctrl+C` | Retourner au menu principal |

### Extraction automatique du code

Quand l'IA répond avec des blocs de code (` ```lang ... ``` `), ils sont **automatiquement extraits et insérés** dans l'éditeur de droite avec leur langage en en-tête.

---

## Providers IA supportés

| Préfixe / Modèle | Provider | URL |
|------------------|----------|-----|
| `claude-*` | Anthropic | https://console.anthropic.com |
| `gpt-*`, `o1-*` | OpenAI | https://platform.openai.com |
| `gemini-*` | Google Gemini | https://aistudio.google.com |
| `grok-*` | xAI | https://console.x.ai |
| `deepseek-*` | DeepSeek | https://platform.deepseek.com |
| `mistral-*`, `mixtral-*`, `codestral-*` | Mistral AI | https://console.mistral.ai |
| `llama-*`, `gemma-*`, `compound-*` | Groq | https://console.groq.com |
| `moonshot-*`, `kimi-*` | Moonshot | https://platform.moonshot.cn |
| `command-*`, `aya-*` | Cohere | https://dashboard.cohere.com |
| `together:*` | Together AI | https://api.together.xyz |
| `ollama:*` | Ollama (local) | http://localhost:11434 |
| `glm-*` | ZAI | https://api.z.ai |
| Tout modèle avec `/` | OpenRouter | https://openrouter.ai |
| Autre | OpenAI-compatible | https://platform.openai.com |

### Exemples de modèles

```
claude-sonnet-4-5
gpt-4o
gemini-2.0-flash
llama-3.3-70b-versatile
mistral-large-latest
deepseek-chat
ollama:llama3
together:meta-llama/Llama-3-70b-chat-hf
```

---

## Configuration des Gateways

### Discord

1. Crée un bot sur https://discord.com/developers/applications
2. Active **"Message Content Intent"** dans Bot → Privileged Gateway Intents
3. Copie le **token** du bot
4. Récupère l'**ID du canal** (clic droit sur le canal → Copier l'identifiant)
5. Dans le menu → `⚙ Configurer un gateway` → Discord

### Telegram

1. Parle à `@BotFather` sur Telegram : `/newbot`
2. Copie le **token** obtenu
3. Récupère ton **Chat ID** via `@userinfobot`
4. Dans le menu → `⚙ Configurer un gateway` → Telegram

---

## Historique des conversations

Les conversations sont sauvegardées dans le dossier `history/` au format JSONL, organisées par agent. Chaque session est identifiée par un ID unique.

---

## Fichiers de données

| Fichier | Contenu |
|---------|---------|
| `agents.json` | Agents enregistrés (nom, modèle, clé API) |
| `gateways.json` | Tokens Discord / Telegram |
| `logs.txt` | Historique des actions |
| `history/<agent>.jsonl` | Historique des conversations par agent |

> ⚠️ **Ne commite pas ces fichiers** — ils contiennent tes clés API !

Ajoute à ton `.gitignore` :
```
agents.json
gateways.json
logs.txt
history/
*.txt
```

---

## Structure du projet

```
multi_agent_terminal/
├── Cargo.toml
├── Cargo.lock
├── README.md
└── src/
    ├── main.rs        # Menu TUI principal, loop événements
    ├── workspace.rs   # Workspace Chat + Éditeur (état + rendu)
    ├── agents.rs      # Ajout / liste / suppression d'agents
    ├── gateways.rs    # Discord / Telegram
    ├── chat.rs        # Chat terminal classique avec rendu markdown
    ├── history.rs     # Persistance JSONL des conversations
    ├── theme.rs       # Détection provider, couleurs, logos ASCII
    └── utils.rs       # Persistance agents, logs
```

---

## Dépendances principales

| Crate | Rôle |
|-------|------|
| `ratatui` | Interface TUI (Terminal UI) |
| `crossterm` | Événements clavier, contrôle terminal |
| `tokio` | Runtime async |
| `reqwest` | Appels HTTP vers les APIs IA |
| `serde` / `serde_json` | Sérialisation JSON |
| `chrono` | Horodatage des messages |
| `dialoguer` | Prompts interactifs (hors TUI) |

