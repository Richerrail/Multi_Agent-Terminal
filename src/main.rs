use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io, time::{Duration, Instant}};

mod agents;
mod chat;
mod gateways;
mod theme;
mod utils;
mod workspace;

use workspace::{Workspace, Focus, MessageRole};

const CYAN:    Color = Color::Rgb(45, 212, 191);
const DARK_BG: Color = Color::Rgb(26, 29, 35);
const PANEL_BG:Color = Color::Rgb(30, 32, 40);
const BORDER:  Color = Color::Rgb(55, 60, 75);
const MUTED:   Color = Color::Rgb(100, 110, 130);
const RED:     Color = Color::Rgb(239, 68, 68);
const AMBER:   Color = Color::Rgb(245, 158, 11);
const GREEN:   Color = Color::Rgb(34, 197, 94);

enum AppState { 
    Menu,
    Workspace(Workspace),
}

struct App {
    menu_state: ListState,
    state: AppState,
    status_msg: Option<(String, Color, Instant)>,
    tick: u64,
}

const MENU_ITEMS: &[(&str, &str)] = &[
    ("+", "Ajouter un agent IA"),
    ("≡", "Lister les agents"),
    ("✕", "Supprimer un agent"),
    ("⚙", "Configurer un gateway"),
    ("◎", "Ouvrir Workspace (Chat+Code)"),
    ("↑", "Envoyer (Discord/TG)"),
    ("▤", "Voir les logs"),
    ("⏻", "Quitter"),
];

impl App {
    fn new() -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        Self { menu_state, state: AppState::Menu, status_msg: None, tick: 0 }
    }
    fn next(&mut self) {
        let i = self.menu_state.selected().unwrap_or(0);
        self.menu_state.select(Some((i + 1) % MENU_ITEMS.len()));
    }
    fn prev(&mut self) {
        let i = self.menu_state.selected().unwrap_or(0);
        self.menu_state.select(Some((i + MENU_ITEMS.len() - 1) % MENU_ITEMS.len()));
    }
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.size();
    f.render_widget(Block::default().style(Style::default().bg(DARK_BG)), area);

    match &app.state {
        AppState::Menu => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(3)])
                .margin(1)
                .split(area);

            render_header(f, chunks[0], app);
            render_body(f, chunks[1], app);
            render_footer(f, chunks[2], app);
        }
        AppState::Workspace(ws) => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
                .margin(0)
                .split(area);

            render_workspace_header(f, ws, chunks[0]);
            workspace::render_workspace(f, ws, chunks[1]);
            workspace::render_workspace_footer(f, ws, chunks[2]);
        }
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let blink = (app.tick / 8) % 2 == 0;
    let cursor = if blink { "█" } else { " " };

    let title_line = Line::from(vec![
        Span::styled("◆ ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("TERMINAL MULTI-AGENT", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" v0.1.0", Style::default().fg(MUTED)),
        Span::styled(cursor, Style::default().fg(CYAN)),
    ]);

    let agents = utils::load_agents().unwrap_or_default();
    let subtitle = Line::from(vec![
        Span::styled("  agents: ", Style::default().fg(MUTED)),
        Span::styled(format!("{}", agents.len()), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("  •  multi-provider  •  Discord/Telegram", Style::default().fg(MUTED)),
    ]);

    let header = Paragraph::new(vec![Line::from(""), title_line, subtitle])
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(CYAN)).style(Style::default().bg(DARK_BG)))
        .alignment(Alignment::Left);

    f.render_widget(header, area);
}

fn render_workspace_header(f: &mut Frame, ws: &Workspace, area: Rect) {
    let (lines, logo_color) = theme::provider_logo(&ws.agent.model);
    
    let title = Line::from(vec![
        Span::styled("◀ ", Style::default().fg(MUTED)),
        Span::styled(format!("{} ", lines[1]), Style::default().fg(logo_color)),
        Span::styled(&ws.agent.name, Style::default().fg(theme::agent_color(&ws.agent.name)).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" ({}) ", ws.agent.model), Style::default().fg(MUTED)),
        Span::styled("▶", Style::default().fg(MUTED)),
        Span::styled("  WORKSPACE  ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
    ]);

    let header = Paragraph::new(title)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(CYAN))
            .style(Style::default().bg(DARK_BG))
        )
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

fn render_body(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Min(0)])
        .split(area);
    render_menu(f, chunks[0], app);
    render_panel(f, chunks[1], app);
}

fn render_menu(f: &mut Frame, area: Rect, app: &App) {
    let selected = app.menu_state.selected().unwrap_or(0);
    let items: Vec<ListItem> = MENU_ITEMS.iter().enumerate().map(|(i, (icon, label))| {
        let is_last = i == MENU_ITEMS.len() - 1;
        let (icon_color, text_color, bg) = if i == selected {
            (CYAN, Color::White, PANEL_BG)
        } else if is_last {
            (RED, MUTED, DARK_BG)
        } else {
            (MUTED, MUTED, DARK_BG)
        };
        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled(if i == selected { "▶ " } else { "  " }, Style::default().fg(CYAN)),
            Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
            Span::styled(*label, Style::default().fg(text_color)),
        ]);
        ListItem::new(line).style(Style::default().bg(bg))
    }).collect();

    let menu = List::new(items).block(
        Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER))
            .title(Span::styled(" Navigation ", Style::default().fg(MUTED).add_modifier(Modifier::ITALIC)))
            .style(Style::default().bg(DARK_BG))
    );
    f.render_stateful_widget(menu, area, &mut app.menu_state.clone());
}

fn render_panel(f: &mut Frame, area: Rect, app: &App) {
    let selected = app.menu_state.selected().unwrap_or(0);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(10)])
        .split(area);
    render_agents_panel(f, chunks[0], selected);
    render_logs_panel(f, chunks[1]);
}

fn render_agents_panel(f: &mut Frame, area: Rect, _selected: usize) {
    let agents = utils::load_agents().unwrap_or_default();
    let items: Vec<ListItem> = if agents.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("Aucun agent — option [+] pour en ajouter", Style::default().fg(MUTED)),
        ]))]
    } else {
        agents.iter().map(|a| {
            let agent_color = theme::agent_color(&a.name);
            let (logo_lines, logo_color) = theme::provider_logo(&a.model);
            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(logo_lines[1], Style::default().fg(logo_color)),
                Span::raw(" "),
                Span::styled(format!("{:<14}", a.name), Style::default().fg(agent_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {:<25}", a.model), Style::default().fg(MUTED)),
                Span::styled("●", Style::default().fg(GREEN)),
            ]);
            ListItem::new(line)
        }).collect()
    };

    let panel = List::new(items).block(
        Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER))
            .title(Span::styled(" Agents actifs ", Style::default().fg(MUTED).add_modifier(Modifier::ITALIC)))
            .style(Style::default().bg(DARK_BG))
    );
    f.render_widget(panel, area);
}

fn render_logs_panel(f: &mut Frame, area: Rect) {
    let log_content = if std::path::Path::new("logs.txt").exists() {
        std::fs::read_to_string("logs.txt").unwrap_or_default()
    } else { String::new() };

    let all_lines: Vec<&str> = log_content.lines().collect();
    let start = all_lines.len().saturating_sub(6);
    let lines: Vec<Line> = all_lines[start..].iter().map(|l: &&str| {
        let l = *l;
        if l.starts_with('[') {
            if let Some(end) = l.find(']') {
                let ts = &l[..=end];
                let msg = &l[end + 2..];
                let (tag, tag_color) = if msg.contains("Chat") { ("INFO", CYAN) }
                    else if msg.contains("Erreur") || msg.contains("erreur") { ("ERR ", RED) }
                    else if msg.contains("Gateway") || msg.contains("configuré") { ("WARN", AMBER) }
                    else { ("INFO", CYAN) };
                return Line::from(vec![
                    Span::styled(format!("{} ", ts), Style::default().fg(MUTED)),
                    Span::styled(tag, Style::default().fg(tag_color).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(msg.to_string(), Style::default().fg(Color::Rgb(180, 185, 200))),
                ]);
            }
        }
        Line::from(Span::styled(l.to_string(), Style::default().fg(MUTED)))
    }).collect();

    let logs = if lines.is_empty() {
        Paragraph::new(Line::from(Span::styled("  Aucun log disponible", Style::default().fg(MUTED))))
    } else {
        Paragraph::new(lines).wrap(Wrap { trim: true })
    };

    f.render_widget(logs.block(
        Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER))
            .title(Span::styled(" Logs récents ", Style::default().fg(MUTED).add_modifier(Modifier::ITALIC)))
            .style(Style::default().bg(DARK_BG))
    ), area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some((msg, color, at)) = &app.status_msg {
        if at.elapsed() < Duration::from_secs(3) {
            Line::from(vec![Span::raw("  "), Span::styled("● ", Style::default().fg(*color)), Span::styled(msg, Style::default().fg(*color))])
        } else { keybind_line() }
    } else { keybind_line() };

    f.render_widget(Paragraph::new(content).block(
        Block::default().borders(Borders::ALL).border_style(Style::default().fg(BORDER)).style(Style::default().bg(DARK_BG))
    ), area);
}

fn keybind_line() -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled("↑↓", Style::default().fg(CYAN)),
        Span::styled(" naviguer  ", Style::default().fg(MUTED)),
        Span::styled("↵", Style::default().fg(CYAN)),
        Span::styled(" sélectionner  ", Style::default().fg(MUTED)),
        Span::styled("q", Style::default().fg(CYAN)),
        Span::styled(" quitter", Style::default().fg(MUTED)),
    ])
}

fn suspend_tui() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn resume_tui() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

use utils::Agent;
use std::io::Write;

fn save_code_to_file(ws: &Workspace) -> Result<(), String> {
    let filename = ws.code_filename.as_ref()
        .cloned()
        .unwrap_or_else(|| "output.txt".to_string());
    
    let mut file = std::fs::File::create(&filename)
        .map_err(|e| e.to_string())?;
    
    file.write_all(ws.code_content.as_bytes())
        .map_err(|e| e.to_string())?;
    
    utils::log(&format!("Code sauvegardé dans {}", filename));
    Ok(())
}

/// Envoie un message à l'agent et retourne la réponse
async fn send_to_agent(agent: Agent, message: String) -> Result<String, String> {
    use reqwest::Client;
    
    let client = Client::new();
    let provider = theme::detect_provider(&agent.model);
    
    if provider == "anthropic" {
        return call_anthropic(&client, &agent.api_key, &agent.model, &message).await;
    }
    if provider == "cohere" {
        return call_cohere(&client, &agent.api_key, &agent.model, &message).await;
    }
    
    let base_url = get_base_url(provider);
    let url = format!("{}/chat/completions", base_url);
    
    let clean_model = if agent.model.starts_with("ollama:") {
        &agent.model[7..]
    } else if agent.model.starts_with("together:") {
        &agent.model[9..]
    } else {
        &agent.model
    };

    let body = serde_json::json!({
        "model": clean_model,
        "messages": [{ "role": "user", "content": message }],
        "max_tokens": 4096
    });

    let mut req = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&body);

    req = match provider {
        "ollama" => req,
        "openrouter" => req
            .header("Authorization", format!("Bearer {}", agent.api_key))
            .header("HTTP-Referer", "https://github.com/multi-agent-terminal")
            .header("X-Title", "Multi-Agent Terminal"),
        _ => req.header("Authorization", format!("Bearer {}", agent.api_key)),
    };

    let resp = req.send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {} - {}", status, body));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("(pas de réponse)")
        .to_string())
}

fn get_base_url(provider: &str) -> &'static str {
    match provider {
        "anthropic"  => "https://api.anthropic.com",
        "openrouter" => "https://openrouter.ai/api/v1",
        "gemini"     => "https://generativelanguage.googleapis.com/v1beta/openai",
        "xai"        => "https://api.x.ai/v1",
        "deepseek"   => "https://api.deepseek.com/v1",
        "mistral"    => "https://api.mistral.ai/v1",
        "groq"       => "https://api.groq.com/openai/v1",
        "moonshot"   => "https://api.moonshot.ai/v1",
        "perplexity" => "https://api.perplexity.ai",
        "cohere"     => "https://api.cohere.ai/v1",
        "together"   => "https://api.together.xyz/v1",
        "fireworks"  => "https://api.fireworks.ai/inference/v1",
        "venice"     => "https://api.venice.ai/api/v1",
        "ollama"     => "http://localhost:11434/v1",
        "zai"        => "https://api.z.ai/api/paas/v4",
        _            => "https://api.openai.com/v1",
    }
}

async fn call_anthropic(client: &reqwest::Client, api_key: &str, model: &str, message: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "messages": [{ "role": "user", "content": message }]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {} - {}", status, body));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(json["content"][0]["text"]
        .as_str()
        .unwrap_or("(pas de réponse)")
        .to_string())
}

async fn call_cohere(client: &reqwest::Client, api_key: &str, model: &str, message: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model,
        "message": message,
        "max_tokens": 4096
    });

    let resp = client
        .post("https://api.cohere.ai/v1/chat")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {} - {}", status, body));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(json["text"]
        .as_str()
        .unwrap_or("(pas de réponse)")
        .to_string())
}

async fn open_workspace() -> Option<Workspace> {
    let agents = match utils::load_agents() {
        Some(a) if !a.is_empty() => a,
        _ => {
            println!("  Aucun agent. Ajoute d'abord un agent (option 1).\n");
            return None;
        }
    };

    println!("\n  Choisir un agent pour le workspace:\n");

    for (i, agent) in agents.iter().enumerate() {
        let (lines, _) = theme::provider_logo(&agent.model);
        println!("  [{}] {} {}", i + 1, lines[1], agent.name);
    }

    println!();
    print!("  > ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let selection: usize = input.trim().parse().unwrap_or(0);

    if selection == 0 || selection > agents.len() {
        println!("  Sélection invalide.\n");
        return None;
    }

    Some(Workspace::new(agents[selection - 1].clone()))
}

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;
        app.tick = app.tick.wrapping_add(1);

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { continue; }

                match &mut app.state {
                    AppState::Menu => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') => break,
                            KeyCode::Up   | KeyCode::Char('k') => app.prev(),
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Enter => {
                                let selected = app.menu_state.selected().unwrap_or(0);
                                if selected == 7 { break; }
                                
                                match selected {
                                    4 => {
                                        // Ouvrir workspace - suspend temporairement pour sélection
                                        suspend_tui()?;
                                        println!();
                                        if let Some(ws) = open_workspace().await {
                                            terminal = resume_tui()?;
                                            app.state = AppState::Workspace(ws);
                                            continue;
                                        }
                                        println!("\n  Appuie sur Entrée pour revenir au menu...");
                                        let mut _input = String::new();
                                        io::stdin().read_line(&mut _input).ok();
                                        terminal = resume_tui()?;
                                    }
                                    _ => {
                                        suspend_tui()?;
                                        println!();
                                        match selected {
                                            0 => agents::add_agent().await,
                                            1 => agents::list_agents(),
                                            2 => agents::remove_agent().await,
                                            3 => gateways::configure_gateway().await,
                                            5 => gateways::send_message().await,
                                            6 => utils::show_logs(),
                                            _ => {}
                                        }
                                        println!("\n  Appuie sur Entrée pour revenir au menu...");
                                        let mut _input = String::new();
                                        io::stdin().read_line(&mut _input).ok();
                                        terminal = resume_tui()?;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    AppState::Workspace(ws) => {
                        // Gestion des touches dans le workspace
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Retour au menu
                                app.state = AppState::Menu;
                            }
                            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Sauvegarder le code
                                if let Err(e) = save_code_to_file(ws) {
                                    ws.add_message(MessageRole::System, format!("Erreur sauvegarde: {}", e));
                                } else {
                                    ws.add_message(MessageRole::System, "💾 Code sauvegardé".to_string());
                                }
                            }
                            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Exécuter le code (si c'est un script)
                                ws.add_message(MessageRole::System, "⚡ Exécution non implémentée".to_string());
                            }
                            KeyCode::Tab => {
                                ws.switch_focus();
                            }
                            KeyCode::Char(c) => {
                                ws.input_char(c);
                            }
                            KeyCode::Backspace => {
                                ws.input_backspace();
                            }
                            KeyCode::Enter => {
                                if ws.focus == Focus::ChatInput {
                                    let msg = ws.get_and_clear_input();
                                    if !msg.is_empty() {
                                        ws.add_message(MessageRole::User, msg.clone());
                                        ws.is_sending = true;
                                        
                                        // Envoi asynchrone à l'API
                                        let agent = ws.agent.clone();
                                        let response = send_to_agent(agent, msg).await;
                                        
                                        match response {
                                            Ok(reply) => {
                                                // Extrait les blocs de code et les insère dans l'éditeur
                                                let extracted = ws.extract_and_insert_code(&reply);
                                                
                                                // Ajoute la réponse au chat
                                                let display_reply = if !extracted.is_empty() {
                                                    format!("{}\n\n[📋 {} bloc(s) inséré(s) dans l'éditeur]", 
                                                        reply, extracted.len())
                                                } else {
                                                    reply
                                                };
                                                ws.add_message(MessageRole::Agent, display_reply);
                                            }
                                            Err(e) => {
                                                ws.add_message(MessageRole::System, format!("Erreur: {}", e));
                                            }
                                        }
                                        ws.is_sending = false;
                                    }
                                } else {
                                    ws.input_enter();
                                }
                            }
                            KeyCode::Up => ws.move_cursor_up(),
                            KeyCode::Down => ws.move_cursor_down(),
                            KeyCode::Left => ws.move_cursor_left(),
                            KeyCode::Right => ws.move_cursor_right(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
