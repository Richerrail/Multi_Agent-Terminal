use crate::theme;
use crate::utils;
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor, SetAttribute, Attribute},
    execute,
};
use std::io::BufRead;
use reqwest::Client;
use std::io::{self, Write};
use std::process::Command;

fn to_rgb(c: ratatui::style::Color) -> (u8, u8, u8) {
    match c {
        ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}

fn print_colored(text: &str, r: u8, g: u8, b: u8) {
    execute!(
        io::stdout(),
        SetForegroundColor(Color::Rgb { r, g, b }),
        Print(text),
        ResetColor,
    )
    .unwrap();
}

fn print_bold(text: &str, r: u8, g: u8, b: u8) {
    execute!(
        io::stdout(),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Rgb { r, g, b }),
        Print(text),
        ResetColor,
        SetAttribute(Attribute::Reset),
    )
    .unwrap();
}

/// Rendu markdown minimaliste pour le terminal
/// Gère: **bold**, `code inline`, ```blocs```, # titres, - listes
fn print_markdown(text: &str, ar: u8, ag: u8, ab: u8) {
    let mut in_code_block = false;
    let mut code_lang = String::new();

    for line in text.lines() {
        let trimmed = line.trim_end();

        // ── Blocs de code ─────────────────────────────────────────────
        if trimmed.starts_with("```") {
            if in_code_block {
                // Fermeture du bloc
                print_colored("  └─────────────────────────────\n", 55, 60, 75);
                in_code_block = false;
                code_lang.clear();
            } else {
                // Ouverture du bloc
                code_lang = trimmed.trim_start_matches('`').to_string();
                let lang_label = if code_lang.is_empty() { String::new() } else { format!(" {} ", code_lang) };
                print_colored(&format!("  ┌─{:─<30}\n", lang_label), 55, 60, 75);
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            print_colored("  │ ", 55, 60, 75);
            // Code en jaune/blanc selon le contenu
            let (cr, cg, cb) = syntax_color(trimmed, &code_lang);
            print_colored(&format!("{}\n", trimmed), cr, cg, cb);
            continue;
        }

        // ── Titres ────────────────────────────────────────────────────
        if trimmed.starts_with("### ") {
            print!("  ");
            print_bold(&format!("▸ {}\n", &trimmed[4..]), 0, 212, 255);
            continue;
        }
        if trimmed.starts_with("## ") {
            print!("  ");
            print_bold(&format!("◆ {}\n", &trimmed[3..]), 0, 212, 255);
            continue;
        }
        if trimmed.starts_with("# ") {
            print!("  ");
            print_bold(&format!("━━ {} ━━\n", &trimmed[2..]), 0, 212, 255);
            continue;
        }

        // ── Séparateur ────────────────────────────────────────────────
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            print_colored(&format!("  {}\n", "─".repeat(40)), 55, 60, 75);
            continue;
        }

        // ── Listes ────────────────────────────────────────────────────
        let (line_content, is_list) = if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            (format!("  • {}", &trimmed[2..]), true)
        } else if trimmed.len() > 2 && trimmed.chars().next().unwrap_or(' ').is_ascii_digit() && trimmed.contains(". ") {
            let rest = trimmed.splitn(2, ". ").nth(1).unwrap_or(trimmed);
            let num = trimmed.splitn(2, ". ").next().unwrap_or("");
            (format!("  {}. {}", num, rest), true)
        } else {
            (format!("  {}", trimmed), false)
        };

        // ── Inline: **bold** et `code` ────────────────────────────────
        print_inline_markdown(&line_content, ar, ag, ab, is_list);
        println!();
    }
}

/// Colorise le code selon le langage (basique)
fn syntax_color(line: &str, lang: &str) -> (u8, u8, u8) {
    let l = lang.to_lowercase();
    let t = line.trim();

    // Commentaires
    if t.starts_with("//") || t.starts_with("#") || t.starts_with("--") {
        return (100, 110, 130);
    }
    // Strings
    if (t.contains('"') || t.contains('\'')) && !t.starts_with("fn ") && !t.starts_with("let ") {
        if l.contains("rust") || l.contains("python") || l.contains("js") || l.contains("py") || l.is_empty() {
            return (255, 200, 100);
        }
    }
    // Mots-clés communs
    let keywords = ["fn ", "let ", "mut ", "pub ", "use ", "mod ", "impl ", "struct ",
                     "def ", "class ", "import ", "from ", "return ", "if ", "else ",
                     "for ", "while ", "const ", "var ", "function ", "async ", "await "];
    for kw in &keywords {
        if t.starts_with(kw) {
            return (191, 100, 255); // violet pour mots-clés
        }
    }
    // Défaut: blanc cassé
    (200, 210, 220)
}

/// Rendu inline: **bold**, *italic*, `code`
fn print_inline_markdown(text: &str, ar: u8, ag: u8, ab: u8, is_list: bool) {
    let base_r = if is_list { (ar / 2).saturating_add(128) } else { ar };
    let base_g = if is_list { (ag / 2).saturating_add(128) } else { ag };
    let base_b = if is_list { (ab / 2).saturating_add(128) } else { ab };

    let mut chars = text.chars().peekable();
    let mut buf = String::new();
    let mut i = 0;
    let bytes = text.as_bytes();

    while i < bytes.len() {
        // **bold**
        if bytes.get(i) == Some(&b'*') && bytes.get(i + 1) == Some(&b'*') {
            if !buf.is_empty() {
                print_colored(&buf, base_r, base_g, base_b);
                buf.clear();
            }
            i += 2;
            let mut bold_buf = String::new();
            while i < bytes.len() {
                if bytes.get(i) == Some(&b'*') && bytes.get(i + 1) == Some(&b'*') {
                    i += 2;
                    break;
                }
                bold_buf.push(bytes[i] as char);
                i += 1;
            }
            print_bold(&bold_buf, 255, 255, 255);
        }
        // `code inline`
        else if bytes.get(i) == Some(&b'`') {
            if !buf.is_empty() {
                print_colored(&buf, base_r, base_g, base_b);
                buf.clear();
            }
            i += 1;
            let mut code_buf = String::new();
            while i < bytes.len() && bytes[i] != b'`' {
                code_buf.push(bytes[i] as char);
                i += 1;
            }
            if bytes.get(i) == Some(&b'`') { i += 1; }
            // Code inline en jaune
            print_colored(&code_buf, 255, 200, 80);
        }
        // *italic* (single star)
        else if bytes.get(i) == Some(&b'*') && bytes.get(i + 1) != Some(&b'*') {
            if !buf.is_empty() {
                print_colored(&buf, base_r, base_g, base_b);
                buf.clear();
            }
            i += 1;
            let mut it_buf = String::new();
            while i < bytes.len() && bytes[i] != b'*' {
                it_buf.push(bytes[i] as char);
                i += 1;
            }
            if bytes.get(i) == Some(&b'*') { i += 1; }
            print_colored(&it_buf, 180, 200, 230); // italic en bleu pâle
        }
        else {
            buf.push(bytes[i] as char);
            i += 1;
        }
    }
    // Vide le buffer restant
    let _ = chars; // suppress warning
    if !buf.is_empty() {
        print_colored(&buf, base_r, base_g, base_b);
    }
}

/// Spinner néon pendant l'envoi
fn neon_spinner() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let running_clone = running.clone();
    std::thread::spawn(move || {
        let frames = ["◉", "◎", "●", "◎", "◉", "○"];
        let mut i = 0usize;
        while running_clone.load(std::sync::atomic::Ordering::Relaxed) {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Rgb { r: 0, g: 212, b: 255 }),
                Print(format!("\r  {} envoi...", frames[i % frames.len()])),
                ResetColor,
            ).unwrap();
            io::stdout().flush().unwrap();
            i += 1;
            std::thread::sleep(std::time::Duration::from_millis(120));
        }
        execute!(io::stdout(), Print("\r                    \r")).unwrap();
        io::stdout().flush().unwrap();
    });
    running
}

/// Exécute une commande shell et retourne (stdout, stderr, code)
fn run_command(cmd: &str) -> (String, String, i32) {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .output()
    } else {
        Command::new("sh")
            .args(["-c", cmd])
            .output()
    };

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let code = o.status.code().unwrap_or(-1);
            (stdout, stderr, code)
        }
        Err(e) => (String::new(), e.to_string(), -1),
    }
}

/// Affiche le résultat d'une commande dans le terminal
fn print_terminal_output(cmd: &str, stdout: &str, stderr: &str, code: i32) {
    // Ligne de commande
    print_colored("  ┌─ $ ", 255, 200, 50);
    print_colored(cmd, 255, 255, 255);
    println!();

    // Stdout
    if !stdout.trim().is_empty() {
        for line in stdout.lines().take(50) {
            print_colored("  │ ", 100, 110, 130);
            println!("{}", line);
        }
        // Tronquer si trop long
        let line_count = stdout.lines().count();
        if line_count > 50 {
            print_colored(&format!("  │ ... ({} lignes tronquées)\n", line_count - 50), 100, 110, 130);
        }
    }

    // Stderr
    if !stderr.trim().is_empty() {
        for line in stderr.lines().take(20) {
            print_colored("  │ ", 239, 68, 68);
            print_colored(line, 239, 100, 68);
            println!();
        }
    }

    // Code de retour
    if code == 0 {
        print_colored(&format!("  └─ ✓ exit {}\n", code), 34, 197, 94);
    } else {
        print_colored(&format!("  └─ ✕ exit {}\n", code), 239, 68, 68);
    }
    println!();
}

/// Demande confirmation avant d'exécuter une commande suggérée par l'IA
fn confirm_and_run(cmd: &str) -> Option<String> {
    print_colored("  ⚡ L'IA suggère: ", 255, 200, 50);
    print_colored(cmd, 255, 255, 255);
    println!();
    print_colored("  Exécuter? [o/N] ", 0, 212, 255);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() == "o" || input.trim().to_lowercase() == "oui" {
        let (stdout, stderr, code) = run_command(cmd);
        print_terminal_output(cmd, &stdout, &stderr, code);
        let combined = format!(
            "$ {}\n{}{}exit: {}",
            cmd,
            if stdout.is_empty() { String::new() } else { stdout.clone() },
            if stderr.is_empty() { String::new() } else { format!("[stderr] {}", stderr) },
            code
        );
        Some(combined)
    } else {
        print_colored("  ↩ Commande annulée.\n\n", 100, 110, 130);
        None
    }
}

/// Extrait les commandes shell suggérées par l'IA dans des blocs ```bash ... ```
fn extract_suggested_commands(reply: &str) -> Vec<String> {
    let mut cmds = Vec::new();
    let mut in_block = false;
    let mut current = Vec::new();

    for line in reply.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```bash") || trimmed.starts_with("```sh") || trimmed == "```shell" {
            in_block = true;
            current.clear();
        } else if trimmed == "```" && in_block {
            in_block = false;
            let cmd = current.join("\n").trim().to_string();
            if !cmd.is_empty() {
                cmds.push(cmd);
            }
            current.clear();
        } else if in_block {
            // Ignore les lignes de commentaire et les lignes vides
            if !trimmed.starts_with('#') && !trimmed.is_empty() {
                current.push(trimmed.to_string());
            }
        }
    }
    cmds
}

/// Lit un message multi-ligne via stdin normal (paste fonctionne!)
/// Ligne "///" seule = envoyer
/// Ligne vide × 2   = envoyer  
/// Ctrl+C            = quitter
fn read_multiline_input() -> String {
    let stdin = io::stdin();
    let mut lines: Vec<String> = Vec::new();
    let mut last_was_empty = false;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim() == "///" {
            break;
        }

        let is_empty = line.trim().is_empty();
        if is_empty && last_was_empty {
            break;
        }
        last_was_empty = is_empty;
        lines.push(line);
    }

    while lines.last().map(|l: &String| l.trim().is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines.join("\n")
}

fn print_logo(model: &str) {
    let (lines, logo_color) = theme::provider_logo(model);
    let (r, g, b) = to_rgb(logo_color);
    println!();
    for line in &lines {
        print!("  ");
        print_colored(line, r, g, b);
        println!();
    }
    println!();
}

fn print_chat_header(agent_name: &str, model: &str) {
    let (ar, ag, ab) = to_rgb(theme::agent_color(agent_name));
    let (_, logo_color) = theme::provider_logo(model);
    let (lr, lg, lb) = to_rgb(logo_color);

    print_logo(model);
    print!("  ");
    print_colored(&format!("[ {} ]", agent_name.to_uppercase()), ar, ag, ab);
    print!("  ");
    print_colored(&format!("({})", model), lr, lg, lb);
    println!();
    print_colored(&format!("  {}\n", "─".repeat(50)), 55, 60, 75);
    println!("  tape 'exit' pour quitter  |  /// ou double Enter pour envoyer  |  paste OK");
    print_colored("  !cmd <commande> pour exécuter  |  L'IA peut suggérer des commandes (auto-détecté)\n\n", 255, 200, 50);
}

pub async fn start_chat() {
    let agents = match utils::load_agents() {
        Some(a) if !a.is_empty() => a,
        _ => {
            println!("  Aucun agent. Ajoute d'abord un agent (option 1).\n");
            return;
        }
    };

    println!();
    print_colored("  Choisir un agent:\n\n", 100, 110, 130);

    for (i, agent) in agents.iter().enumerate() {
        let (lines, logo_color) = theme::provider_logo(&agent.model);
        let (ar, ag, ab) = to_rgb(theme::agent_color(&agent.name));
        let (lr, lg, lb) = to_rgb(logo_color);

        print!("  ");
        print_colored(&format!("[{}] ", i + 1), 100, 110, 130);
        print_colored(lines[1], lr, lg, lb);
        print!("  ");
        print_colored(&agent.name, ar, ag, ab);
        print!("  ");
        print_colored(&format!("({})", agent.model), 100, 110, 130);
        println!();
    }

    println!();
    print_colored("  > ", 0, 212, 255);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let selection: usize = input.trim().parse().unwrap_or(0);

    if selection == 0 || selection > agents.len() {
        println!("  Sélection invalide.\n");
        return;
    }

    let agent = &agents[selection - 1];
    print_chat_header(&agent.name, &agent.model);

    let (ar, ag, ab) = to_rgb(theme::agent_color(&agent.name));

    // Historique de conversation pour le contexte terminal
    let mut terminal_history: Vec<String> = Vec::new();

    loop {
        print_colored("  vous  > ", 0, 212, 255);
        io::stdout().flush().unwrap();

        let message = read_multiline_input();
        let message = message.trim().to_string();

        if message.eq_ignore_ascii_case("exit") {
            println!();
            print_colored("  Au revoir !\n\n", 100, 110, 130);
            break;
        }

        // ─── Commande terminal directe: !cmd <commande> ───────────────────
        if message.starts_with("!cmd ") || message == "!cmd" {
            let cmd = message.trim_start_matches("!cmd").trim();
            if cmd.is_empty() {
                print_colored("  Usage: !cmd <commande shell>\n\n", 100, 110, 130);
                continue;
            }

            println!();
            let (stdout, stderr, code) = run_command(cmd);
            print_terminal_output(cmd, &stdout, &stderr, code);

            // Ajoute au contexte pour l'IA
            let entry = format!("$ {}\n{}{}", cmd,
                if stdout.is_empty() { String::new() } else { stdout.clone() },
                if stderr.is_empty() { String::new() } else { format!("[stderr] {}", stderr) }
            );
            terminal_history.push(entry);

            // Demande si on veut envoyer le résultat à l'IA
            print_colored("  Envoyer le résultat à l'IA? [o/N] ", 0, 212, 255);
            io::stdout().flush().unwrap();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();

            if confirm.trim().to_lowercase() == "o" || confirm.trim().to_lowercase() == "oui" {
                let ctx = build_terminal_context(&terminal_history);
                let prompt = format!("{}\n\nVoici la sortie de la commande:\n```\n$ {}\n{}{}\n```\nAnalyse ce résultat et aide-moi.", ctx, cmd, stdout, stderr);

                let spinner = neon_spinner();
                let result = call_api(&agent.api_key, &agent.model, &prompt).await;
                spinner.store(false, std::sync::atomic::Ordering::Relaxed);
                std::thread::sleep(std::time::Duration::from_millis(150));

                match result {
                    Ok(reply) => {
                println!();
                print_colored(&format!("  {} > \n", agent.name), ar, ag, ab);
                // Rendu markdown colorisé
                print_markdown(&reply, ar, ag, ab);
                println!();

                // Détecte et propose les commandes suggérées
                let suggestions = extract_suggested_commands(&reply);
                for suggested_cmd in &suggestions {
                    if let Some(output) = confirm_and_run(suggested_cmd) {
                        terminal_history.push(output);
                    }
                }

                utils::log(&format!("Terminal [{}] cmd: {} | réponse IA ok", agent.name, cmd));
            }
                    Err(e) => {
                        print_colored(&format!("  ✕ Erreur API: {}\n\n", e), 239, 68, 68);
                    }
                }
            }
            continue;
        }

        // ─── Message normal avec contexte terminal si disponible ──────────
        let full_prompt = if terminal_history.is_empty() {
            // Système: l'IA sait qu'elle a accès au terminal
            format!(
                "Tu es un assistant avec accès à un terminal. \
                L'utilisateur peut exécuter des commandes avec !cmd, \
                et tu peux suggérer des commandes dans des blocs ```bash ... ``` \
                qui seront proposées à l'exécution automatiquement.\n\n{}",
                message
            )
        } else {
            let ctx = build_terminal_context(&terminal_history);
            format!(
                "Tu es un assistant avec accès à un terminal. \
                L'utilisateur peut exécuter des commandes avec !cmd, \
                et tu peux suggérer des commandes dans des blocs ```bash ... ``` \
                qui seront proposées à l'exécution.\n\n\
                Historique terminal:\n{}\n\nUtilisateur: {}",
                ctx, message
            )
        };

        let spinner = neon_spinner();
        let result = call_api(&agent.api_key, &agent.model, &full_prompt).await;
        spinner.store(false, std::sync::atomic::Ordering::Relaxed);
        std::thread::sleep(std::time::Duration::from_millis(150));

        match result {
            Ok(reply) => {
                println!();
                print_colored(&format!("  {} >\n", agent.name), ar, ag, ab);
                print_markdown(&reply, ar, ag, ab);
                println!();

                // Détecte les commandes suggérées par l'IA
                let suggestions = extract_suggested_commands(&reply);
                if !suggestions.is_empty() {
                    print_colored(&format!(
                        "  ⚡ {} commande(s) détectée(s) dans la réponse\n\n",
                        suggestions.len()
                    ), 255, 200, 50);

                    for suggested_cmd in &suggestions {
                        if let Some(output) = confirm_and_run(suggested_cmd) {
                            terminal_history.push(output.clone());

                            // Envoie le résultat à l'IA pour qu'elle continue
                            let followup = format!(
                                "Résultat de la commande exécutée:\n```\n{}\n```\nContinue l'analyse.",
                                output
                            );
                            let spinner2 = neon_spinner();
                            let result2 = call_api(&agent.api_key, &agent.model, &followup).await;
                            spinner2.store(false, std::sync::atomic::Ordering::Relaxed);
                            std::thread::sleep(std::time::Duration::from_millis(150));

                            if let Ok(reply2) = result2 {
                                println!();
                                print_colored(&format!("  {} >\n", agent.name), ar, ag, ab);
                                print_markdown(&reply2, ar, ag, ab);
                                println!();

                                // Récursion 1 niveau: si l'IA suggère encore des commandes
                                let nested = extract_suggested_commands(&reply2);
                                for ncmd in &nested {
                                    if let Some(nout) = confirm_and_run(ncmd) {
                                        terminal_history.push(nout);
                                    }
                                }
                            }
                        }
                    }
                }

                utils::log(&format!(
                    "Chat [{}] > {} | < {}",
                    agent.name, message, reply
                ));
            }
            Err(e) => {
                print_colored(&format!("  ✕ Erreur API: {}\n\n", e), 239, 68, 68);
            }
        }
    }
}

/// Construit un résumé du contexte terminal (max 5 dernières entrées)
fn build_terminal_context(history: &[String]) -> String {
    let start = history.len().saturating_sub(5);
    history[start..].join("\n---\n")
}

fn detect_provider(model: &str) -> &'static str {
    theme::detect_provider(model)
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

async fn call_api(api_key: &str, model: &str, message: &str) -> Result<String, String> {
    let client = Client::new();
    let provider = detect_provider(model);

    if provider == "anthropic" {
        return call_anthropic(&client, api_key, model, message).await;
    }
    if provider == "cohere" {
        return call_cohere(&client, api_key, model, message).await;
    }

    if provider == "zai" {
        let url = "https://api.z.ai/api/paas/v4/chat/completions";
        let body = serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": message}],
            "max_tokens": 4096
        });

        let resp = client.post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {} - {}", status, body));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        return Ok(json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("(pas de réponse)")
            .to_string());
    }

    let base_url = get_base_url(provider);
    let url = format!("{}/chat/completions", base_url);
    let clean_model = if model.starts_with("ollama:") {
        &model[7..]
    } else if model.starts_with("together:") {
        &model[9..]
    } else if model.starts_with("moonshotai/") || model.starts_with("moonshot:") {
        "kimi-k2.5"
    } else if model.contains("kimi-k2.5") {
        "kimi-k2.5"
    } else {
        model
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
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/multi-agent-terminal")
            .header("X-Title", "Multi-Agent Terminal"),
        _ => req.header("Authorization", format!("Bearer {}", api_key)),
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

async fn call_anthropic(client: &Client, api_key: &str, model: &str, message: &str) -> Result<String, String> {
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

async fn call_cohere(client: &Client, api_key: &str, model: &str, message: &str) -> Result<String, String> {
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
