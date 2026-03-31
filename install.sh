#!/bin/sh
set -e

# ClawdShell installer — downloads the binary, then hands off to it
# Usage: curl -fsSL https://clawdshell.dev | sh

VERSION="${CLAWDSHELL_VERSION:-latest}"
REPO="${CLAWDSHELL_REPO:-user/clawdshell}"

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

main() {
    PLATFORM=$(detect_platform)

    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/clawdshell-${PLATFORM}"
    DEST="${INSTALL_DIR}/clawdshell"

    echo "Downloading clawdshell for ${PLATFORM}..."

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "$DEST" || { echo "Download failed: $DOWNLOAD_URL"; exit 1; }
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "$DEST" || { echo "Download failed."; exit 1; }
    else
        echo "Error: curl or wget required"; exit 1
    fi

    chmod +x "$DEST"

    # Hand off to the binary — it handles everything else with proper TUI
    "$DEST" --install
}

main "$@"
