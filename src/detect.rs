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
///
/// Handles bash, zsh, fish, and PowerShell. Uses an interactive login shell
/// (-li) so that both .zprofile/.bash_profile AND .zshrc/.bashrc PATH
/// entries are captured. Timeout prevents hanging on slow shell configs.
pub fn inherit_shell_environment(fallback_shell: &str) {
    let shell_name = std::path::Path::new(fallback_shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    #[cfg(unix)]
    {
        let shell_args: &[&str] = match shell_name {
            // fish: PATH is a list, use string join to get colon-separated
            "fish" => &["-l", "-c", "string join : $PATH"],
            // bash/zsh/sh: interactive login to capture both profile and rc
            _ => &["-l", "-i", "-c", "echo $PATH"],
        };

        let result = Command::new(fallback_shell)
            .args(shell_args)
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output();

        match result {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() && path.contains('/') {
                    std::env::set_var("PATH", &path);
                }
            }
            _ => {
                // Fallback: try non-interactive login shell
                let result = Command::new(fallback_shell)
                    .args(["-l", "-c", "echo $PATH"])
                    .stdin(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .output();
                if let Ok(output) = result {
                    if output.status.success() {
                        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !path.is_empty() && path.contains('/') {
                            std::env::set_var("PATH", &path);
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        // PowerShell: get the PATH from a login session
        let result = Command::new(fallback_shell)
            .args(["-NoProfile", "-Command", "echo $env:PATH"])
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
