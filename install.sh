#!/bin/sh
set -e

# ClawdShell installer
# Usage: curl -fsSL https://clawdshell.dev | bash

VERSION="${CLAWDSHELL_VERSION:-latest}"
REPO="${CLAWDSHELL_REPO:-coffecup25/clawdshell}"

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

    BINARY_NAME="clawdshell-${PLATFORM}"
    if [ "$VERSION" = "latest" ]; then
        DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"
    else
        DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}"
    fi
    DEST="${INSTALL_DIR}/clawdshell"

    echo "Downloading clawdshell for ${PLATFORM}..."

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "$DEST" || {
            echo "Download failed: $DOWNLOAD_URL"
            echo "Make sure a release exists at https://github.com/${REPO}/releases"
            exit 1
        }
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "$DEST" || { echo "Download failed."; exit 1; }
    else
        echo "Error: curl or wget required"; exit 1
    fi

    chmod +x "$DEST"

    # macOS: clear xattrs and ad-hoc sign to bypass Gatekeeper
    if [ "$(uname -s)" = "Darwin" ]; then
        xattr -c "$DEST" 2>/dev/null || true
        codesign --remove-signature "$DEST" 2>/dev/null || true
        codesign --force --sign - "$DEST" 2>/dev/null || true
    fi

    # Ensure ~/.local/bin is in PATH
    case ":$PATH:" in
        *":$HOME/.local/bin:"*) ;;
        *) export PATH="$HOME/.local/bin:$PATH" ;;
    esac

    echo ""
    echo "  Installed clawdshell to $DEST"
    echo ""

    # If we have a real terminal, run --install directly with the TUI
    if [ -t 0 ] && [ -t 1 ]; then
        "$DEST" --install
    else
        # Piped context (curl | bash) — can't run interactive TUI
        echo "  Run the setup wizard:"
        echo ""
        echo "    clawdshell --install"
        echo ""
    fi
}

main "$@"
