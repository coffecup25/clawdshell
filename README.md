# clawdshell

> You weren't using your terminal anyways.

A login shell that launches AI coding tools. Open a terminal, land in Claude Code (or Codex, Gemini, Aider, etc.) instead of bash.

## Install

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/coffecup25/clawdshell/main/install.sh)
```

Or download a binary from [Releases](https://github.com/coffecup25/clawdshell/releases) and run:

```bash
clawdshell --install
```

## How it works

1. You open a terminal
2. ClawdShell shows your companion and launches your AI tool
3. When the tool exits (Ctrl+D), you drop to your regular shell
4. When that shell exits, the terminal closes

## Configuration

Config lives at `~/.config/clawdshell/config.toml`:

```toml
[defaults]
tool = "claude"
fallback_shell = "/bin/zsh"

[companion]
enabled = true
seed = "a7f2b3"    # auto-generated, determines your companion

[tools.claude]
args = ["--model", "opus"]

[tools.codex]
args = ["--full-auto"]

[tools.gemini]
command = "/usr/local/bin/gemini-cli"
```

## Switching tools

```bash
clawdshell --set-tool codex
```

## Your companion

Every clawdshell installation gets a unique ASCII companion. See yours:

```bash
clawdshell --companion
```

## Supported tools

| Tool | Binary |
|------|--------|
| Claude Code | `claude` |
| Codex CLI | `codex` |
| Gemini CLI | `gemini` |
| OpenCode | `opencode` |
| Aider | `aider` |
| ForgeCode | `forge` |

Any binary works — just add a `[tools.<name>]` section.

## Uninstall

```bash
clawdshell --uninstall
```

## License

MIT
