#!/bin/sh
set -e

# ClawdShell installer
# Usage: curl -fsSL https://clawdshell.dev | sh

VERSION="${CLAWDSHELL_VERSION:-latest}"
REPO="${CLAWDSHELL_REPO:-user/clawdshell}"

# Colors
BOLD='\033[1m'
DIM='\033[2m'
ORANGE='\033[38;2;217;119;87m'
GREEN='\033[32m'
RED='\033[31m'
RESET='\033[0m'

# --- Platform Detection ---

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)  OS="linux" ;;
        Darwin*) OS="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *) printf "  ${RED}Unsupported OS: %s${RESET}\n" "$OS"; exit 1 ;;
    esac

    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) printf "  ${RED}Unsupported architecture: %s${RESET}\n" "$ARCH"; exit 1 ;;
    esac

    echo "${OS}-${ARCH}"
}

# --- Companion Sprites ---

SPRITE_0='   /\_/\|  ( {E}   {E})|  (  w  )|  (")_(")'
SPRITE_1='    __|  <({E} )___|   (  ._>|    `--´'
SPRITE_2='   .[||].|  [ {E}  {E} ]|  [ ==== ]|  `------´'
SPRITE_3='   .----.|  / {E}  {E} \|  |      ||  ~`~``~`~'
SPRITE_4='   ^    ^|  ({E}  w {E})|  ( `--´ )|   ~~~~~~'
SPRITE_5='   .----.|  ( {E}  {E} )|  (      )|   `----´'

EYES='· ✦ × ◉ @ °'

pick_random() {
    if [ -n "${RANDOM+x}" ]; then
        echo $(( RANDOM % $1 ))
    else
        echo $(( $(date +%s) % $1 ))
    fi
}

SPRITE_IDX=$(pick_random 6)
EYE_IDX=$(pick_random 6)
EYE=$(echo "$EYES" | tr ' ' '\n' | sed -n "$((EYE_IDX + 1))p")
[ -z "$EYE" ] && EYE="·"

eval "SPRITE=\$SPRITE_${SPRITE_IDX}"
SPRITE=$(echo "$SPRITE" | sed "s/{E}/$EYE/g")

show_sprite() {
    echo "$SPRITE" | tr '|' '\n'
}

# --- Dependency Checks ---

ensure_node() {
    if command -v node >/dev/null 2>&1; then
        return 0
    fi

    printf "  ${DIM}Node.js not found. Installing...${RESET}\n"

    # Try common node version managers first
    if command -v nvm >/dev/null 2>&1; then
        nvm install --lts
        return $?
    fi

    if command -v fnm >/dev/null 2>&1; then
        fnm install --lts
        fnm use lts-latest
        return $?
    fi

    # Try package managers
    if [ "$(uname -s)" = "Darwin" ]; then
        if command -v brew >/dev/null 2>&1; then
            brew install node
            return $?
        fi
    elif [ "$(uname -s)" = "Linux" ]; then
        if command -v apt-get >/dev/null 2>&1; then
            sudo apt-get update -qq && sudo apt-get install -y -qq nodejs npm
            return $?
        elif command -v dnf >/dev/null 2>&1; then
            sudo dnf install -y nodejs npm
            return $?
        elif command -v pacman >/dev/null 2>&1; then
            sudo pacman -S --noconfirm nodejs npm
            return $?
        fi
    fi

    printf "  ${RED}Could not install Node.js automatically.${RESET}\n"
    printf "  Please install Node.js from https://nodejs.org and try again.\n"
    exit 1
}

ensure_claude() {
    if command -v claude >/dev/null 2>&1; then
        CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "unknown")
        printf "  ${GREEN}✓${RESET} Claude Code already installed ${DIM}(%s)${RESET}\n" "$CLAUDE_VERSION"
        return 0
    fi

    printf "  ${DIM}Claude Code not found. Installing...${RESET}\n"

    ensure_node

    if command -v npm >/dev/null 2>&1; then
        npm install -g @anthropic-ai/claude-code 2>&1 | tail -1
    elif command -v npx >/dev/null 2>&1; then
        # npx will download and run it on first use
        printf "  ${DIM}Claude Code will be available via npx${RESET}\n"
        return 0
    else
        printf "  ${RED}npm not found. Install Node.js first.${RESET}\n"
        exit 1
    fi

    if command -v claude >/dev/null 2>&1; then
        printf "  ${GREEN}✓${RESET} Claude Code installed\n"
    else
        printf "  ${RED}✗${RESET} Claude Code installation may have failed\n"
        printf "  ${DIM}Try: npm install -g @anthropic-ai/claude-code${RESET}\n"
    fi
}

# --- Installation ---

main() {
    PLATFORM=$(detect_platform)

    printf "\n"
    show_sprite
    printf "\n"
    printf "  ${BOLD}${ORANGE}CLAWDSHELL${RESET} installer\n"
    printf "  ${DIM}you weren't using your terminal anyways${RESET}\n"
    printf "\n"
    printf "  ${DIM}Platform:${RESET} %s\n" "$PLATFORM"
    printf "\n"

    # --- Step 1: Install clawdshell binary ---
    printf "  ${BOLD}${ORANGE}Step 1:${RESET} Installing clawdshell binary\n\n"

    # Determine install directory
    INSTALL_DIR=""
    if [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        mkdir -p "$HOME/.local/bin"
        INSTALL_DIR="$HOME/.local/bin"
    fi

    # Check if ~/.local/bin is in PATH, if not suggest adding it
    case ":$PATH:" in
        *":$HOME/.local/bin:"*) ;;
        *)
            printf "  ${DIM}Note: %s is not in your PATH${RESET}\n" "$HOME/.local/bin"
            printf "  ${DIM}Add it: export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}\n\n"
            ;;
    esac

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/clawdshell-${PLATFORM}"
    DEST="${INSTALL_DIR}/clawdshell"

    printf "  ${DIM}Downloading...${RESET}\n"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "$DEST" 2>/dev/null || {
            printf "  ${RED}Download failed.${RESET}\n"
            printf "  ${DIM}URL: %s${RESET}\n" "$DOWNLOAD_URL"
            printf "  ${DIM}Make sure the release exists.${RESET}\n"
            exit 1
        }
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "$DEST" || {
            printf "  ${RED}Download failed.${RESET}\n"
            exit 1
        }
    else
        printf "  ${RED}Error: curl or wget required${RESET}\n"
        exit 1
    fi

    chmod +x "$DEST"
    printf "  ${GREEN}✓${RESET} Installed to: %s\n\n" "$DEST"

    # --- Step 2: Ensure Claude Code is installed ---
    printf "  ${BOLD}${ORANGE}Step 2:${RESET} Checking for AI coding tools\n\n"

    ensure_claude

    # Check for other tools too
    for tool in codex gemini aider opencode forge; do
        if command -v "$tool" >/dev/null 2>&1; then
            printf "  ${GREEN}✓${RESET} %s found\n" "$tool"
        fi
    done

    printf "\n"

    # --- Step 3: Run clawdshell --install ---
    printf "  ${BOLD}${ORANGE}Step 3:${RESET} Setting up clawdshell as your login shell\n\n"

    "$DEST" --install
}

main "$@"
