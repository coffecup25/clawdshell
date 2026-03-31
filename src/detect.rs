use std::path::PathBuf;
use std::process::Command;

const KNOWN_TOOLS: &[&str] = &["claude", "codex", "gemini", "opencode", "aider", "forge"];

pub fn detect_fallback_shell() -> String {
    #[cfg(unix)]
    { std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()) }
    #[cfg(windows)]
    { std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string()) }
}

/// Source the user's shell environment to get the full PATH.
/// When clawdshell runs as a login shell, it doesn't inherit PATH from
/// .zshrc/.bash_profile/etc. This asks the fallback shell for its PATH
/// and sets it in the current process so tool resolution works.
pub fn inherit_shell_environment(fallback_shell: &str) {
    // Ask the shell to run as a login shell and print its PATH
    let result = Command::new(fallback_shell)
        .args(["-l", "-c", "echo $PATH"])
        .output();

    if let Ok(output) = result {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                std::env::set_var("PATH", &path);
            }
        }
    }
}

pub fn resolve_tool_binary(tool_name: &str, command_override: Option<&str>) -> Option<PathBuf> {
    if let Some(cmd) = command_override {
        let path = PathBuf::from(cmd);
        if path.is_absolute() && path.exists() { return Some(path); }
        if let Ok(p) = which::which(cmd) { return Some(p); }
    }
    which::which(tool_name).ok()
}

pub fn detect_available_tools() -> Vec<&'static str> {
    KNOWN_TOOLS.iter().filter(|name| which::which(name).is_ok()).copied().collect()
}
