#!/bin/sh
set -e

# ClawdShell installer
# Usage: curl -fsSL https://get-clawd.sh | sh

VERSION="${CLAWDSHELL_VERSION:-latest}"
REPO="${CLAWDSHELL_REPO:-user/clawdshell}"

# --- Platform Detection ---

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)  OS="linux" ;;
        Darwin*) OS="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *) echo "Unsupported OS: $OS"; exit 1 ;;
    esac

    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac

    echo "${OS}-${ARCH}"
}

# --- Companion Sprites (subset for script size) ---

# Each sprite is stored as a single string with | as line delimiter
# {E} is the eye placeholder
SPRITE_0='   /\_/\|  ( {E}   {E})|  (  w  )|  (")_(")'
SPRITE_1='    __|  <({E} )___|   (  ._>|    `--´'
SPRITE_2='   .[||].|  [ {E}  {E} ]|  [ ==== ]|  `------´'
SPRITE_3='   .----.|  / {E}  {E} \|  |      ||  ~`~``~`~'
SPRITE_4='   ^    ^|  ({E}  w {E})|  ( `--´ )|   ~~~~~~'
SPRITE_5='   .----.|  ( {E}  {E} )|  (      )|   `----´'

FACE_0='={E}w{E}='
FACE_1='({E}>'
FACE_2='[{E}{E}]'
FACE_3='/{E}{E}\'
FACE_4='({E}w{E})'
FACE_5='({E}{E})'

# Eye options
EYES='· ✦ × ◉ @ °'

# --- Random Selection ---

pick_random() {
    if [ -n "${RANDOM+x}" ]; then
        echo $(( RANDOM % $1 ))
    else
        echo $(( $(date +%s) % $1 ))
    fi
}

# Pick sprite and eyes
SPRITE_IDX=$(pick_random 6)
EYE_IDX=$(pick_random 6)
EYE=$(echo "$EYES" | tr ' ' '\n' | sed -n "$((EYE_IDX + 1))p")
[ -z "$EYE" ] && EYE="·"

# Select sprite
eval "SPRITE=\$SPRITE_${SPRITE_IDX}"
eval "FACE=\$FACE_${SPRITE_IDX}"

# Substitute eyes
SPRITE=$(echo "$SPRITE" | sed "s/{E}/$EYE/g")
FACE=$(echo "$FACE" | sed "s/{E}/$EYE/g")

show_sprite() {
    echo "$SPRITE" | tr '|' '\n'
}

# --- Animation ---

# Idle sequence: 0=normal, 1=blink (eyes become -)
# Simplified from the full 15-tick sequence for shell script
BLINK_SPRITE=$(echo "$SPRITE_0" | sed "s/{E}/-/g" | tr '|' '\n')

animate_download() {
    # Show sprite while download happens in background
    local pid=$1
    local tick=0

    while kill -0 "$pid" 2>/dev/null; do
        # Simple animation: mostly show sprite, occasionally blink
        if [ $((tick % 8)) -eq 4 ]; then
            # Blink frame
            printf "\033[4A"  # Move up 4 lines
            eval "BLINK=\$SPRITE_${SPRITE_IDX}"
            BLINK=$(echo "$BLINK" | sed "s/{E}/-/g")
            echo "$BLINK" | tr '|' '\n'
        else
            if [ "$tick" -gt 0 ]; then
                printf "\033[4A"  # Move up 4 lines
            fi
            show_sprite
        fi
        tick=$((tick + 1))
        sleep 0.5
    done
}

# --- Installation ---

main() {
    PLATFORM=$(detect_platform)

    printf "\n"
    printf "  \033[1mInstalling clawdshell\033[0m for %s\n" "$PLATFORM"
    printf "\n"

    # Show initial sprite
    show_sprite
    printf "\n"

    # Determine install directory
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        mkdir -p "$HOME/.local/bin"
        INSTALL_DIR="$HOME/.local/bin"
    fi

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/clawdshell-${PLATFORM}"
    DEST="${INSTALL_DIR}/clawdshell"

    printf "  Downloading from GitHub...\n"

    # Download
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "$DEST" 2>/dev/null || {
            printf "\n  \033[31mDownload failed.\033[0m\n"
            printf "  URL: %s\n" "$DOWNLOAD_URL"
            printf "  Make sure the release exists.\n"
            exit 1
        }
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "$DEST" || {
            printf "\n  \033[31mDownload failed.\033[0m\n"
            exit 1
        }
    else
        printf "  \033[31mError: curl or wget required\033[0m\n"
        exit 1
    fi

    chmod +x "$DEST"

    printf "\n"
    printf "  %s Installed to: %s\n" "$FACE" "$DEST"
    printf "\n"

    # Run --install
    "$DEST" --install
}

main "$@"
