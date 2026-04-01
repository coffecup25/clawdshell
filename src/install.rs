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

    println!(
        "  {}Installing Claude Code...{}\n",
        NICE_ORANGE_ANSI, RESET
    );

    // Use the official installer for the current platform
    #[cfg(unix)]
    let install_result = Command::new("bash")
        .args(["-c", "curl -fsSL https://claude.ai/install.sh | bash"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    #[cfg(windows)]
    let install_result = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command",
               "irm https://claude.ai/install.ps1 | iex"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    let mut child = match install_result {
        Ok(c) => c,
        Err(_) => {
            // Fallback: try npm if primary installer not available
            if which::which("npm").is_err() {
                println!(
                    "  {}✗{} Could not install Claude Code automatically.",
                    YELLOW, RESET
                );
                #[cfg(unix)]
                println!(
                    "    Install manually: {}curl -fsSL https://claude.ai/install.sh | bash{}\n",
                    DIM, RESET
                );
                #[cfg(windows)]
                println!(
                    "    Install manually: {}irm https://claude.ai/install.ps1 | iex{}\n",
                    DIM, RESET
                );
                return;
            }
            match Command::new("npm")
                .args(["install", "-g", "@anthropic-ai/claude-code"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    println!("  {}✗{} Failed to install: {}\n", RED, RESET, e);
                    return;
                }
            }
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
            #[cfg(unix)]
            println!(
                "    {}curl -fsSL https://claude.ai/install.sh | bash{}\n",
                DIM, RESET
            );
            #[cfg(windows)]
            println!(
                "    {}irm https://claude.ai/install.ps1 | iex{}\n",
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

#[derive(Clone, Copy, PartialEq)]
enum HatchResult {
    Keep,
    Reroll,
    Quit,
}

/// Run Screen 1 (egg hatching, title animation, companion selection) using ratatui.
/// On Unix: opens /dev/tty directly so it works from curl | sh.
/// On Windows: uses stdout normally (user runs --install directly from console).
fn run_hatch_screen(companion: &Companion) -> HatchResult {
    use ratatui::backend::CrosstermBackend;

    #[cfg(unix)]
    let tty = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
    {
        Ok(f) => f,
        Err(_) => {
            eprintln!("clawdshell: no terminal available, skipping setup wizard");
            return HatchResult::Keep;
        }
    };

    let _ = crossterm::terminal::enable_raw_mode();

    #[cfg(unix)]
    let backend = CrosstermBackend::new(tty);
    #[cfg(windows)]
    let backend = CrosstermBackend::new(std::io::stdout());

    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => {
            let _ = crossterm::terminal::disable_raw_mode();
            return HatchResult::Keep;
        }
    };

    let _ = crossterm::execute!(terminal.backend_mut(), terminal::EnterAlternateScreen);
    let _ = terminal.clear();

    let result = hatch_render_loop(&mut terminal, companion);

    // Restore terminal
    let _ = crossterm::execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    );
    let _ = crossterm::terminal::disable_raw_mode();

    result
}

/// Read a key from /dev/tty directly, bypassing crossterm's event system.
/// Returns None if no key available within timeout.
#[cfg(unix)]
fn read_key_from_tty(tty_fd: i32, timeout_ms: u64) -> Option<KeyCode> {
    use std::io::Read;

    // Use poll() to check if data available
    let mut pfd = libc::pollfd {
        fd: tty_fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let ready = unsafe { libc::poll(&mut pfd, 1, timeout_ms as i32) };
    if ready <= 0 {
        return None;
    }

    let mut buf = [0u8; 8];
    let n = unsafe { libc::read(tty_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    if n <= 0 {
        return None;
    }
    let bytes = &buf[..n as usize];

    match bytes {
        [3] => Some(KeyCode::Char('c')),          // Ctrl+C
        [13] | [10] => Some(KeyCode::Enter),       // Enter
        [27] => Some(KeyCode::Esc),                // Esc
        [27, 91, 65] => Some(KeyCode::Up),         // Arrow up
        [27, 91, 66] => Some(KeyCode::Down),       // Arrow down
        [27, 91, 67] => Some(KeyCode::Right),      // Arrow right
        [27, 91, 68] => Some(KeyCode::Left),       // Arrow left
        [b'k'] | [b'K'] => Some(KeyCode::Char('k')),
        [b'j'] | [b'J'] => Some(KeyCode::Char('j')),
        [b'q'] | [b'Q'] => Some(KeyCode::Char('q')),
        [b'r'] | [b'R'] => Some(KeyCode::Char('r')),
        _ => None,
    }
}

/// Poll for a key press.
/// Unix: reads from /dev/tty fd directly (works even when stdin is piped).
/// Windows: uses crossterm events (console always available when run directly).
#[cfg(unix)]
fn poll_key_impl(tty_fd: i32, timeout_ms: u64) -> Option<KeyCode> {
    read_key_from_tty(tty_fd, timeout_ms)
}

#[cfg(windows)]
fn poll_key_impl(_unused: i32, timeout_ms: u64) -> Option<KeyCode> {
    if event::poll(Duration::from_millis(timeout_ms)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                return Some(key.code);
            }
        }
    }
    None
}

fn hatch_render_loop<W: std::io::Write>(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<W>>,
    companion: &Companion,
) -> HatchResult {
    #[cfg(unix)]
    let key_fd = unsafe {
        libc::open(b"/dev/tty\0".as_ptr() as *const libc::c_char, libc::O_RDONLY | libc::O_NONBLOCK)
    };
    #[cfg(windows)]
    let key_fd: i32 = 0; // unused on Windows, poll_key_impl ignores it

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
                if let Some(key) = poll_key_impl(key_fd, 30) {
                    match key {
                        KeyCode::Char('c') => return HatchResult::Quit,
                        KeyCode::Esc | KeyCode::Enter => {
                            phase = HatchPhase::Hatched;
                            title_chars = 10;
                            selection = 0;
                        }
                        _ => {}
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
                    if let Some(key) = poll_key_impl(key_fd, 10) {
                        match key {
                            KeyCode::Char('c') => return HatchResult::Quit,
                            KeyCode::Esc | KeyCode::Enter => {
                                title_chars = 10;
                                phase = HatchPhase::Hatched;
                                selection = 0;
                            }
                            _ => {}
                        }
                    }
                } else {
                    phase = HatchPhase::Hatched;
                    selection = 0;
                }
            }
            HatchPhase::Hatched => {
                if let Some(key) = poll_key_impl(key_fd, 50) {
                    match key {
                        KeyCode::Up | KeyCode::Char('k') => selection = 0,
                        KeyCode::Down | KeyCode::Char('j') => selection = 1,
                        KeyCode::Enter => {
                            #[cfg(unix)]
                            unsafe { libc::close(key_fd); }
                            return if selection == 1 { HatchResult::Reroll } else { HatchResult::Keep };
                        }
                        KeyCode::Char('c') | KeyCode::Esc | KeyCode::Char('q') => {
                            #[cfg(unix)]
                            unsafe { libc::close(key_fd); }
                            return HatchResult::Quit;
                        }
                        _ => {}
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
        // Tagline — centered under the title area
        let tagline = "Lets be real, you weren't using the terminal anyway";
        let tagline_pad = (sprite_display_w as usize + gap as usize + logo_display_w as usize)
            .saturating_sub(tagline.len()) / 2
            + x_pos as usize;
        lines.push(Line::from(Span::styled(
            format!("{}{}", " ".repeat(tagline_pad), tagline),
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

        match run_hatch_screen(&c) {
            HatchResult::Reroll => {
                config.companion.seed = Some(generate_seed());
                continue;
            }
            HatchResult::Quit => {
                println!("\n  {}Install cancelled.{}\n", DIM, RESET);
                return;
            }
            HatchResult::Keep => break,
        }
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
    let chsh_ok = match Command::new("chsh").args(["-s", exe_path]).status() {
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
    };

    // On macOS, ensure Terminal.app uses the login shell (not a hardcoded command).
    // Without this, Terminal.app may ignore chsh if its profile has a CommandString override.
    if chsh_ok && std::path::Path::new("/usr/libexec/PlistBuddy").exists() {
        configure_macos_terminal();
    }

    chsh_ok
}

/// On macOS, set Terminal.app's default profile to use the login shell
/// instead of a hardcoded command. This makes `chsh` take effect in Terminal.app.
#[cfg(unix)]
fn configure_macos_terminal() {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let plist = format!("{}/Library/Preferences/com.apple.Terminal.plist", home);
    if !std::path::Path::new(&plist).exists() {
        return; // Terminal.app not configured / not macOS
    }

    // Get the default profile name (e.g. "Basic", "Pro", etc.)
    let profile_name = Command::new("defaults")
        .args(["read", "com.apple.Terminal", "Default Window Settings"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "Basic".to_string());

    // Set CommandIsShell = true so Terminal.app uses the login shell
    let key = format!("Set ':Window Settings:{}:CommandIsShell' true", profile_name);
    let ok = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", &key, &plist])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if ok {
        // Remove any CommandString override that would bypass the login shell
        let del_key = format!("Delete ':Window Settings:{}:CommandString'", profile_name);
        let _ = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", &del_key, &plist])
            .output();

        println!(
            "  {}✓{} Configured Terminal.app to use login shell",
            NICE_ORANGE_ANSI, RESET
        );
    }

    // Also handle Startup Window Settings profile if different
    let startup_name = Command::new("defaults")
        .args(["read", "com.apple.Terminal", "Startup Window Settings"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(startup) = startup_name {
        if startup != profile_name {
            let key = format!("Set ':Window Settings:{}:CommandIsShell' true", startup);
            let _ = Command::new("/usr/libexec/PlistBuddy")
                .args(["-c", &key, &plist])
                .output();
            let del_key = format!("Delete ':Window Settings:{}:CommandString'", startup);
            let _ = Command::new("/usr/libexec/PlistBuddy")
                .args(["-c", &del_key, &plist])
                .output();
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
    // Use Windows Terminal Fragments to register profile automatically.
    // Fragments are the official way for apps to add profiles without modifying
    // the user's settings.json. Supported since Windows Terminal 1.11+.
    // https://learn.microsoft.com/en-us/windows/terminal/json-fragment-extensions
    let local_app_data = match std::env::var("LOCALAPPDATA") {
        Ok(v) => v,
        Err(_) => {
            println!(
                "  {}✗{} LOCALAPPDATA not set — cannot register profile",
                YELLOW, RESET
            );
            return false;
        }
    };

    // Escape backslashes for JSON
    let exe_json = exe_path.replace('\\', "\\\\");
    let fragment_json = format!(
        "{{\n    \"profiles\": [\n        {{\n            \"name\": \"clawdshell\",\n            \"commandline\": \"{}\",\n            \"startingDirectory\": \"%USERPROFILE%\",\n            \"hidden\": false\n        }}\n    ]\n}}",
        exe_json
    );

    let mut registered = false;

    // Register fragment for Windows Terminal (stable)
    let wt_fragment_dir = std::path::PathBuf::from(&local_app_data)
        .join("Microsoft")
        .join("Windows Terminal")
        .join("Fragments")
        .join("clawdshell");

    if std::fs::create_dir_all(&wt_fragment_dir).is_ok() {
        if std::fs::write(wt_fragment_dir.join("profile.json"), &fragment_json).is_ok() {
            println!(
                "  {}✓{} Registered clawdshell profile (Windows Terminal)",
                NICE_ORANGE_ANSI, RESET
            );
            registered = true;
        }
    }

    // Also register for Windows Terminal Preview
    let preview_fragment_dir = std::path::PathBuf::from(&local_app_data)
        .join("Microsoft")
        .join("Windows Terminal Preview")
        .join("Fragments")
        .join("clawdshell");

    if std::fs::create_dir_all(&preview_fragment_dir).is_ok() {
        let _ = std::fs::write(preview_fragment_dir.join("profile.json"), &fragment_json);
    }

    if !registered {
        println!(
            "  {}✗{} Could not register profile fragment",
            YELLOW, RESET
        );
    }

    registered
}

#[cfg(windows)]
fn windows_uninstall() {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let dirs = [
            std::path::PathBuf::from(&local)
                .join("Microsoft")
                .join("Windows Terminal")
                .join("Fragments")
                .join("clawdshell"),
            std::path::PathBuf::from(&local)
                .join("Microsoft")
                .join("Windows Terminal Preview")
                .join("Fragments")
                .join("clawdshell"),
        ];
        for dir in &dirs {
            let _ = std::fs::remove_dir_all(dir);
        }
        println!(
            "  {}✓{} Removed clawdshell profile fragments",
            NICE_ORANGE_ANSI, RESET
        );
    }
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
