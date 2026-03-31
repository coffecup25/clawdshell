use crate::companion::{render, Companion};

/// The ASCII art tagline — baked in as a static string.
/// Use a compact figlet-style font that fits in ~80 columns.
const TAGLINE: &str = r#"
 _   _                                   _   _            _
| | | | ___  _   _  __      _____ _ __ | |_| |_         | |_
| |_| |/ _ \| | | | \ \ /\ / / _ | '__||  _|  _|       | | |
 \__, | (_) | |_| |  \ V  V /  __| |   | | | |_    _   |_|_|
 |___/ \___/ \__,_|   \_/\_/ \___|_|    \_|  \__|  (_)  (_|_)
          _                                   _
 _  _ ___(_)_ _  __ _   _  _ ___ _  _ _ _   | |_ ___ _ _ _ __
| || (_-<| | ' \/ _` | | || / _ | || | '_|  |  _/ -_| '_| '  \
 \_,_/__/|_|_||_\__, |  \_, \___/\_,_|_|     \__\___|_| |_|_|_|
                |___/   |__/
                                  _
  __ _ _ _ _  ___ __ ____ _ _  _| _|___
 / _` | ' | || \ V  V / _` | || |_  (_-<
 \__,_|_||_\_, |\_/\_/\__,_|\_, /__//__/
           |__/              |__/
"#;

const TAGLINE_NARROW: &str = "you weren't using your terminal anyways";

pub fn render_greeting(
    tool_name: &str,
    fallback_shell: &str,
    companion: &Companion,
    terminal_width: u16,
) -> String {
    let mut out = String::new();

    if terminal_width >= 100 {
        out.push_str(TAGLINE);
        out.push_str(&format!("  — {} —\n\n", TAGLINE_NARROW));
        let sprite_lines = render::render_sprite(companion, 0);
        let info_lines = [
            format!("clawdshell v{} — launching {}", env!("CARGO_PKG_VERSION"), tool_name),
            format!("Ctrl+D to drop to {}", fallback_shell),
        ];
        for (i, line) in sprite_lines.iter().enumerate() {
            let right = if i < info_lines.len() { &info_lines[i] } else { "" };
            out.push_str(&format!("{}   {}\n", line, right));
        }
    } else {
        let face = render::render_face(companion);
        out.push_str(&format!("{}\n", TAGLINE_NARROW));
        out.push_str(&format!("{} launching {}... (Ctrl+D for {})\n", face, tool_name, fallback_shell));
    }

    out
}
