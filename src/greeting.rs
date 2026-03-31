use crate::companion::{render, Companion};
use crossterm::{cursor, execute, terminal};
use std::io::{self, Write};

const TAGLINE: &str = "you weren't using your terminal anyways";

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

// Larger bubble-charm style title box
const TITLE_TEXT: &str = "  C   L   A   W   D   S   H   E   L   L  ";
const BOX_WIDTH: usize = 52;

fn make_box_top() -> String {
    format!("╔{}╗", "═".repeat(BOX_WIDTH))
}

fn make_box_mid(content: &str) -> String {
    // Pad content to box width
    let pad = if content.len() < BOX_WIDTH {
        BOX_WIDTH - content.len()
    } else {
        0
    };
    format!("║{}{}║", content, " ".repeat(pad))
}

fn make_box_bottom() -> String {
    format!("╚{}╝", "═".repeat(BOX_WIDTH))
}

/// Animate the title box appearing letter-by-letter from left to right.
/// Call this after the companion has hatched.
pub fn animate_title() -> io::Result<()> {
    let mut stdout = io::stdout();
    let letters: Vec<char> = TITLE_TEXT.chars().collect();

    // Draw empty box first
    let top = make_box_top();
    let bottom = make_box_bottom();
    let empty_mid = make_box_mid("");

    writeln!(stdout)?;
    writeln!(stdout, "  {}{}{}", BOLD, top, RESET)?;
    writeln!(stdout, "  {}", empty_mid)?;
    writeln!(stdout, "  {}{}{}", BOLD, bottom, RESET)?;
    stdout.flush()?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Type letters in one by one
    let mut revealed = String::new();
    for (i, ch) in letters.iter().enumerate() {
        revealed.push(*ch);
        let mid = make_box_mid(&format!("{}{}{}", BOLD, revealed, RESET));

        // Move up to the middle line and redraw it
        execute!(stdout, cursor::MoveUp(2))?;
        execute!(stdout, cursor::MoveToColumn(0))?;
        write!(stdout, "  {}", mid)?;
        execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
        writeln!(stdout)?;
        // Move back down past the bottom line
        execute!(stdout, cursor::MoveDown(1))?;
        stdout.flush()?;

        // Speed: start slow, get faster
        let delay = if i < 5 { 80 } else if i < 15 { 50 } else { 35 };
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }

    // Final flash: redraw complete box bold
    std::thread::sleep(std::time::Duration::from_millis(100));
    execute!(stdout, cursor::MoveUp(3))?;
    execute!(stdout, cursor::MoveToColumn(0))?;
    writeln!(stdout, "  {}{}{}", BOLD, top, RESET)?;
    let final_mid = make_box_mid(&format!("{}{}{}", BOLD, TITLE_TEXT, RESET));
    writeln!(stdout, "  {}", final_mid)?;
    writeln!(stdout, "  {}{}{}", BOLD, bottom, RESET)?;
    stdout.flush()?;

    Ok(())
}

/// Render the static startup greeting (for normal shell launch).
pub fn render_greeting(
    tool_name: &str,
    fallback_shell: &str,
    companion: &Companion,
    terminal_width: u16,
) -> String {
    let mut out = String::new();
    let version = env!("CARGO_PKG_VERSION");

    let top = make_box_top();
    let mid = make_box_mid(TITLE_TEXT);
    let bottom = make_box_bottom();
    let title_box = [top.as_str(), mid.as_str(), bottom.as_str()];

    if terminal_width >= 70 {
        let sprite_lines = render::render_sprite(companion, 0);

        out.push('\n');

        let total_lines = sprite_lines.len().max(title_box.len() + 1);
        for i in 0..total_lines {
            let sprite = if i < sprite_lines.len() {
                &sprite_lines[i]
            } else {
                "            "
            };
            let right = if i < title_box.len() {
                format!("{}  {}{}", BOLD, title_box[i], RESET)
            } else if i == title_box.len() {
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
