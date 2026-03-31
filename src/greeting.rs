use crate::companion::{render, Companion};

const TAGLINE: &str = "you weren't using your terminal anyways";

// ANSI escape codes
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub fn render_greeting(
    tool_name: &str,
    fallback_shell: &str,
    companion: &Companion,
    terminal_width: u16,
) -> String {
    let mut out = String::new();
    let version = env!("CARGO_PKG_VERSION");
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    if terminal_width >= 60 {
        // Wide mode: sprite on left, info on right (like Claude Code)
        let sprite_lines = render::render_sprite(companion, 0);

        let info_lines = [
            format!("{}CLAWDSHELL{} v{}", BOLD, RESET, version),
            format!("launching {} {}{}{}", tool_name, DIM, TAGLINE, RESET),
            cwd,
        ];

        out.push('\n');
        for (i, sprite_line) in sprite_lines.iter().enumerate() {
            let right = if i < info_lines.len() { &info_lines[i] } else { "" };
            out.push_str(&format!("{}    {}\n", sprite_line, right));
        }
        out.push('\n');
        out.push_str(&format!(
            "{}Ctrl+D to drop to {}{}\n",
            DIM, fallback_shell, RESET
        ));
    } else {
        // Narrow mode: compact
        let face = render::render_face(companion);
        out.push_str(&format!(
            "\n{}CLAWDSHELL{} {} launching {}\n",
            BOLD, RESET, face, tool_name
        ));
        out.push_str(&format!(
            "{}Ctrl+D for {}{}\n",
            DIM, fallback_shell, RESET
        ));
    }

    out
}
