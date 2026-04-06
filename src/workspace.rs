use crate::theme;
use crate::utils::Agent;
use chrono::Timelike;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
    Frame,
};

const PANEL_BG: Color = Color::Rgb(30, 32, 40);
const BORDER: Color = Color::Rgb(55, 60, 75);
const CYAN: Color = Color::Rgb(45, 212, 191);
const MUTED: Color = Color::Rgb(100, 110, 130);
const GREEN: Color = Color::Rgb(34, 197, 94);
const RED: Color = Color::Rgb(239, 68, 68);

/// Message dans le chat
#[derive(Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MessageRole {
    User,
    Agent,
    System,
}

#[derive(Clone)]
/// État du workspace (chat + code)
pub struct Workspace {
    pub agent: Agent,
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub focus: Focus,
    pub code_content: String,
    pub code_cursor: (usize, usize), // (line, col)
    pub code_filename: Option<String>,
    pub scroll_chat: u16,
    pub scroll_code: u16,
    pub is_sending: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    ChatInput,
    CodeEditor,
}

impl Workspace {
    pub fn new(agent: Agent) -> Self {
        let mut ws = Self {
            agent,
            messages: Vec::new(),
            input: String::new(),
            focus: Focus::ChatInput,
            code_content: String::new(),
            code_cursor: (0, 0),
            code_filename: None,
            scroll_chat: 0,
            scroll_code: 0,
            is_sending: false,
        };
        ws.add_system_message("Workspace ouvert. Chat à gauche, éditeur à droite. Tab pour switcher.".to_string());
        ws
    }

    pub fn add_message(&mut self, role: MessageRole, content: String) {
        let timestamp = format!("{:02}:{:02}", 
            chrono::Local::now().hour(),
            chrono::Local::now().minute()
        );
        self.messages.push(ChatMessage { role, content, timestamp });
        // Auto-scroll vers le bas
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    pub fn add_system_message(&mut self, content: String) {
        self.add_message(MessageRole::System, content);
    }

    pub fn switch_focus(&mut self) {
        self.focus = match self.focus {
            Focus::ChatInput => Focus::CodeEditor,
            Focus::CodeEditor => Focus::ChatInput,
        };
    }

    // ─── Input handling ─────────────────────────────────────────────

    pub fn input_char(&mut self, c: char) {
        if self.focus == Focus::ChatInput {
            self.input.push(c);
        } else {
            self.insert_at_cursor(c);
        }
    }

    pub fn input_backspace(&mut self) {
        if self.focus == Focus::ChatInput {
            self.input.pop();
        } else {
            self.backspace_at_cursor();
        }
    }

    pub fn input_enter(&mut self) {
        if self.focus == Focus::ChatInput {
            // Le enter est géré par l'appelant pour envoyer
        } else {
            self.insert_at_cursor('\n');
        }
    }

    pub fn get_and_clear_input(&mut self) -> String {
        let msg = self.input.trim().to_string();
        self.input.clear();
        msg
    }

    // ─── Code editor ────────────────────────────────────────────────

    fn insert_at_cursor(&mut self, c: char) {
        let lines: Vec<&str> = self.code_content.lines().collect();
        let mut new_content = String::new();
        
        for (i, line) in lines.iter().enumerate() {
            if i == self.code_cursor.0 {
                let mut new_line = line.to_string();
                if self.code_cursor.1 <= new_line.len() {
                    new_line.insert(self.code_cursor.1, c);
                    self.code_cursor.1 += 1;
                }
                new_content.push_str(&new_line);
            } else {
                new_content.push_str(line);
            }
            if i < lines.len() - 1 || self.code_content.ends_with('\n') {
                new_content.push('\n');
            }
        }
        
        // Si on est sur une nouvelle ligne
        if self.code_cursor.0 >= lines.len() {
            new_content.push(c);
            self.code_cursor.1 += 1;
        }
        
        self.code_content = new_content;
    }

    fn backspace_at_cursor(&mut self) {
        let lines: Vec<&str> = self.code_content.lines().collect();
        if lines.is_empty() { return; }
        
        let mut new_content = String::new();
        let mut cursor_moved = false;
        
        for (i, line) in lines.iter().enumerate() {
            if i == self.code_cursor.0 && self.code_cursor.1 > 0 {
                let mut new_line = line.to_string();
                new_line.remove(self.code_cursor.1 - 1);
                self.code_cursor.1 -= 1;
                new_content.push_str(&new_line);
                cursor_moved = true;
            } else if i == self.code_cursor.0 && self.code_cursor.1 == 0 && i > 0 {
                // Merge avec ligne précédente
                continue;
            } else {
                new_content.push_str(line);
            }
            if i < lines.len() - 1 || self.code_content.ends_with('\n') {
                new_content.push('\n');
            }
        }
        
        if !cursor_moved && self.code_cursor.0 > 0 {
            self.code_cursor.0 -= 1;
            let prev_line_len = lines.get(self.code_cursor.0).map(|l| l.len()).unwrap_or(0);
            self.code_cursor.1 = prev_line_len;
        }
        
        self.code_content = new_content;
    }

    pub fn move_cursor_up(&mut self) {
        if self.focus == Focus::CodeEditor && self.code_cursor.0 > 0 {
            self.code_cursor.0 -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        let lines: Vec<&str> = self.code_content.lines().collect();
        if self.focus == Focus::CodeEditor && self.code_cursor.0 < lines.len().saturating_sub(1) {
            self.code_cursor.0 += 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.focus == Focus::CodeEditor && self.code_cursor.1 > 0 {
            self.code_cursor.1 -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let lines: Vec<&str> = self.code_content.lines().collect();
        let line_len = lines.get(self.code_cursor.0).map(|l| l.len()).unwrap_or(0);
        if self.focus == Focus::CodeEditor && self.code_cursor.1 < line_len {
            self.code_cursor.1 += 1;
        }
    }

    pub fn clear_code(&mut self) {
        self.code_content.clear();
        self.code_cursor = (0, 0);
    }

    pub fn insert_code_block(&mut self, code: &str, lang: &str) {
        self.code_content.push_str(&format!("// --- {} ---\n", lang));
        self.code_content.push_str(code);
        self.code_content.push('\n');
    }

    /// Extrait les blocs de code de la réponse de l'IA et les insère dans l'éditeur
    pub fn extract_and_insert_code(&mut self, reply: &str) -> Vec<String> {
        let mut extracted = Vec::new();
        let mut in_block = false;
        let mut current_lang = String::new();
        let mut current_code = Vec::new();

        for line in reply.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("```") {
                if in_block {
                    // Fin du bloc
                    let code = current_code.join("\n");
                    if !code.trim().is_empty() {
                        self.insert_code_block(&code, &current_lang);
                        extracted.push(format!("{} ({} lignes)", current_lang, current_code.len()));
                    }
                    in_block = false;
                    current_lang.clear();
                    current_code.clear();
                } else {
                    // Début du bloc
                    current_lang = trimmed.trim_start_matches('`').to_string();
                    if current_lang.is_empty() {
                        current_lang = "text".to_string();
                    }
                    in_block = true;
                }
                continue;
            }
            
            if in_block {
                current_code.push(line.to_string());
            }
        }
        
        extracted
    }
}

// ─── Rendering ──────────────────────────────────────────────────────

pub fn render_workspace(f: &mut Frame, ws: &Workspace, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    render_chat_panel(f, ws, chunks[0]);
    render_code_panel(f, ws, chunks[1]);
}

fn render_chat_panel(f: &mut Frame, ws: &Workspace, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    // Zone messages
    let is_focused = ws.focus == Focus::ChatInput;
    let border_style = if is_focused { 
        Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
    } else { 
        Style::default().fg(BORDER) 
    };

    let messages_text = format_messages(&ws.messages, &ws.agent.name);
    let messages_para = Paragraph::new(messages_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(Span::styled(
                    format!(" 💬 {} ", ws.agent.name),
                    Style::default().fg(theme::agent_color(&ws.agent.name)).add_modifier(Modifier::BOLD)
                ))
                .style(Style::default().bg(PANEL_BG))
        )
        .wrap(Wrap { trim: false })
        .scroll((ws.scroll_chat, 0));

    f.render_widget(messages_para, chunks[0]);

    // Zone input
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            if is_focused { " [INPUT] " } else { " input " },
            Style::default().fg(if is_focused { CYAN } else { MUTED })
        ))
        .style(Style::default().bg(PANEL_BG));

    let input_text = if ws.is_sending {
        "⏳ Envoi en cours...".to_string()
    } else {
        format!("{}█", ws.input)
    };

    let input_para = Paragraph::new(input_text)
        .block(input_block)
        .wrap(Wrap { trim: false });

    f.render_widget(input_para, chunks[1]);
}

fn render_code_panel(f: &mut Frame, ws: &Workspace, area: Rect) {
    let is_focused = ws.focus == Focus::CodeEditor;
    let border_style = if is_focused { 
        Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
    } else { 
        Style::default().fg(BORDER) 
    };

    let title = match &ws.code_filename {
        Some(name) => format!(" 📝 {} ", name),
        None => " 📝 Éditeur ".to_string(),
    };

    // Numéros de ligne + contenu
    let lines: Vec<&str> = ws.code_content.lines().collect();
    let mut content_with_lines = String::new();
    
    for (i, line) in lines.iter().enumerate() {
        let line_num = format!("{:3} │ ", i + 1);
        content_with_lines.push_str(&line_num);
        content_with_lines.push_str(line);
        content_with_lines.push('\n');
    }

    // Cursor indicator
    let display_content = if is_focused && !ws.is_sending {
        format!("{}█", content_with_lines.trim_end())
    } else {
        content_with_lines
    };

    let code_para = Paragraph::new(display_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(Span::styled(
                    title,
                    Style::default().fg(if is_focused { GREEN } else { MUTED }).add_modifier(Modifier::BOLD)
                ))
                .style(Style::default().bg(PANEL_BG))
        )
        .wrap(Wrap { trim: false })
        .scroll((ws.scroll_code, 0));

    f.render_widget(code_para, area);
}

fn format_messages<'a>(messages: &'a [ChatMessage], agent_name: &'a str) -> Text<'a> {
    let mut lines: Vec<Line> = Vec::new();

    for msg in messages {
        let (prefix, color) = match msg.role {
            MessageRole::User => (" vous", theme::USER_COLOR),
            MessageRole::Agent => (agent_name, theme::agent_color(agent_name)),
            MessageRole::System => (" sys", MUTED),
        };

        // Header du message
        lines.push(Line::from(vec![
            Span::styled(
                format!("[{}] ", msg.timestamp),
                Style::default().fg(MUTED)
            ),
            Span::styled(
                prefix.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            ),
        ]));

        // Contenu avec word wrap manuel simple
        for content_line in msg.content.lines() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    content_line.to_string(),
                    Style::default().fg(Color::Rgb(200, 210, 220))
                ),
            ]));
        }

        lines.push(Line::from(""));
    }

    Text::from(lines)
}

/// Rendu de la barre de statut du workspace
pub fn render_workspace_footer(f: &mut Frame, ws: &Workspace, area: Rect) {
    let focus_text = match ws.focus {
        Focus::ChatInput => "CHAT",
        Focus::CodeEditor => "CODE",
    };

    let content = Line::from(vec![
        Span::raw("  "),
        Span::styled("Tab", Style::default().fg(CYAN)),
        Span::styled(" switch  ", Style::default().fg(MUTED)),
        Span::styled("↵", Style::default().fg(CYAN)),
        Span::styled(" envoyer  ", Style::default().fg(MUTED)),
        Span::styled("Ctrl+S", Style::default().fg(CYAN)),
        Span::styled(" sauver  ", Style::default().fg(MUTED)),
        Span::styled("Ctrl+C", Style::default().fg(CYAN)),
        Span::styled(" retour  ", Style::default().fg(MUTED)),
        Span::styled("│", Style::default().fg(BORDER)),
        Span::styled(" Focus: ", Style::default().fg(MUTED)),
        Span::styled(
            focus_text,
            Style::default().fg(if ws.focus == Focus::ChatInput { CYAN } else { GREEN })
                .add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            format!(" │ msgs:{} code:{}b", ws.messages.len(), ws.code_content.len()),
            Style::default().fg(MUTED)
        ),
    ]);

    f.render_widget(
        Paragraph::new(content).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .style(Style::default().bg(PANEL_BG))
        ),
        area
    );
}
