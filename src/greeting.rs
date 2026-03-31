use crate::companion::{render, Companion};

const TAGLINE: &str = "you weren't using your terminal anyways";

// ANSI escape codes
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

// Block letter "CLAWDSHELL" — 5 lines tall to match companion sprite height
const LOGO: [&str; 5] = [
    " ██████╗██╗      █████╗ ██╗    ██╗██████╗ ███████╗██╗  ██╗███████╗██╗     ██╗     ",
    "██╔════╝██║     ██╔══██╗██║    ██║██╔══██╗██╔════╝██║  ██║██╔════╝██║     ██║     ",
    "██║     ██║     ███████║██║ █╗ ██║██║  ██║███████╗███████║█████╗  ██║     ██║     ",
    "██║     ██║     ██╔══██║██║███╗██║██║  ██║╚════██║██╔══██║██╔══╝  ██║     ██║     ",
    "╚██████╗███████╗██║  ██║╚███╔███╔╝██████╔╝███████║██║  ██║███████╗███████╗███████╗",
];

pub fn render_greeting(
    tool_name: &str,
    fallback_shell: &str,
    companion: &Companion,
    terminal_width: u16,
) -> String {
    let mut out = String::new();
    let version = env!("CARGO_PKG_VERSION");

    if terminal_width >= 100 {
        // Wide mode: companion sprite + block letter logo
        let sprite_lines = render::render_sprite(companion, 0);

        out.push('\n');
        // Render sprite and logo side by side
        let max_lines = sprite_lines.len().max(LOGO.len());
        for i in 0..max_lines {
            let sprite = if i < sprite_lines.len() { &sprite_lines[i] } else { "            " };
            let logo = if i < LOGO.len() { LOGO[i] } else { "" };
            out.push_str(&format!("{}  {}{}{}\n", sprite, BOLD, logo, RESET));
        }
        if !tool_name.is_empty() {
            out.push_str(&format!(
                "{}              v{} · launching {} · {}{}\n",
                DIM, version, tool_name, TAGLINE, RESET
            ));
            out.push_str(&format!(
                "{}              Ctrl+D to drop to {}{}\n\n",
                DIM, fallback_shell, RESET
            ));
        } else {
            out.push_str(&format!(
                "{}              v{} · {}{}\n\n",
                DIM, version, TAGLINE, RESET
            ));
        }
    } else if terminal_width >= 60 {
        // Medium mode: just logo, no sprite
        out.push('\n');
        let face = render::render_face(companion);
        out.push_str(&format!(
            "{} {}CLAWDSHELL{} v{}\n",
            face, BOLD, RESET, version
        ));
        out.push_str(&format!(
            "  {}launching {} · {}{}\n",
            DIM, tool_name, TAGLINE, RESET
        ));
        out.push_str(&format!(
            "  {}Ctrl+D to drop to {}{}\n\n",
            DIM, fallback_shell, RESET
        ));
    } else {
        // Narrow mode: compact
        let face = render::render_face(companion);
        out.push_str(&format!(
            "\n{} {}CLAWDSHELL{} launching {}\n",
            face, BOLD, RESET, tool_name
        ));
        out.push_str(&format!(
            "{}Ctrl+D for {}{}\n",
            DIM, fallback_shell, RESET
        ));
    }

    out
}
