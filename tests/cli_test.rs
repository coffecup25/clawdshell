#[test]
fn test_greeting_contains_tagline() {
    let greeting = clawdshell::greeting::render_greeting(
        "claude", "/bin/zsh",
        &clawdshell::companion::generate("test"),
        120,
    );
    assert!(greeting.contains("terminal"));
    assert!(greeting.contains("anyways"));
}

#[test]
fn test_greeting_narrow_uses_face() {
    let greeting = clawdshell::greeting::render_greeting(
        "claude", "/bin/zsh",
        &clawdshell::companion::generate("test"),
        50,
    );
    assert!(greeting.contains("CLAWDSHELL"));
    assert!(greeting.contains("launching claude"));
}

#[test]
fn test_greeting_contains_tool_name() {
    let greeting = clawdshell::greeting::render_greeting(
        "codex", "/bin/bash",
        &clawdshell::companion::generate("test"),
        120,
    );
    assert!(greeting.contains("codex"));
    assert!(greeting.contains("/bin/bash"));
}

#[test]
fn test_shell_run_c_flag() {
    let status = clawdshell::shell::run_command_via_shell("/bin/sh", "true", &[]);
    assert!(status.is_ok());
    assert!(status.unwrap().success());
}

#[test]
fn test_shell_run_c_flag_with_positional_args() {
    let status = clawdshell::shell::run_command_via_shell(
        "/bin/sh",
        "test \"$1\" = \"hello\"",
        &["arg0".to_string(), "hello".to_string()],
    );
    assert!(status.is_ok());
    assert!(status.unwrap().success());
}

// --- Integration tests that run the compiled binary ---

use std::process::Command;

#[test]
fn test_cli_help_contains_config_reference() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CONFIG:"), "Help should contain CONFIG section");
    assert!(stdout.contains("claude"), "Help should mention claude");
    assert!(stdout.contains("SUPPORTED TOOLS:"), "Help should list supported tools");
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--version"])
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clawdshell"), "Version should contain clawdshell");
    assert!(stdout.contains("0.1.0"), "Version should contain 0.1.0");
}

#[test]
fn test_cli_companion_flag() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, r#"
[companion]
seed = "test123"
"#).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--companion"])
        .env("CLAWDSHELL_CONFIG", config_path.to_str().unwrap())
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DEBUGGING"), "Card should show DEBUGGING stat");
    assert!(stdout.contains("CHAOS"), "Card should show CHAOS stat");
}

#[test]
fn test_cli_c_flag_executes_command() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-c", "echo clawdtest"])
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clawdtest"), "Should execute the command and output result");
}

#[test]
fn test_cli_c_flag_with_positional_args() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-c", "echo $1", "arg0", "hello_posix"])
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello_posix"), "Should pass positional args to -c command");
}

#[test]
fn test_cli_set_tool() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[defaults]\ntool = \"claude\"\n").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--set-tool", "codex"])
        .env("CLAWDSHELL_CONFIG", config_path.to_str().unwrap())
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("codex"), "Should confirm tool was set");

    // Verify config was updated
    let config_content = std::fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("codex"), "Config should contain new tool");
}
