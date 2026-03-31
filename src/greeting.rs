use crate::companion::{render, Companion};
use crossterm::{cursor, execute, terminal};
use std::io::{self, Write};

const TAGLINE: &str = "you weren't using your terminal anyways";

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

const TITLE_TEXT: &str = "CLAWDSHELL";
const BOX_INNER_WIDTH: usize = 30;

fn box_top() -> String {
    format!("╔{}╗", "═".repeat(BOX_INNER_WIDTH))
}

fn box_mid(content: &str, content_display_len: usize) -> String {
    let pad = if content_display_len < BOX_INNER_WIDTH {
        BOX_INNER_WIDTH - content_display_len
    } else {
        0
    };
    format!("║{}{}║", content, " ".repeat(pad))
}

fn box_bottom() -> String {
    format!("╚{}╝", "═".repeat(BOX_INNER_WIDTH))
}

/// Animate the title box typing in letter-by-letter, to the right of the companion.
pub fn animate_title(companion: &Companion) -> io::Result<()> {
    let mut stdout = io::stdout();
    let sprite_lines = render::render_sprite(companion, 0);
    let sprite_width = 14; // sprite is ~12 chars + padding

    // The box is 3 lines. We'll align it vertically centered with the sprite.
    // sprite is 4-5 lines, box is 3 lines. Put box at lines 0-2 of sprite.
    let box_start_line = 0;

    let top = box_top();
    let bottom = box_bottom();

    // Draw sprite + empty box
    let total_lines = sprite_lines.len().max(box_start_line + 3);
    println!(); // blank line above
    for i in 0..total_lines {
        let sprite = if i < sprite_lines.len() {
            &sprite_lines[i]
        } else {
            "            "
        };
        let right = if i == box_start_line {
            format!("  {}{}{}", BOLD, top, RESET)
        } else if i == box_start_line + 1 {
            format!("  {}", box_mid("", 0))
        } else if i == box_start_line + 2 {
            format!("  {}{}{}", BOLD, bottom, RESET)
        } else {
            String::new()
        };
        writeln!(stdout, " {}{}", sprite, right)?;
    }
    stdout.flush()?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Type letters into the middle line, one by one
    let letters: Vec<char> = TITLE_TEXT.chars().collect();
    let mid_row_offset = total_lines - (box_start_line + 1); // lines from bottom to middle row

    // Position: the middle row of the box
    // We need to go up from current position to that row
    let lines_from_bottom = total_lines - (box_start_line + 1);

    let mut revealed = String::new();
    for (i, ch) in letters.iter().enumerate() {
        revealed.push(*ch);

        // Build the padded content: center the text in the box
        let text_with_padding = format!(
            "{}{}   {}{}",
            " ".repeat(3),
            BOLD,
            revealed,
            RESET
        );
        let display_len = 3 + 3 + revealed.len(); // padding + "   " + text
        let mid = box_mid(&text_with_padding, display_len);

        // Move up to the middle row
        execute!(stdout, cursor::MoveUp(lines_from_bottom as u16))?;
        execute!(stdout, cursor::MoveToColumn(0))?;

        // Redraw: sprite + box middle
        let sprite = if (box_start_line + 1) < sprite_lines.len() {
            &sprite_lines[box_start_line + 1]
        } else {
            "            "
        };
        write!(stdout, " {}  {}", sprite, mid)?;
        execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;

        // Move back down
        execute!(stdout, cursor::MoveDown(lines_from_bottom as u16))?;
        execute!(stdout, cursor::MoveToColumn(0))?;
        stdout.flush()?;

        let delay = if i < 3 { 90 } else if i < 7 { 60 } else { 40 };
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }

    Ok(())
}

/// Render the static startup greeting (for normal shell launch, no animation).
pub fn render_greeting(
    tool_name: &str,
    fallback_shell: &str,
    companion: &Companion,
    terminal_width: u16,
) -> String {
    let mut out = String::new();
    let version = env!("CARGO_PKG_VERSION");

    let top = box_top();
    let text_padded = format!("   {}   ", TITLE_TEXT);
    let mid = box_mid(&text_padded, text_padded.len());
    let bottom = box_bottom();
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
