use crate::companion;
use crate::companion::animate::EGG_FRAMES;
use crate::companion::Companion;
use crate::config::Config;
use crate::detect;
use crate::greeting;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::{cursor, execute, terminal};
use dialoguer::{Confirm, Select};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use std::io::Write;
use std::process::Command;
use std::time::{Duration, Instant};

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";
const NICE_ORANGE_ANSI: &str = "\x1b[38;2;217;119;87m";
const NICE_ORANGE: Color = Color::Rgb(217, 119, 87);

const IDLE_SEQUENCE: &[i8] = &[0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, 2, 0, 0, 0];

/// Install Claude Code if not already present. Shows animated companion + progress bar.
fn ensure_claude_code(companion: &Companion) {
    // Check if claude is already available
    if which::which("claude").is_ok() {
        println!("  {}✓{} Claude Code already installed\n", NICE_ORANGE_ANSI, RESET);
        return;
    }

    // Check if npm is available
    if which::which("npm").is_err() {
        println!(
            "  {}✗{} Claude Code not found and {}npm{} is not installed.",
            YELLOW, RESET, BOLD, RESET
        );
        println!(
            "    Install Node.js from: {}{}https://nodejs.org{}\n",
            BOLD, CYAN, RESET
        );
        return;
    }

    println!(
        "  {}Installing Claude Code...{}\n",
        NICE_ORANGE_ANSI, RESET
    );

    // Spawn npm install in the background
    let mut child = match Command::new("npm")
        .args(["install", "-g", "@anthropic-ai/claude-code"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            println!(
                "  {}✗{} Failed to start npm: {}\n",
                RED, RESET, e
            );
            return;
        }
    };

    let mut stdout = std::io::stdout();

    // Draw initial companion sprite
    let sprite_lines = companion::render::render_sprite(companion, 0);
    let sprite_height = sprite_lines.len();
    for line in &sprite_lines {
        let _ = execute!(stdout, cursor::MoveToColumn(0));
        write!(stdout, "     {}", line).ok();
        let _ = execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine));
        writeln!(stdout).ok();
    }

    // Draw progress bar line + status line (2 lines below sprite)
    let bar_width: usize = 30;
    writeln!(stdout).ok(); // blank line
    writeln!(stdout, "  Installing...").ok();
    stdout.flush().ok();

    // Total lines to move up: sprite_height + 2 (blank + status)
    let total_height = sprite_height + 2;

    let mut tick: usize = 0;
    let mut bounce_pos: usize = 0;
    let mut bounce_dir: i8 = 1;
    let bounce_max = bar_width - 6; // width of the bouncing highlight

    loop {
        // Check if child is done
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => {}
            Err(_) => break,
        }

        std::thread::sleep(std::time::Duration::from_millis(150));
        tick += 1;

        // Move up to redraw everything
        let _ = execute!(stdout, cursor::MoveUp(total_height as u16));

        // Draw companion frame
        let seq_idx = (tick / 3) % IDLE_SEQUENCE.len();
        let frame_code = IDLE_SEQUENCE[seq_idx];
        let lines = if frame_code == -1 {
            companion::render::render_sprite_blink(companion, 0)
        } else {
            companion::render::render_sprite(companion, frame_code as usize)
        };

        for line in &lines {
            let _ = execute!(stdout, cursor::MoveToColumn(0));
            write!(stdout, "     {}", line).ok();
            let _ = execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine));
            writeln!(stdout).ok();
        }

        // Blank line
        let _ = execute!(stdout, cursor::MoveToColumn(0));
        let _ = execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine));
        writeln!(stdout).ok();

        // Build bouncing progress bar
        let mut bar = String::new();
        for i in 0..bar_width {
            if i >= bounce_pos && i < bounce_pos + 6 {
                bar.push('█');
            } else {
                bar.push('░');
            }
        }

        let _ = execute!(stdout, cursor::MoveToColumn(0));
        write!(
            stdout,
            "  {}{}{}  Installing Claude Code...",
            NICE_ORANGE_ANSI, bar, RESET
        )
        .ok();
        let _ = execute!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine));
        writeln!(stdout).ok();

        stdout.flush().ok();

        // Update bounce position
        if bounce_dir > 0 {
            if bounce_pos >= bounce_max {
                bounce_dir = -1;
                bounce_pos = bounce_pos.saturating_sub(1);
            } else {
                bounce_pos += 1;
            }
        } else {
            if bounce_pos == 0 {
                bounce_dir = 1;
                bounce_pos += 1;
            } else {
                bounce_pos -= 1;
            }
        }
    }

    // Move up to overwrite animation area
    let _ = execute!(stdout, cursor::MoveUp(total_height as u16));
    for _ in 0..total_height {
        let _ = execute!(stdout, cursor::MoveToColumn(0));
        let _ = execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine));
        writeln!(stdout).ok();
    }
    let _ = execute!(stdout, cursor::MoveUp(total_height as u16));

    // Show result
    let status = child.wait();
    match status {
        Ok(s) if s.success() => {
            println!(
                "  {}✓{} Claude Code installed successfully\n",
                NICE_ORANGE_ANSI, RESET
            );
        }
        _ => {
            println!(
                "  {}✗{} Failed to install Claude Code. Try manually:",
                RED, RESET
            );
            println!(
                "    {}npm install -g @anthropic-ai/claude-code{}\n",
                DIM, RESET
            );
        }
    }
}

fn generate_seed() -> String {
    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf).expect("Failed to generate random seed");
    format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]
    )
}

// ── Hatching animation phases ──────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum HatchPhase {
    Egg,       // egg animation frames
    Reveal,    // companion revealed, title typing in
    Hatched,   // final state with selection
}

/// Run Screen 1 (egg hatching, title animation, companion selection) using ratatui.
/// Returns `true` if the user chose to reroll.
fn run_hatch_screen(companion: &Companion) -> bool {
    // Set up ratatui terminal on alternate screen
    let mut terminal = ratatui::init();
    let result = hatch_render_loop(&mut terminal, companion);
    ratatui::restore();
    result
}

fn hatch_render_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    companion: &Companion,
) -> bool {
    let egg_sequence: &[usize] = &[0, 1, 2, 1, 2, 3, 4, 5, 6, 7];
    let egg_timing: &[u64] = &[800, 300, 300, 300, 300, 600, 600, 400, 400, 300];

    let mut phase = HatchPhase::Egg;
    let mut egg_frame_idx: usize = 0;
    let mut phase_start = Instant::now();
    let mut title_chars: usize = 0;
    let mut selection: usize = 0; // 0 = Keep, 1 = Reroll

    // Pre-render sprite lines
    let sprite_lines = companion::render::render_sprite(companion, 0);

    loop {
        // Draw
        let _ = terminal.draw(|frame| {
            let area = frame.area();

            match phase {
                HatchPhase::Egg => {
                    render_egg_frame(frame, area, egg_sequence[egg_frame_idx]);
                }
                HatchPhase::Reveal => {
                    render_companion_with_title(frame, area, &sprite_lines, title_chars, companion, false, 0);
                }
                HatchPhase::Hatched => {
                    render_companion_with_title(frame, area, &sprite_lines, 10, companion, true, selection);
                }
            }
        });

        // Phase transitions
        match phase {
            HatchPhase::Egg => {
                let delay = egg_timing[egg_frame_idx];
                if phase_start.elapsed() >= Duration::from_millis(delay) {
                    egg_frame_idx += 1;
                    if egg_frame_idx >= egg_sequence.len() {
                        // Transition to reveal
                        phase = HatchPhase::Reveal;
                        title_chars = 0;
                        phase_start = Instant::now();
                    } else {
                        phase_start = Instant::now();
                    }
                }
                // Check for Ctrl+C / Esc / Enter to skip animation
                if event::poll(Duration::from_millis(30)).unwrap_or(false) {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                    std::process::exit(0);
                                }
                                KeyCode::Esc | KeyCode::Enter => {
                                    // Skip to hatched state
                                    phase = HatchPhase::Hatched;
                                    title_chars = 10;
                                    selection = 0;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            HatchPhase::Reveal => {
                if title_chars < 10 {
                    let delay = if title_chars < 3 {
                        60
                    } else if title_chars < 7 {
                        40
                    } else {
                        25
                    };
                    if phase_start.elapsed() >= Duration::from_millis(delay) {
                        title_chars += 1;
                        phase_start = Instant::now();
                    }
                    // Check for Ctrl+C / Esc / Enter to skip animation
                    if event::poll(Duration::from_millis(10)).unwrap_or(false) {
                        if let Ok(Event::Key(key)) = event::read() {
                            if key.kind == KeyEventKind::Press {
                                match key.code {
                                    KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                        std::process::exit(0);
                                    }
                                    KeyCode::Esc | KeyCode::Enter => {
                                        title_chars = 10;
                                        phase = HatchPhase::Hatched;
                                        selection = 0;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                } else {
                    // Title fully revealed — move to hatched
                    phase = HatchPhase::Hatched;
                    selection = 0;
                }
            }
            HatchPhase::Hatched => {
                // Handle keyboard input for selection
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Up | KeyCode::Char('k') => {
                                    selection = 0;
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    selection = 1;
                                }
                                KeyCode::Enter => {
                                    return selection == 1; // true = reroll
                                }
                                KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                    std::process::exit(0);
                                }
                                KeyCode::Esc => {
                                    std::process::exit(0);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        std::thread::sleep(Duration::from_millis(16)); // ~60fps cap
    }
}

/// Render an egg frame centered on screen with "An egg appeared..." below.
fn render_egg_frame(frame: &mut ratatui::Frame, area: Rect, egg_idx: usize) {
    let egg = EGG_FRAMES[egg_idx];
    let egg_height = egg.len() as u16;
    let total_height = egg_height + 2; // egg + blank + label
    let egg_display_width = 14u16;

    // Center both vertically and horizontally
    let y_start = area.height.saturating_sub(total_height) / 2;
    let x_center = area.width.saturating_sub(egg_display_width) / 2;

    let mut lines: Vec<Line> = Vec::new();
    for row in egg {
        lines.push(Line::from(Span::raw(*row)));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " An egg appeared...",
        Style::default().add_modifier(Modifier::DIM),
    )));

    let paragraph = Paragraph::new(lines);
    let egg_area = Rect::new(x_center, area.y + y_start, egg_display_width + 6, total_height);
    frame.render_widget(paragraph, egg_area);
}

/// Render companion sprite + CLAWDSHELL title (partially or fully revealed),
/// plus optional hatched message and selection.
/// `x_offset` allows sliding the companion from center to left during reveal.
fn render_companion_with_title(
    frame: &mut ratatui::Frame,
    area: Rect,
    sprite_lines: &[String],
    title_chars: usize,
    companion: &Companion,
    show_hatched: bool,
    selection: usize,
) {
    let logo_lines = greeting::build_partial_logo(title_chars);
    let logo_height = greeting::LETTER_HEIGHT;
    let sprite_height = sprite_lines.len();
    let row_count = sprite_height.max(logo_height);

    // Measure actual display width of the full logo
    let logo_display_w = greeting::logo_display_width() as u16;
    let sprite_display_w = 14u16;
    let gap = 2u16;
    let total_content_w = sprite_display_w + gap + logo_display_w;

    // If the logo + sprite is wider than terminal, just left-align everything
    let final_x = if total_content_w >= area.width {
        0u16
    } else {
        // Center the whole thing, but at minimum x=1
        ((area.width.saturating_sub(total_content_w)) / 2).max(1)
    };

    let center_x = area.width.saturating_sub(sprite_display_w) / 2;

    // Interpolate x position: center → final as title_chars goes 0 → 10
    let progress = (title_chars as f32 / 10.0).min(1.0);
    let x_pos = center_x as f32 - (center_x as f32 - final_x as f32) * progress;
    let x_pos = x_pos.max(0.0) as u16;

    // Calculate total content height
    let mut total_lines: usize = row_count;
    if show_hatched {
        total_lines += 5; // tagline + blank + hatched msg + blank + selection
    }

    // Center vertically
    let y_start = (area.height as usize).saturating_sub(total_lines) / 2;

    let mut lines: Vec<Line> = Vec::new();

    // Companion sprite side-by-side with logo
    let pad = " ".repeat(x_pos as usize);
    let available_for_logo = (area.width as usize)
        .saturating_sub(x_pos as usize)
        .saturating_sub(sprite_display_w as usize)
        .saturating_sub(gap as usize);

    for i in 0..row_count {
        let sprite_part = if i < sprite_height {
            &sprite_lines[i]
        } else {
            "            "
        };
        let logo_part = if i < logo_height {
            // Truncate logo if it would overflow
            let full = &logo_lines[i];
            if full.chars().count() > available_for_logo {
                let truncated: String = full.chars().take(available_for_logo).collect();
                truncated
            } else {
                full.to_string()
            }
        } else {
            String::new()
        };

        let spans = vec![
            Span::raw(format!("{}{}  ", pad, sprite_part)),
            Span::styled(
                logo_part,
                Style::default()
                    .fg(NICE_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
        ];
        lines.push(Line::from(spans));
    }

    if show_hatched {
        // Tagline
        lines.push(Line::from(Span::styled(
            "  you weren't using your terminal anyways",
            Style::default().add_modifier(Modifier::DIM),
        )));
        lines.push(Line::from(""));

        // Hatched message
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("✨ {} hatched! ✨", companion.name),
                Style::default()
                    .fg(NICE_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Selection
        let keep_style = if selection == 0 {
            Style::default()
                .fg(NICE_ORANGE)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };
        let reroll_style = if selection == 1 {
            Style::default()
                .fg(NICE_ORANGE)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };

        let keep_prefix = if selection == 0 { "> " } else { "  " };
        let reroll_prefix = if selection == 1 { "> " } else { "  " };

        lines.push(Line::from(Span::styled(
            format!("  {} Keep this companion", keep_prefix),
            keep_style,
        )));
        lines.push(Line::from(Span::styled(
            format!("  {} Reroll companion", reroll_prefix),
            reroll_style,
        )));
    }

    let content_area = Rect::new(
        area.x,
        area.y + y_start as u16,
        area.width,
        (lines.len() as u16).min(area.height.saturating_sub(y_start as u16)),
    );
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, content_area);
}

pub fn install(config: &mut Config) {
    let width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);

    // --- Companion hatching & selection (Screen 1 — ratatui) ---
    // Always generate a fresh seed for install — this is the onboarding experience
    config.companion.seed = Some(generate_seed());

    loop {
        let c = companion::generate(config.companion.seed.as_deref().unwrap());

        let reroll = run_hatch_screen(&c);

        if reroll {
            config.companion.seed = Some(generate_seed());
            continue;
        }
        break;
    }

    // --- Setup (Screen 2 — normal scrolling terminal) ---
    print!("\x1b[2J\x1b[H");
    let c = companion::generate(config.companion.seed.as_deref().unwrap());
    print!("{}", greeting::render_greeting("", "", &c, width));

    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        DIM, RESET
    );
    println!("  {}{}Setup{}\n", BOLD, NICE_ORANGE_ANSI, RESET);

    let current_exe = std::env::current_exe().expect("Failed to get executable path");
    let exe_path = current_exe.to_string_lossy().to_string();

    // Detect current shell
    let current_shell = detect::detect_fallback_shell();
    println!("  {}Detected shell:{} {}", DIM, RESET, current_shell);
    config.defaults.fallback_shell = Some(current_shell.clone());

    // Inherit shell environment for tool detection
    detect::inherit_shell_environment(&current_shell);

    // Ensure Claude Code is installed
    ensure_claude_code(&c);

    // Detect available tools and let user pick
    let tools = detect::detect_available_tools();
    if tools.is_empty() {
        println!(
            "  {}No AI tools found in PATH. Default: claude{}",
            DIM, RESET
        );
    } else {
        println!(
            "  {}Found:{} {}\n",
            DIM,
            RESET,
            tools
                .iter()
                .map(|t| format!("{}{}{}", NICE_ORANGE_ANSI, t, RESET))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let default_idx = tools
            .iter()
            .position(|&t| t == config.defaults.tool.as_str())
            .unwrap_or(0);

        let choice = Select::new()
            .with_prompt("  Which tool should launch when you open a terminal?")
            .items(&tools)
            .default(default_idx)
            .interact()
            .unwrap_or(default_idx);

        config.defaults.tool = tools[choice].to_string();
        println!(
            "\n  {}Default tool:{} {}{}{}\n",
            DIM, RESET, NICE_ORANGE_ANSI, config.defaults.tool, RESET
        );
    }

    // Save config
    let config_path = Config::default_path();
    if let Err(e) = config.save_to(&config_path) {
        eprintln!("  {}{}Failed to save config: {}{}", YELLOW, BOLD, e, RESET);
        return;
    }
    println!(
        "  {}Config saved:{} {}\n",
        DIM,
        RESET,
        config_path.display()
    );

    // --- Shell Registration (Screen 3 — normal scrolling terminal) ---
    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        DIM, RESET
    );
    println!("  {}{}Shell Registration{}\n", BOLD, NICE_ORANGE_ANSI, RESET);

    #[cfg(unix)]
    let registered = unix_install(&exe_path);

    #[cfg(windows)]
    let registered = windows_install(&exe_path);

    println!(
        "\n{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        DIM, RESET
    );
    if registered {
        println!(
            "  {}{}Done!{} Open a new terminal to start using clawdshell.\n",
            BOLD, NICE_ORANGE_ANSI, RESET
        );
    } else {
        println!(
            "  {}Setup cancelled.{} Run {}clawdshell --install{} to try again.\n",
            DIM, RESET, BOLD, RESET
        );
    }
}

pub fn uninstall(config: &Config) {
    println!("\n  {}{}CLAWDSHELL{} — Uninstall", BOLD, NICE_ORANGE_ANSI, RESET);
    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}\n",
        DIM, RESET
    );

    let current_exe = std::env::current_exe().expect("Failed to get executable path");
    let exe_path = current_exe.to_string_lossy().to_string();

    let fallback = config
        .defaults
        .fallback_shell
        .as_deref()
        .unwrap_or("/bin/sh");

    println!("  This will:");
    println!(
        "    1. Restore your shell to: {}{}{}",
        NICE_ORANGE_ANSI, fallback, RESET
    );
    #[cfg(unix)]
    println!("    2. Remove clawdshell from /etc/shells");
    println!();

    let proceed = Confirm::new()
        .with_prompt("  Proceed?")
        .default(false)
        .interact()
        .unwrap_or(false);

    if !proceed {
        println!("  Aborted.");
        return;
    }

    #[cfg(unix)]
    unix_uninstall(&exe_path, fallback);

    #[cfg(windows)]
    windows_uninstall();

    println!(
        "\n  {}{}Done!{} Config preserved at: {}\n",
        BOLD,
        NICE_ORANGE_ANSI,
        RESET,
        Config::default_path().display()
    );
}

#[cfg(unix)]
fn unix_install(exe_path: &str) -> bool {
    println!("  This will:");
    println!(
        "    1. Add clawdshell to {}/etc/shells{} (requires sudo)",
        DIM, RESET
    );
    println!(
        "    2. Set it as your login shell via {}chsh{}\n",
        DIM, RESET
    );

    let proceed = Confirm::new()
        .with_prompt("  Proceed?")
        .default(true)
        .interact()
        .unwrap_or(false);

    if !proceed {
        return false;
    }

    println!();

    // Add to /etc/shells
    let shells_content = std::fs::read_to_string("/etc/shells").unwrap_or_default();
    if !shells_content.contains(exe_path) {
        let status = Command::new("sudo")
            .args(["sh", "-c", &format!("echo '{}' >> /etc/shells", exe_path)])
            .status();
        match status {
            Ok(s) if s.success() => println!("  {}✓{} Added to /etc/shells", NICE_ORANGE_ANSI, RESET),
            _ => {
                eprintln!("  {}✗{} Failed to add to /etc/shells", YELLOW, RESET);
                return false;
            }
        }
    } else {
        println!("  {}✓{} Already in /etc/shells", NICE_ORANGE_ANSI, RESET);
    }

    // Run chsh
    match Command::new("chsh").args(["-s", exe_path]).status() {
        Ok(s) if s.success() => {
            println!("  {}✓{} Login shell changed to clawdshell", NICE_ORANGE_ANSI, RESET);
            true
        }
        _ => {
            eprintln!(
                "  {}✗{} Failed to run chsh. Try: chsh -s {}",
                YELLOW, RESET, exe_path
            );
            false
        }
    }
}

#[cfg(unix)]
fn unix_uninstall(exe_path: &str, fallback: &str) {
    match Command::new("chsh").args(["-s", fallback]).status() {
        Ok(s) if s.success() => println!("  {}✓{} Shell restored to: {}", NICE_ORANGE_ANSI, RESET, fallback),
        _ => eprintln!(
            "  {}✗{} Failed. Run manually: chsh -s {}",
            YELLOW, RESET, fallback
        ),
    }

    let _ = Command::new("sudo")
        .args([
            "sed",
            "-i",
            "",
            &format!("\\|{}|d", exe_path),
            "/etc/shells",
        ])
        .status();
}

#[cfg(windows)]
fn windows_install(exe_path: &str) -> bool {
    println!("  Windows Terminal setup:");
    if let Some(settings_path) = find_windows_terminal_settings() {
        println!("  Found: {}", settings_path.display());
        let add = Confirm::new()
            .with_prompt("  Add clawdshell profile?")
            .default(true)
            .interact()
            .unwrap_or(false);
        if add {
            println!("  Profile addition not yet implemented. Add manually:");
        }
    } else {
        println!("  Windows Terminal not found.");
    }
    println!(
        "  Manual setup: Add {} as a profile in your terminal.",
        exe_path
    );
    true
}

#[cfg(windows)]
fn windows_uninstall() {
    println!("  Remove the clawdshell profile from Windows Terminal manually.");
}

#[cfg(windows)]
fn find_windows_terminal_settings() -> Option<std::path::PathBuf> {
    let local = std::env::var("LOCALAPPDATA").ok()?;
    let paths = [
        format!(
            "{}/Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState/settings.json",
            local
        ),
        format!(
            "{}/Packages/Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe/LocalState/settings.json",
            local
        ),
        format!("{}/Microsoft/Windows Terminal/settings.json", local),
    ];
    paths
        .iter()
        .map(std::path::PathBuf::from)
        .find(|p| p.exists())
}
