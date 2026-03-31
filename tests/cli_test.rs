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
        60,
    );
    assert!(!greeting.contains("___/"));
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
