<p align="center">
  <br>
  <img src="https://img.shields.io/github/v/release/coffecup25/clawdshell?color=D97757&label=version&style=flat-square" alt="version">
  <img src="https://img.shields.io/github/actions/workflow/status/coffecup25/clawdshell/release.yml?style=flat-square&color=D97757" alt="build">
  <img src="https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-D97757?style=flat-square" alt="platforms">
  <img src="https://img.shields.io/github/license/coffecup25/clawdshell?style=flat-square&color=D97757" alt="license">
</p>

<h1 align="center">clawdshell</h1>

<p align="center">
  <i>Lets be real, you weren't using the terminal anyway.</i>
</p>

<p align="center">
  A login shell that launches AI coding tools.<br>
  Open a terminal. Land in Claude Code. Or Codex. Or Gemini. Or Aider.<br>
  Your shell, your choice.
</p>

---

## Install

<table>
<tr>
<td><b>macOS / Linux</b></td>
<td>

```bash
curl -fsSL https://clawdshell.sh | sh
```

</td>
</tr>
<tr>
<td><b>Windows (PowerShell)</b></td>
<td>

```powershell
irm https://clawdshell.sh | iex
```

</td>
</tr>
<tr>
<td><b>Windows (CMD)</b></td>
<td>

```cmd
curl -fsSL https://clawdshell.sh/install.cmd -o install.cmd && install.cmd && del install.cmd
```

</td>
</tr>
</table>

Or grab a binary from [Releases](https://github.com/coffecup25/clawdshell/releases) and run `clawdshell --install`.

## How it works

```
Terminal opens
  → ClawdShell launches your AI tool (claude, codex, gemini, ...)
  → When the tool exits, you drop to your regular shell
  → When that shell exits, the terminal closes
```

Zero friction. Zero delay. Your AI tool is always one terminal-open away.

## Your companion

Every installation hatches a unique ASCII companion from an egg.

```
  ╭─────────────────────────╮
  │   /\_/\       Mochi     │
  │  ( ·   ·)      ★★★     │
  │  (  ω  )       rare     │
  │  (")_(")                │
  ├─────────────────────────┤
  │ DEBUGGING  ████░ 8      │
  │ PATIENCE   ██░░░ 4      │
  │ CHAOS      █████ 10     │
  │ WISDOM     ███░░ 6      │
  │ SNARK      ████░ 7      │
  ╰─────────────────────────╯
```

See yours with `clawdshell --companion`. Reroll during setup if you want a different one.

## Configuration

Config lives at `~/.config/clawdshell/config.toml`:

```toml
[defaults]
tool = "claude"              # which tool to launch
fallback_shell = "/bin/zsh"  # shell to drop to after tool exits

[companion]
enabled = true
seed = "a7f2b3"             # determines your companion

[tools.claude]
args = ["--model", "opus"]

[tools.codex]
args = ["--full-auto"]

[tools.gemini]
command = "/usr/local/bin/gemini-cli"
```

## Supported tools

| Tool | Binary | Website |
|------|--------|---------|
| Claude Code | `claude` | [claude.ai/code](https://claude.ai/code) |
| Codex CLI | `codex` | [openai.com](https://openai.com) |
| Gemini CLI | `gemini` | [cloud.google.com](https://cloud.google.com) |
| OpenCode | `opencode` | [opencode.ai](https://opencode.ai) |
| Aider | `aider` | [aider.chat](https://aider.chat) |
| ForgeCode | `forge` | [forgecode.dev](https://forgecode.dev) |

Any binary works — just add a `[tools.<name>]` section to your config.

## Commands

```
clawdshell                     # launch (normal startup)
clawdshell --install           # setup wizard
clawdshell --uninstall         # restore previous shell
clawdshell --set-tool <name>   # switch default tool
clawdshell --companion         # show your companion
clawdshell -- --resume         # pass args to the tool
clawdshell -c "command"        # run command via fallback shell (ssh/scp compat)
```

## Uninstall

```bash
clawdshell --uninstall
```

Restores your previous shell. Config file is preserved.

## License

MIT

---

<p align="center">
  <sub>Built with Rust. Powered by your favorite AI.</sub>
</p>
