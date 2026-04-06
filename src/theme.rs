use ratatui::style::Color;

const AGENT_COLORS: &[(u8, u8, u8)] = &[
    (0, 212, 255),   // bleu néon
    (255, 0, 255),   // magenta
    (57, 255, 20),   // vert lime
    (255, 102, 0),   // orange néon
    (255, 0, 128),   // rose hot pink
    (191, 0, 255),   // violet
    (255, 255, 0),   // jaune électrique
    (0, 255, 204),   // turquoise
    (255, 51, 51),   // rouge vif
    (255, 165, 0),   // orange gold
    (0, 255, 127),   // spring green
    (138, 43, 226),  // blue violet
];

pub const USER_COLOR: Color = Color::Rgb(0, 212, 255);

pub fn agent_color(name: &str) -> Color {
    let hash: usize = name.bytes().fold(0usize, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as usize)
    });
    let (r, g, b) = AGENT_COLORS[hash % AGENT_COLORS.len()];
    Color::Rgb(r, g, b)
}

pub fn detect_provider(model: &str) -> &'static str {
    let m = model.to_lowercase();
    if m.starts_with("claude")                                              { return "anthropic"; }
    if m.starts_with("together:")                                           { return "together"; }
    if m.starts_with("groq/") || m.starts_with("groq:")                    { return "groq"; }
    if m.contains('/')                                                      { return "openrouter"; }
    if m.starts_with("gemini") || m.starts_with("palm")                    { return "gemini"; }
    if m.starts_with("grok")                                               { return "xai"; }
    if m.starts_with("deepseek")                                           { return "deepseek"; }
    if m.starts_with("mistral") || m.starts_with("mixtral") || m.starts_with("codestral") { return "mistral"; }
    if m.starts_with("compound") || m.starts_with("llama") || m.starts_with("gemma") || m.starts_with("whisper") { return "groq"; }
    if m.starts_with("moonshot") || m.starts_with("kimi")                  { return "moonshot"; }
    if m.contains("sonar") || m.starts_with("pplx")                        { return "perplexity"; }
    if m.starts_with("command") || m.starts_with("embed") || m.starts_with("tiny-") || m.starts_with("aya") { return "cohere"; }
    if m.contains("together")                                              { return "together"; }
    if m.contains("fireworks") || m.contains("fw/")                        { return "fireworks"; }
    if m.starts_with("ollama:") || m.starts_with("llama3") || m.starts_with("phi") || m.starts_with("qwen") { return "ollama"; }
    if m.starts_with("glm")                                                { return "zai"; }
    "openai"
}

pub fn provider_logo(model: &str) -> ([&'static str; 3], Color) {
    match detect_provider(model) {
        "anthropic"  => (["╭─╮", "│ │", "╰─╯"], Color::Rgb(204, 153, 255)),
        "openai"     => (["┌─┐", "│●│", "└─┘"], Color::Rgb(120, 220, 120)),
        "gemini"     => (["◇ ◇", " ◈ ", "◇ ◇"], Color::Rgb(66, 165, 245)),
        "xai"        => (["╔═╗", "║×║", "╚═╝"], Color::Rgb(255, 255, 255)),
        "deepseek"   => (["≋≋≋", "≋ ≋", "≋≋≋"], Color::Rgb(0, 180, 255)),
        "mistral"    => (["/\\/", "\\/\\", "/\\/"], Color::Rgb(255, 140, 0)),
        "groq"       => (["▗▄▖", "▐█▌", "▝▀▘"], Color::Rgb(255, 80, 80)),
        "moonshot"   => (["▪▪▪", "▪ ▪", "▪▪▪"], Color::Rgb(0, 140, 255)),
        "perplexity" => (["◎─◎", "│ │", "◎─◎"], Color::Rgb(100, 220, 200)),
        "cohere"     => (["●─●", " ╲ ", "●─●"], Color::Rgb(255, 100, 150)),
        "ollama"     => (["∩|∩", "|||", "╙╨╜"], Color::Rgb(150, 255, 100)),
        "openrouter" => (["◁─▷", "─┼─", "◁─▷"], Color::Rgb(255, 200, 50)),
        "together"   => (["○●○", "●○●", "○●○"], Color::Rgb(180, 100, 255)),
        "fireworks"  => (["░▒█", "▒█▒", "█▒░"], Color::Rgb(255, 200, 0)),
        "zai"        => (["▀▄▀", "█Z█", "▄▀▄"], Color::Rgb(0, 255, 180)),
        "venice"     => (["~≈~", "≈V≈", "~≈~"], Color::Rgb(100, 180, 255)),
        _            => (["[  ]", "[ ·]", "[  ]"], Color::Rgb(150, 150, 150)),
    }
}
