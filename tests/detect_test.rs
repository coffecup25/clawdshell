#[test]
fn test_detect_fallback_shell_returns_something() {
    let shell = clawdshell::detect::detect_fallback_shell();
    assert!(!shell.is_empty());
}

#[test]
fn test_resolve_tool_with_known_binary() {
    let result = clawdshell::detect::resolve_tool_binary("echo", None);
    assert!(result.is_some());
}

#[test]
fn test_resolve_tool_with_command_override() {
    let result = clawdshell::detect::resolve_tool_binary("nonexistent", Some("/bin/sh"));
    assert!(result.is_some());
}

#[test]
fn test_resolve_tool_not_found() {
    let result = clawdshell::detect::resolve_tool_binary("definitely_not_a_real_tool_xyz", None);
    assert!(result.is_none());
}

#[test]
fn test_detect_available_tools() {
    let tools = clawdshell::detect::detect_available_tools();
    // Function should not panic, tools list may be empty
    let _ = tools.len(); // Function should not panic, tools list may be empty
}
