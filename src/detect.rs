use std::path::PathBuf;

const KNOWN_TOOLS: &[&str] = &["claude", "codex", "gemini", "opencode", "aider", "forge"];

pub fn detect_fallback_shell() -> String {
    #[cfg(unix)]
    { std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()) }
    #[cfg(windows)]
    { std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string()) }
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
