use crate::companion::{render, Companion};

const TAGLINE: &str = "you weren't using your terminal anyways";

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

// ASCII art logo — 6 lines tall, standard figlet style
const LOGO: [&str; 6] = [
    r"  _____   _           _    _ ____    _____ _    _ ______ _      _      ",
    r" / ____| | |         | |  | |  _ \  / ____| |  | |  ____| |    | |     ",
    r"| |      | |     __ _| |  | | | | || (___ | |__| | |__  | |    | |     ",
    r"| |      | |    / _` | |/\| | | | | \___ \|  __  |  __| | |    | |     ",
    r"| |____  | |___| (_| \  /\  / |_| | ____) | |  | | |____| |____| |____ ",
    r" \_____| |______\__,_|\/  \/|____/ |_____/|_|  |_|______|______|______|",
];

/// Render the startup greeting.
/// `tool_name` and `fallback_shell` can be empty (for install banner).
pub fn render_greeting(
    tool_name: &str,
    fallback_shell: &str,
    companion: &Companion,
    terminal_width: u16,
) -> String {
    let mut out = String::new();
    let version = env!("CARGO_PKG_VERSION");

    if terminal_width >= 80 {
        // Wide: companion on left, logo on right
        let sprite_lines = render::render_sprite(companion, 0);

        out.push('\n');

        // Merge sprite and logo side by side
        let max_lines = sprite_lines.len().max(LOGO.len());
        for i in 0..max_lines {
            let sprite = if i < sprite_lines.len() {
                &sprite_lines[i]
            } else {
                "            "
            };
            let logo = if i < LOGO.len() {
                format!("{}{}{}", BOLD, LOGO[i], RESET)
            } else {
                String::new()
            };
            out.push_str(&format!(" {}  {}\n", sprite, logo));
        }

        // Subtitle
        if !tool_name.is_empty() {
            out.push_str(&format!(
                " {}v{} · launching {} · {}{}\n",
                DIM, version, tool_name, TAGLINE, RESET
            ));
            out.push_str(&format!(
                " {}Ctrl+D to drop to {}{}\n",
                DIM, fallback_shell, RESET
            ));
        } else {
            out.push_str(&format!(
                " {}v{} · {}{}\n",
                DIM, version, TAGLINE, RESET
            ));
        }
        out.push('\n');
    } else {
        // Narrow: face + text
        let face = render::render_face(companion);
        out.push('\n');
        out.push_str(&format!(
            " {} {}CLAWDSHELL{} v{}\n",
            face, BOLD, RESET, version
        ));
        if !tool_name.is_empty() {
            out.push_str(&format!(
                " {}launching {} · {}{}\n",
                DIM, tool_name, TAGLINE, RESET
            ));
            out.push_str(&format!(
                " {}Ctrl+D for {}{}\n",
                DIM, fallback_shell, RESET
            ));
        } else {
            out.push_str(&format!(" {}{}{}\n", DIM, TAGLINE, RESET));
        }
        out.push('\n');
    }

    out
}
