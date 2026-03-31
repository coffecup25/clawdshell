use crate::companion;
use crate::config::Config;
use crate::detect;
use crate::greeting;
use dialoguer::{Confirm, Select};
use std::io::Write;
use std::process::Command;

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

fn generate_seed() -> String {
    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf).expect("Failed to generate random seed");
    format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]
    )
}

pub fn install(config: &mut Config) {
    let width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);

    // --- Companion hatching & selection ---
    // Always generate a fresh seed for install — this is the onboarding experience
    config.companion.seed = Some(generate_seed());

    let mut play_hatch = true;
    loop {
        let c = companion::generate(config.companion.seed.as_deref().unwrap());

        // Clear screen
        print!("\x1b[2J\x1b[H");
        let _ = std::io::stdout().flush();

        // Always play egg hatching — every companion hatches from an egg
        println!();
        println!("  {}An egg appeared...{}\n", DIM, RESET);
        let _ = companion::animate::play_hatch(&c);

        // Animate the title box in letter by letter
        let _ = greeting::animate_title();

        println!();
        println!(
            "  {}✨ {}{}{} hatched! ✨{}",
            BOLD, GREEN, c.name, RESET, RESET
        );

        println!();
        let reroll = Select::new()
            .items(&["Keep this companion", "Reroll companion"])
            .default(0)
            .interact()
            .unwrap_or(0);

        if reroll == 1 {
            config.companion.seed = Some(generate_seed());
            continue;
        }
        break;
    }

    // --- Setup ---
    print!("\x1b[2J\x1b[H");
    let c = companion::generate(config.companion.seed.as_deref().unwrap());
    print!("{}", greeting::render_greeting("", "", &c, width));

    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        DIM, RESET
    );
    println!("  {}{}Setup{}\n", BOLD, CYAN, RESET);

    let current_exe = std::env::current_exe().expect("Failed to get executable path");
    let exe_path = current_exe.to_string_lossy().to_string();

    // Detect current shell
    let current_shell = detect::detect_fallback_shell();
    println!("  {}Detected shell:{} {}", DIM, RESET, current_shell);
    config.defaults.fallback_shell = Some(current_shell.clone());

    // Inherit shell environment for tool detection
    detect::inherit_shell_environment(&current_shell);

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
                .map(|t| format!("{}{}{}", GREEN, t, RESET))
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
            DIM, RESET, GREEN, config.defaults.tool, RESET
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

    // --- Shell Registration ---
    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        DIM, RESET
    );
    println!("  {}{}Shell Registration{}\n", BOLD, CYAN, RESET);

    #[cfg(unix)]
    unix_install(&exe_path);

    #[cfg(windows)]
    windows_install(&exe_path);

    println!(
        "\n{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        DIM, RESET
    );
    println!(
        "  {}{}Done!{} Open a new terminal to start using clawdshell.\n",
        BOLD, GREEN, RESET
    );
}

pub fn uninstall(config: &Config) {
    println!("\n  {}{}CLAWDSHELL{} — Uninstall", BOLD, CYAN, RESET);
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
        GREEN, fallback, RESET
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
        GREEN,
        RESET,
        Config::default_path().display()
    );
}

#[cfg(unix)]
fn unix_install(exe_path: &str) {
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
        println!("  Aborted.");
        return;
    }

    println!();

    // Add to /etc/shells
    let shells_content = std::fs::read_to_string("/etc/shells").unwrap_or_default();
    if !shells_content.contains(exe_path) {
        let status = Command::new("sudo")
            .args(["sh", "-c", &format!("echo '{}' >> /etc/shells", exe_path)])
            .status();
        match status {
            Ok(s) if s.success() => println!("  {}✓{} Added to /etc/shells", GREEN, RESET),
            _ => {
                eprintln!("  {}✗{} Failed to add to /etc/shells", YELLOW, RESET);
                return;
            }
        }
    } else {
        println!("  {}✓{} Already in /etc/shells", GREEN, RESET);
    }

    // Run chsh
    match Command::new("chsh").args(["-s", exe_path]).status() {
        Ok(s) if s.success() => {
            println!("  {}✓{} Login shell changed to clawdshell", GREEN, RESET)
        }
        _ => eprintln!(
            "  {}✗{} Failed to run chsh. Try: chsh -s {}",
            YELLOW, RESET, exe_path
        ),
    }
}

#[cfg(unix)]
fn unix_uninstall(exe_path: &str, fallback: &str) {
    match Command::new("chsh").args(["-s", fallback]).status() {
        Ok(s) if s.success() => println!("  {}✓{} Shell restored to: {}", GREEN, RESET, fallback),
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
fn windows_install(exe_path: &str) {
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
