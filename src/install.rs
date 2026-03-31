use crate::config::Config;
use crate::detect;
use std::process::Command;
use std::io::{self, Write, BufRead};

fn prompt_confirm(message: &str) -> bool {
    print!("{} [y/N] ", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap();
    input.trim().eq_ignore_ascii_case("y")
}

pub fn install(config: &mut Config) {
    println!("clawdshell --install");
    println!("=====================\n");

    let current_exe = std::env::current_exe().expect("Failed to get executable path");
    let exe_path = current_exe.to_string_lossy();

    // Detect current shell
    let current_shell = detect::detect_fallback_shell();
    println!("Detected current shell: {}", current_shell);
    config.defaults.fallback_shell = Some(current_shell.clone());

    // Detect available tools
    let tools = detect::detect_available_tools();
    if !tools.is_empty() {
        println!("Found AI tools: {}", tools.join(", "));
        if !tools.contains(&config.defaults.tool.as_str()) && !tools.is_empty() {
            config.defaults.tool = tools[0].to_string();
            println!("Set default tool to: {}", config.defaults.tool);
        }
    } else {
        println!("No AI tools found in PATH. Default: claude");
    }

    // Save config
    let config_path = Config::default_path();
    if let Err(e) = config.save_to(&config_path) {
        eprintln!("Failed to save config: {}", e);
        return;
    }
    println!("Config saved to: {}\n", config_path.display());

    #[cfg(unix)]
    unix_install(&exe_path);

    #[cfg(windows)]
    windows_install(&exe_path);

    println!("\nInstallation complete! Open a new terminal to start using clawdshell.");
}

pub fn uninstall(config: &Config) {
    println!("clawdshell --uninstall");
    println!("========================\n");

    let current_exe = std::env::current_exe().expect("Failed to get executable path");
    let exe_path = current_exe.to_string_lossy();

    let fallback = config.defaults.fallback_shell.as_deref().unwrap_or("/bin/sh");

    println!("This will:");
    println!("  1. Restore your shell to: {}", fallback);
    #[cfg(unix)]
    println!("  2. Remove clawdshell from /etc/shells");

    if !prompt_confirm("\nProceed?") {
        println!("Aborted.");
        return;
    }

    #[cfg(unix)]
    unix_uninstall(&exe_path, fallback);

    #[cfg(windows)]
    windows_uninstall();

    println!("\nUninstall complete. Config preserved at: {}", Config::default_path().display());
}

#[cfg(unix)]
fn unix_install(exe_path: &str) {
    println!("This will:");
    println!("  1. Add {} to /etc/shells (requires sudo)", exe_path);
    println!("  2. Set it as your login shell via chsh");

    if !prompt_confirm("\nProceed?") {
        println!("Aborted.");
        return;
    }

    // Add to /etc/shells
    let shells_content = std::fs::read_to_string("/etc/shells").unwrap_or_default();
    if !shells_content.contains(exe_path) {
        let status = Command::new("sudo")
            .args(["sh", "-c", &format!("echo '{}' >> /etc/shells", exe_path)])
            .status();
        match status {
            Ok(s) if s.success() => println!("Added to /etc/shells"),
            _ => { eprintln!("Failed to add to /etc/shells"); return; }
        }
    } else {
        println!("Already in /etc/shells");
    }

    // Run chsh
    match Command::new("chsh").args(["-s", exe_path]).status() {
        Ok(s) if s.success() => println!("Login shell changed to clawdshell"),
        _ => eprintln!("Failed to run chsh. Try: chsh -s {}", exe_path),
    }
}

#[cfg(unix)]
fn unix_uninstall(exe_path: &str, fallback: &str) {
    match Command::new("chsh").args(["-s", fallback]).status() {
        Ok(s) if s.success() => println!("Shell restored to: {}", fallback),
        _ => eprintln!("Failed. Run manually: chsh -s {}", fallback),
    }

    let _ = Command::new("sudo")
        .args(["sed", "-i", "", &format!("\\|{}|d", exe_path), "/etc/shells"])
        .status();
}

#[cfg(windows)]
fn windows_install(exe_path: &str) {
    println!("Windows Terminal setup:");
    if let Some(settings_path) = find_windows_terminal_settings() {
        println!("Found: {}", settings_path.display());
        if prompt_confirm("Add clawdshell profile?") {
            println!("Profile addition not yet implemented. Add manually:");
        }
    } else {
        println!("Windows Terminal not found.");
    }
    println!("Manual setup: Add {} as a profile in your terminal.", exe_path);
}

#[cfg(windows)]
fn windows_uninstall() {
    println!("Remove the clawdshell profile from Windows Terminal manually.");
}

#[cfg(windows)]
fn find_windows_terminal_settings() -> Option<std::path::PathBuf> {
    let local = std::env::var("LOCALAPPDATA").ok()?;
    let paths = [
        format!("{}/Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState/settings.json", local),
        format!("{}/Packages/Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe/LocalState/settings.json", local),
        format!("{}/Microsoft/Windows Terminal/settings.json", local),
    ];
    paths.iter().map(std::path::PathBuf::from).find(|p| p.exists())
}
