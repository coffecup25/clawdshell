use crate::companion::{render, Companion};

const TAGLINE: &str = "you weren't using your terminal anyways";

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

// Bubble-charm style title: clean spaced letters in a box
const TITLE_BOX: [&str; 3] = [
    "╔══════════════════════════════════════════╗",
    "║   C  L  A  W  D  S  H  E  L  L          ║",
    "╚══════════════════════════════════════════╝",
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

    if terminal_width >= 70 {
        let sprite_lines = render::render_sprite(companion, 0);

        out.push('\n');

        // Companion sprite + title box side by side
        // Title box is 3 lines, sprite is 4-5 lines. Align vertically.
        let total_lines = sprite_lines.len().max(TITLE_BOX.len() + 1);
        for i in 0..total_lines {
            let sprite = if i < sprite_lines.len() {
                &sprite_lines[i]
            } else {
                "            "
            };
            let right = if i < TITLE_BOX.len() {
                format!("{}  {}{}", BOLD, TITLE_BOX[i], RESET)
            } else if i == TITLE_BOX.len() {
                // Subtitle line right after the box
                if !tool_name.is_empty() {
                    format!(
                        "{}  v{} · launching {} · {}{}",
                        DIM, version, tool_name, TAGLINE, RESET
                    )
                } else {
                    format!("{}  v{} · {}{}", DIM, version, TAGLINE, RESET)
                }
            } else {
                String::new()
            };
            out.push_str(&format!(" {} {}\n", sprite, right));
        }

        if !fallback_shell.is_empty() {
            out.push_str(&format!(
                "               {}Ctrl+D to drop to {}{}\n",
                DIM, fallback_shell, RESET
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
