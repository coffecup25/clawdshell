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
  -> Show companion greeting (if enabled):
       [companion sprite]  clawdshell v0.1.0
                           launching claude...
                           Ctrl+D to drop to /bin/zsh
  -> Spawn tool as child process with configured args + CLI passthrough args
  -> Wait for tool to exit
  -> Spawn fallback shell as child process
  -> Wait for fallback shell to exit
  -> Exit (terminal closes / logout)
```

### Shell Compatibility

ClawdShell must handle flags that programs like `scp`, `rsync`, and `ssh` pass to login shells:

- `clawdshell -c "command"` -> Forward to fallback shell (critical for ssh remote commands, scp, rsync)
- `clawdshell -c "command" arg1 arg2` -> Forward to fallback shell with positional args (POSIX-compliant)
- `clawdshell -l` -> No-op, normal launch (login shell flag)

Any unrecognized flags are passed through to the fallback shell. If `-c` is not handled, remote file operations and ssh commands will break when clawdshell is the login shell.

### Signal Handling

On Unix, clawdshell forwards signals to the active child process:

- **SIGINT (Ctrl+C):** Forward to child. Do not exit clawdshell itself.
- **SIGHUP (terminal close):** Forward to child, then exit.
- **SIGWINCH (terminal resize):** Forward to child (tools need this for UI reflow).
- **SIGTSTP (Ctrl+Z):** Forward to child. The tool decides whether to suspend.

On Windows, signals are handled by the OS/console subsystem and do not need explicit forwarding.

### Non-Interactive / Non-TTY Mode

If stdin is not a TTY (e.g., piped input like `echo "ls" | clawdshell`), skip the tool and forward directly to the fallback shell. Interactive AI tools will fail or hang without a TTY.

### Exit Behavior

- Tool exits (any exit code) -> spawn fallback shell
- Fallback shell exits -> clawdshell exits with the fallback shell's exit code
- Tool binary not found -> warn and skip straight to fallback shell

On Unix, the fallback shell is launched via `exec` (replaces the clawdshell process) to avoid an idle parent process. On Windows, spawn-and-wait is used since `exec` semantics differ.

## Configuration

**Location:** `~/.config/clawdshell/config.toml` (override with `CLAWDSHELL_CONFIG` env var)

Configuration is optional. If no config file exists, defaults apply: tool=claude, auto-detect fallback shell, companion on. Set `CLAWDSHELL_DEBUG=1` for diagnostic output to stderr.

```toml
[defaults]
tool = "claude"              # Which [tools.*] section to use
fallback_shell = "/bin/zsh"  # Auto-detected and saved during --install

[companion]
enabled = true               # Show companion on startup, transitions, and errors
seed = "a7f2b3"             # Auto-generated on first launch, determines your companion

[tools.claude]
command = "claude"           # Binary name or absolute path (defaults to tool name)
args = ["--model", "opus"]   # Default arguments for this tool

[tools.codex]
args = ["--full-auto"]

[tools.gemini]
command = "/usr/local/bin/gemini-cli"
args = []
```

## Companion System

Each clawdshell installation has a persistent companion — a randomly generated ASCII creature that appears throughout the shell lifecycle. The companion is determined by a seed auto-generated on first launch and stored in config, so you always get the same one.

### Companion Generation

On first launch (no `[companion]` section in config):
1. Generate a random seed
2. Derive species, eyes, hat, and rarity from the seed (same weighted system as Claude Code: common 60%, uncommon 25%, rare 10%, epic 4%, legendary 1%)
3. Save seed to config
4. Play a brief "hatching" animation

### Where the Companion Appears

**Startup greeting** (replaces traditional banner):
```
   /\_/\
  ( ·   ·)   clawdshell v0.1.0
  (  ω  )    launching claude...
  (")_(")    Ctrl+D to drop to /bin/zsh
```

**Fallback shell transition** (tool exits):
```
   /\_/\
  ( ·   ·)   dropping to /bin/zsh
  (  ω  )~   type 'claude' to come back
  (")_(")
```

**Error states** (tool not found):
```
   /\_/\
  ( ×   ×)   'codex' not found in PATH
  (  ω  )    falling back to /bin/zsh
  (")_(")
```

**First launch** — hatching animation when the companion is generated for the first time.

**Stats card** via `clawdshell --companion`:
```
  ╭─────────────────────╮
  │   /\_/\             │
  │  ( ·   ·)    Mochi  │
  │  (  ω  )     ★★★   │
  │  (")_(")     rare   │
  ├─────────────────────┤
  │ DEBUGGING  ████░ 8  │
  │ PATIENCE   ██░░░ 4  │
  │ CHAOS      █████ 10 │
  │ WISDOM     ███░░ 6  │
  │ SNARK      ████░ 7  │
  ╰─────────────────────╯
```

### Animation

Same system as Claude Code:
- 500ms per tick
- 15-tick idle sequence: `[0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, 2, 0, 0, 0]`
- Blink: eyes replaced with `-`
- Used during startup greeting, hatching, and install script download

### Companion in Narrow Terminals

When terminal width < 100 cols, collapse to single-line face mode:
```
=·ω·= launching claude... (Ctrl+D for /bin/zsh)
```

### Disabling

Set `companion.enabled = false` in config. When disabled, no companion output is shown — the tool launches silently.

### Resolution Order for Tool Binary

1. `command` field in the active `[tools.*]` section. If it's an absolute path, use directly; if it's a bare name, look up in PATH.
2. Tool name (from `[defaults].tool`) looked up in PATH (used when no `command` field is set).
3. If not found -> warn and skip to fallback shell.

### Fallback Shell Detection

On `--install`, clawdshell snapshots the current `$SHELL` value into `config.toml` as `fallback_shell`. If no config exists at runtime, it auto-detects:

- **macOS/Linux:** Check `$SHELL`, fall back to `/bin/sh`
- **Windows:** Check `COMSPEC`, fall back to `powershell.exe`

### Environment / Profile Sourcing

ClawdShell does not source shell profile files (`~/.zprofile`, `~/.bash_profile`, etc.) — it is a Rust binary, not a POSIX shell. It relies on the terminal emulator inheriting an environment where PATH is already set (which is the case for macOS Terminal.app, iTerm2, Windows Terminal, and most Linux terminal emulators). Users whose tool binaries require PATH entries from login profiles should set those in `~/.zshenv` (sourced by all zsh instances) or equivalent.

## CLI Interface

```
clawdshell                     # Normal startup (launch tool -> fallback shell)
clawdshell --install           # Register as login shell
clawdshell --uninstall         # Restore previous shell
clawdshell --set-tool <name>   # Quick switch: sets defaults.tool in config
clawdshell --companion         # Show your companion's stats card
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
    --companion            Show your companion's stats card
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

    [companion]
    enabled = true             # Show companion on startup/transitions/errors
    seed = "a7f2b3"           # Auto-generated, determines your companion

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

ENVIRONMENT:
    CLAWDSHELL_CONFIG    Override config file path
    CLAWDSHELL_DEBUG=1   Print diagnostic info to stderr

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
3. Locate Windows Terminal `settings.json` (check Store path `%LOCALAPPDATA%\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json`, then unpackaged/preview paths)
4. Add a Windows Terminal profile for clawdshell
5. Optionally set as default profile (record whether this was done, so uninstall can restore only if changed)
6. If Windows Terminal not found, print manual setup instructions for the user's terminal
7. Print summary

**Safety:** Both platforms print what they're about to do and ask for confirmation before any sudo calls or system modifications.

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
3. Prompts for confirmation before proceeding with installation
4. Places it in `/usr/local/bin/clawdshell` (requires sudo) or `~/.local/bin/` if no sudo
5. Runs `clawdshell --install` (which will request sudo for `/etc/shells` and `chsh`)

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
│   ├── companion.rs     # Companion generation, rendering, animation, stats card
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
| Signal forwarding | Explicit (SIGINT, SIGHUP, SIGWINCH, SIGTSTP) | Handled by OS/console subsystem |
| Fallback shell exec | `exec` replaces process | Spawn-and-wait |

## Decisions Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Cross-platform from one codebase, single binary, Windows support needed |
| Architecture | Thin wrapper | Unix philosophy - do one thing well, tool handles the rest |
| Tool coupling | Tool-agnostic | Configurable per-tool args, supports any binary |
| Fallback shell | Configurable with auto-detect | Snapshot on install, user can override, handles edge case where $SHELL becomes clawdshell |
| Companion | Persistent per-install, appears at all transitions | Replaces traditional banner with personality; disabled via `companion.enabled = false` |
| Install script animation | Claude Code companion sprites | Fun loading experience, matches Claude ecosystem aesthetic |
