use crate::companion::{render, Companion};
use crossterm::{cursor, execute, terminal};
use std::io::{self, Write};

const TAGLINE: &str = "you weren't using your terminal anyways";

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

// Each letter is 5 lines tall. Letters: C L A W D S H E L L
// Using █ block characters for filled look.
// Each letter is 6 chars wide (5 + 1 space separator).
const LETTER_HEIGHT: usize = 5;

// Each letter is 8 chars wide, 5 tall. Thin strokes with ░ background shading.
fn get_letter(ch: char) -> [&'static str; 5] {
    match ch {
        'C' => [
            "░░████░░",
            "░██░░░░░",
            "░██░░░░░",
            "░██░░░░░",
            "░░████░░",
        ],
        'L' => [
            "░██░░░░░",
            "░██░░░░░",
            "░██░░░░░",
            "░██░░░░░",
            "░██████░",
        ],
        'A' => [
            "░░████░░",
            "░██░░██░",
            "░██████░",
            "░██░░██░",
            "░██░░██░",
        ],
        'W' => [
            "░██░░░██",
            "░██░░░██",
            "░██░█░██",
            "░░██░██░",
            "░░░█░█░░",
        ],
        'D' => [
            "░█████░░",
            "░██░░██░",
            "░██░░██░",
            "░██░░██░",
            "░█████░░",
        ],
        'S' => [
            "░░████░░",
            "░██░░░░░",
            "░░████░░",
            "░░░░░██░",
            "░░████░░",
        ],
        'H' => [
            "░██░░██░",
            "░██░░██░",
            "░██████░",
            "░██░░██░",
            "░██░░██░",
        ],
        'E' => [
            "░██████░",
            "░██░░░░░",
            "░█████░░",
            "░██░░░░░",
            "░██████░",
        ],
        _ => [
            "░░░░░░░░",
            "░░░░░░░░",
            "░░░░░░░░",
            "░░░░░░░░",
            "░░░░░░░░",
        ],
    }
}

/// Build the full CLAWDSHELL text as 5 lines of block characters.
fn build_logo_lines() -> [String; LETTER_HEIGHT] {
    let word = "CLAWDSHELL";
    let mut lines = [
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];
    for ch in word.chars() {
        let letter = get_letter(ch);
        for (i, row) in letter.iter().enumerate() {
            lines[i].push_str(row);
        }
    }
    lines
}

/// Build partial logo (first N characters revealed) for animation.
fn build_partial_logo(n_chars: usize) -> [String; LETTER_HEIGHT] {
    let word = "CLAWDSHELL";
    let chars: Vec<char> = word.chars().collect();
    let show = n_chars.min(chars.len());

    let mut lines = [
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];
    for (idx, &ch) in chars.iter().enumerate() {
        let letter = if idx < show {
            get_letter(ch)
        } else {
            get_letter(' ')
        };
        for (i, row) in letter.iter().enumerate() {
            lines[i].push_str(row);
        }
    }
    lines
}

/// Animate the logo typing in letter-by-letter to the right of the companion.
pub fn animate_title(companion: &Companion) -> io::Result<()> {
    let mut stdout = io::stdout();
    let sprite_lines = render::render_sprite(companion, 0);

    let total_height = sprite_lines.len().max(LETTER_HEIGHT);

    // Draw companion + empty space first
    let empty_logo = build_partial_logo(0);
    println!();
    for i in 0..total_height {
        let sprite = if i < sprite_lines.len() {
            &sprite_lines[i]
        } else {
            "            "
        };
        let logo = if i < LETTER_HEIGHT {
            &empty_logo[i]
        } else {
            ""
        };
        writeln!(stdout, " {}  {}{}{}", sprite, BOLD, logo, RESET)?;
    }
    stdout.flush()?;

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Animate letters appearing one by one
    let word_len = "CLAWDSHELL".len();
    for n in 1..=word_len {
        let logo_lines = build_partial_logo(n);

        // Move cursor up to redraw
        execute!(stdout, cursor::MoveUp(total_height as u16))?;

        for i in 0..total_height {
            execute!(stdout, cursor::MoveToColumn(0))?;
            let sprite = if i < sprite_lines.len() {
                &sprite_lines[i]
            } else {
                "            "
            };
            let logo = if i < LETTER_HEIGHT {
                &logo_lines[i]
            } else {
                ""
            };
            write!(stdout, " {}  {}{}{}", sprite, BOLD, logo, RESET)?;
            execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
            writeln!(stdout)?;
        }
        stdout.flush()?;

        let delay = if n <= 3 { 100 } else if n <= 7 { 70 } else { 50 };
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }

    Ok(())
}

/// Render the static startup greeting (no animation).
pub fn render_greeting(
    tool_name: &str,
    fallback_shell: &str,
    companion: &Companion,
    terminal_width: u16,
) -> String {
    let mut out = String::new();
    let version = env!("CARGO_PKG_VERSION");
    let logo_lines = build_logo_lines();

    if terminal_width >= 80 {
        let sprite_lines = render::render_sprite(companion, 0);

        out.push('\n');

        let total_height = sprite_lines.len().max(LETTER_HEIGHT);
        for i in 0..total_height {
            let sprite = if i < sprite_lines.len() {
                &sprite_lines[i]
            } else {
                "            "
            };
            let logo = if i < LETTER_HEIGHT {
                format!("{}  {}{}", BOLD, logo_lines[i], RESET)
            } else {
                String::new()
            };
            out.push_str(&format!(" {} {}\n", sprite, logo));
        }

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
