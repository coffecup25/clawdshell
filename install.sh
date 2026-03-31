#!/bin/sh
set -e

# ClawdShell installer
# Usage: bash <(curl -fsSL https://clawdshell.dev)

VERSION="${CLAWDSHELL_VERSION:-latest}"
REPO="${CLAWDSHELL_REPO:-coffecup25/clawdshell}"

ORANGE='\033[38;2;217;119;87m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    case "$OS" in
        Linux*)  OS="linux" ;;
        Darwin*) OS="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *) printf "${ORANGE}Unsupported OS: %s${RESET}\n" "$OS"; exit 1 ;;
    esac
    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) printf "${ORANGE}Unsupported architecture: %s${RESET}\n" "$ARCH"; exit 1 ;;
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

    printf "\n"
    printf "  ${BOLD}${ORANGE}clawdshell${RESET} ${DIM}installer${RESET}\n"
    printf "  ${DIM}%s${RESET}\n" "$PLATFORM"
    printf "\n"
    printf "  ${DIM}Downloading...${RESET}"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "$DEST" || {
            printf "\r  ${ORANGE}Download failed.${RESET}                    \n"
            printf "  ${DIM}%s${RESET}\n" "$DOWNLOAD_URL"
            exit 1
        }
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "$DEST" || {
            printf "\r  ${ORANGE}Download failed.${RESET}                    \n"
            exit 1
        }
    else
        printf "\r  ${ORANGE}curl or wget required${RESET}                  \n"
        exit 1
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

    printf "\r  ${ORANGE}✓${RESET} Downloaded to ${DIM}%s${RESET}          \n\n" "$DEST"

    # Run the interactive installer
    "$DEST" --install
}

main "$@"
