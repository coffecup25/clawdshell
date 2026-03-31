# ClawdShell Design Spec

> A cross-platform login shell that launches AI coding tools instead of a traditional shell.

## Overview

ClawdShell is a Rust binary that acts as a valid login shell. When a terminal opens, it launches a configurable AI coding tool (Claude Code by default) and drops to a fallback shell when the tool exits. It supports macOS, Linux, and Windows.

## Use Case

Personal workstation shell. The user sets clawdshell as their login shell so every terminal session starts with an AI coding tool. Not designed as a restricted/kiosk shell.

## Core Behavior

### Startup Flow

```
Terminal launches clawdshell
  -> Load config from ~/.config/clawdshell/config.toml (optional, defaults if missing)
  -> Resolve tool binary (lookup in PATH or use configured command)
  -> Show banner (if enabled):
       "clawdshell v0.1.0 - launching claude
        Tip: Ctrl+D to drop to /bin/zsh"
  -> Spawn tool as child process with configured args + CLI passthrough args
  -> Wait for tool to exit
  -> Spawn fallback shell as child process
  -> Wait for fallback shell to exit
  -> Exit (terminal closes / logout)
```

### Shell Compatibility

ClawdShell must handle flags that programs like `scp`, `rsync`, and `ssh` pass to login shells:

- `clawdshell -c "command"` -> Forward to fallback shell (critical for ssh remote commands, scp, rsync)
- `clawdshell -l` -> No-op, normal launch (login shell flag)

If `-c` is not handled, remote file operations and ssh commands will break when clawdshell is the login shell.

### Exit Behavior

- Tool exits (any exit code) -> spawn fallback shell
- Fallback shell exits -> clawdshell exits
- Tool binary not found -> warn and skip straight to fallback shell

## Configuration

**Location:** `~/.config/clawdshell/config.toml`

Configuration is optional. If no config file exists, defaults apply: tool=claude, auto-detect fallback shell, banner on.

```toml
[defaults]
tool = "claude"              # Which [tools.*] section to use
fallback_shell = "/bin/zsh"  # Auto-detected and saved during --install
show_banner = true           # Show startup banner

[tools.claude]
command = "claude"           # Binary name or absolute path (defaults to tool name)
args = ["--model", "opus"]   # Default arguments for this tool

[tools.codex]
args = ["--full-auto"]

[tools.gemini]
command = "/usr/local/bin/gemini-cli"
args = []
```

### Resolution Order for Tool Binary

1. `tool_command` field in the active `[tools.*]` section (absolute path, no PATH lookup)
2. `command` field looked up in PATH
3. Tool name (from `[defaults].tool`) looked up in PATH
4. If not found -> warn and skip to fallback shell

### Fallback Shell Detection

On `--install`, clawdshell snapshots the current `$SHELL` value into `config.toml` as `fallback_shell`. If no config exists at runtime, it auto-detects:

- **macOS/Linux:** Check `$SHELL`, fall back to `/bin/sh`
- **Windows:** Check `COMSPEC`, fall back to `powershell.exe`

## CLI Interface

```
clawdshell                     # Normal startup (launch tool -> fallback shell)
clawdshell --install           # Register as login shell
clawdshell --uninstall         # Restore previous shell
clawdshell --set-tool <name>   # Quick switch: sets defaults.tool in config
clawdshell --version           # Print version
clawdshell --help              # Print help (includes full config reference)
clawdshell -- --resume         # Everything after -- is passed to the tool
```

### Help Output

The `--help` output includes full configuration reference and supported tools list, so that LLMs can assist users with configuration when the help text is pasted into a conversation:

```
clawdshell 0.1.0 - a login shell that launches AI coding tools

USAGE:
    clawdshell [OPTIONS] [-- <TOOL_ARGS>...]

OPTIONS:
    --install              Register clawdshell as your login shell
    --uninstall            Restore your previous login shell
    --set-tool <NAME>      Set the default tool (e.g., claude, codex, gemini)
    --version              Print version
    --help                 Print this help

SHELL COMPATIBILITY:
    -c <COMMAND>           Execute command via fallback shell (for scp/rsync/ssh)
    -l                     Login shell flag (default behavior, no-op)

PASSTHROUGH:
    clawdshell -- <ARGS>   Pass arguments directly to the active tool

CONFIG:
    ~/.config/clawdshell/config.toml

    [defaults]
    tool = "claude"            # Which tool to launch
    fallback_shell = "/bin/zsh"  # Shell to drop to after tool exits
    show_banner = true         # Show startup banner

    [tools.claude]
    command = "claude"         # Binary name or path (defaults to tool name)
    args = ["--model", "opus"] # Default arguments for this tool

    [tools.codex]
    args = ["--full-auto"]

SUPPORTED TOOLS:
    claude     Claude Code (https://claude.ai/code)
    codex      OpenAI Codex CLI
    gemini     Google Gemini CLI
    opencode   OpenCode
    aider      Aider
    forge      ForgeCode

    Any binary can be used by adding a [tools.<name>] section.

EXAMPLES:
    clawdshell --install                # Set up as login shell
    clawdshell --set-tool codex         # Switch to Codex
    clawdshell -- --resume              # Launch with --resume passed to tool
```

## Installation & Uninstallation

### `clawdshell --install`

**macOS/Linux:**
1. Detect current shell from `$SHELL`
2. Save it as `fallback_shell` in config.toml
3. Add clawdshell's absolute path to `/etc/shells` (requires sudo)
4. Run `chsh -s /path/to/clawdshell`
5. Detect available tools in PATH and set first found as default
6. Print summary of changes

**Windows:**
1. Detect current shell (PowerShell/cmd)
2. Save as fallback
3. Add a Windows Terminal profile for clawdshell (modify `settings.json`)
4. Optionally set as default profile
5. Print summary

**Safety:** Both platforms print what they're about to do and ask for confirmation before making changes.

### `clawdshell --uninstall`

1. Run `chsh -s <fallback_shell>` (or restore Windows Terminal default)
2. Remove from `/etc/shells`
3. Leave config.toml intact (user's settings, don't delete)

### `curl | sh` Installation

A standalone `install.sh` script supports:

```bash
curl -fsSL https://get-clawd.sh | sh
```

The script:
1. Detects OS and architecture
2. Downloads the correct prebuilt binary from a known URL
3. Places it in `/usr/local/bin/clawdshell` (or `~/.local/bin/`)
4. Runs `clawdshell --install`

#### Companion Animation During Download

While the binary downloads, the install script displays an animated ASCII companion sprite (from Claude Code's buddy system). This provides a delightful loading experience.

**Sprite system:**
- 22 species available: duck, goose, blob, cat, dragon, octopus, owl, penguin, turtle, snail, ghost, axolotl, capybara, cactus, robot, rabbit, mushroom, chonk, fox, frog, bat, jellyfish, panda
- Each sprite is 5 lines tall, 12 characters wide
- 3 animation frames per species (resting, fidget 1, fidget 2)
- 6 eye styles: `·`, `✦`, `×`, `◉`, `@`, `°`
- 8 hat types: none, crown, tophat, propeller, halo, wizard, beanie, tinyduck

**Animation loop** (matching Claude Code's implementation):
- 500ms per tick
- 15-tick idle sequence: `[0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, 2, 0, 0, 0]`
  - `0` = resting frame
  - `1` = fidget frame 1
  - `-1` = blink (eyes replaced with `-`)
  - `2` = fidget frame 2
- Randomly selected species, eyes, and hat on each install

**Terminal adaptation:**
- Wide terminal (>=100 cols): Full 5-line sprite above progress bar
- Narrow terminal (<100 cols): Single-line face mode (e.g., `=·w·=` for cat)

**Example display:**
```
  Downloading clawdshell for aarch64-apple-darwin...

   \^^^/
   /\_/\
  ( ·   ·)
  (  w  )
  (")_(")

  [████████░░░░░░░░] 48%
```

## Project Structure

```
clawdshell/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI parsing (clap), entry point
│   ├── config.rs        # Config loading/saving, defaults, TOML serde
│   ├── install.rs       # --install / --uninstall logic per platform
│   ├── banner.rs        # Banner display
│   ├── shell.rs         # Spawn tool, spawn fallback shell, -c forwarding
│   └── detect.rs        # Detect available tools in PATH, detect current shell
├── install.sh           # curl-pipe-sh installer with companion animation
└── README.md
```

## Dependencies

Minimal:
- `clap` - CLI argument parsing
- `toml` / `serde` - Config file parsing
- `dirs` - Cross-platform config directory (`~/.config/` on Unix, `%APPDATA%` on Windows)
- `which` - Find binaries in PATH

No async runtime. This is a synchronous, short-lived process.

## Platform Considerations

| Concern | macOS/Linux | Windows |
|---------|-------------|---------|
| Shell registration | `/etc/shells` + `chsh` | Windows Terminal `settings.json` profile |
| Process spawning | `std::process::Command` | `std::process::Command` |
| Config directory | `~/.config/clawdshell/` | `%APPDATA%/clawdshell/` |
| Fallback detection | `$SHELL` env var | `COMSPEC` env var |
| `-c` forwarding | Forward to fallback shell | Forward to fallback shell |

## Decisions Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Cross-platform from one codebase, single binary, Windows support needed |
| Architecture | Thin wrapper | Unix philosophy - do one thing well, tool handles the rest |
| Tool coupling | Tool-agnostic | Configurable per-tool args, supports any binary |
| Fallback shell | Configurable with auto-detect | Snapshot on install, user can override, handles edge case where $SHELL becomes clawdshell |
| Banner | On by default, easy disable | Quick orientation without clutter, `show_banner = false` |
| Install script animation | Claude Code companion sprites | Fun loading experience, matches Claude ecosystem aesthetic |
