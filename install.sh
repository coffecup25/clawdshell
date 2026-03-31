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

animate_while() {
    # Run a command in the background, show companion + progress bar while it runs
    # Usage: animate_while "message" command args...
    local msg="$1"
    shift

    "$@" >/dev/null 2>&1 &
    local pid=$!
    local tick=0

    # Build blink sprite
    local blink_sprite
    eval "blink_sprite=\$SPRITE_${SPRITE_IDX}"
    blink_sprite=$(echo "$blink_sprite" | sed "s/{E}/-/g")

    # Total lines drawn: 1 (message) + 1 (blank) + 4 (sprite) + 1 (bar) = 7
    local draw_height=6

    printf "  ${DIM}%s${RESET}\n\n" "$msg"
    show_sprite
    printf "\n"
    printf "  ${ORANGE}░░░░░░░░░░░░░░░░░░░░${RESET}"  # empty bar

    while kill -0 "$pid" 2>/dev/null; do
        printf "\033[${draw_height}A"  # Move up to sprite start

        # Draw sprite frame
        if [ $((tick % 8)) -eq 4 ]; then
            echo "$blink_sprite" | tr '|' '\n'
        else
            show_sprite
        fi

        # Progress bar (bouncing fill based on tick)
        local bar_pos=$(( tick % 20 ))
        local bar=""
        local i=0
        while [ $i -lt 20 ]; do
            local dist=$(( bar_pos - i ))
            [ $dist -lt 0 ] && dist=$(( -dist ))
            if [ $dist -lt 4 ]; then
                bar="${bar}█"
            else
                bar="${bar}░"
            fi
            i=$((i + 1))
        done
        printf "\n  ${ORANGE}${bar}${RESET}  "

        tick=$((tick + 1))
        sleep 0.3
    done

    # Wait for exit code
    wait "$pid"
    local exit_code=$?

    # Show full bar briefly
    printf "\033[${draw_height}A"
    show_sprite
    printf "\n  ${ORANGE}████████████████████${RESET}  \n"
    sleep 0.3

    # Clear the animation area
    printf "\033[$((draw_height + 1))A"
    local j=0
    while [ $j -le $draw_height ]; do
        printf "\033[2K\n"
        j=$((j + 1))
    done
    printf "\033[$((draw_height + 1))A"

    return $exit_code
}

ensure_claude() {
    if command -v claude >/dev/null 2>&1; then
        CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "unknown")
        printf "  ${GREEN}✓${RESET} Claude Code already installed ${DIM}(%s)${RESET}\n" "$CLAUDE_VERSION"
        return 0
    fi

    ensure_node

    if command -v npm >/dev/null 2>&1; then
        if animate_while "Installing Claude Code..." npm install -g @anthropic-ai/claude-code; then
            printf "  ${GREEN}✓${RESET} Claude Code installed\n"
        else
            printf "  ${RED}✗${RESET} Claude Code installation failed\n"
            printf "  ${DIM}Try manually: npm install -g @anthropic-ai/claude-code${RESET}\n"
        fi
    elif command -v npx >/dev/null 2>&1; then
        printf "  ${DIM}Claude Code will be available via npx${RESET}\n"
        return 0
    else
        printf "  ${RED}npm not found. Install Node.js first.${RESET}\n"
        exit 1
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

    download_clawdshell() {
        if command -v curl >/dev/null 2>&1; then
            curl -fsSL "$DOWNLOAD_URL" -o "$DEST" 2>/dev/null
        elif command -v wget >/dev/null 2>&1; then
            wget -q "$DOWNLOAD_URL" -O "$DEST"
        else
            return 1
        fi
    }

    if animate_while "Downloading clawdshell..." download_clawdshell; then
        chmod +x "$DEST"
        printf "  ${GREEN}✓${RESET} Installed to: %s\n\n" "$DEST"
    else
        printf "  ${RED}Download failed.${RESET}\n"
        printf "  ${DIM}URL: %s${RESET}\n" "$DOWNLOAD_URL"
        exit 1
    fi

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
