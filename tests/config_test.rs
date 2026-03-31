use std::path::PathBuf;

#[test]
fn test_default_config_has_claude_as_tool() {
    let config = clawdshell::config::Config::default();
    assert_eq!(config.defaults.tool, "claude");
    assert!(config.defaults.fallback_shell.is_none());
    assert!(config.companion.enabled);
    assert!(config.companion.seed.is_none());
}

#[test]
fn test_config_loads_from_toml_string() {
    let toml_str = r#"
[defaults]
tool = "codex"
fallback_shell = "/bin/zsh"

[companion]
enabled = false
seed = "abc123"

[tools.codex]
args = ["--full-auto"]
"#;
    let config: clawdshell::config::Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.defaults.tool, "codex");
    assert_eq!(config.defaults.fallback_shell, Some("/bin/zsh".to_string()));
    assert!(!config.companion.enabled);
    assert_eq!(config.companion.seed, Some("abc123".to_string()));
    let codex = config.tools.get("codex").unwrap();
    assert_eq!(codex.args, vec!["--full-auto"]);
}

#[test]
fn test_config_loads_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, r#"
[defaults]
tool = "gemini"

[companion]
enabled = true
"#).unwrap();
    let config = clawdshell::config::Config::load_from(&path).unwrap();
    assert_eq!(config.defaults.tool, "gemini");
}

#[test]
fn test_config_missing_file_returns_default() {
    let path = PathBuf::from("/nonexistent/config.toml");
    let config = clawdshell::config::Config::load_from(&path).unwrap();
    assert_eq!(config.defaults.tool, "claude");
}

#[test]
fn test_config_save_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut config = clawdshell::config::Config::default();
    config.defaults.fallback_shell = Some("/bin/fish".to_string());
    config.companion.seed = Some("deadbeef".to_string());
    config.save_to(&path).unwrap();

    let loaded = clawdshell::config::Config::load_from(&path).unwrap();
    assert_eq!(loaded.defaults.fallback_shell, Some("/bin/fish".to_string()));
    assert_eq!(loaded.companion.seed, Some("deadbeef".to_string()));
}
