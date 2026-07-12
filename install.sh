#!/bin/sh
# p2ptokens installer for macOS and Linux
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/p2ptokens/p2ptokens/main/install.sh | sh
#   ./install.sh
#
# Environment variables:
#   P2PTOKENS_REPO   GitHub repo slug to download releases from.
#                    Defaults to "p2ptokens/p2ptokens". Override this if the
#                    real repository lives at a different owner/name, e.g.:
#                      P2PTOKENS_REPO=myorg/p2ptokens ./install.sh
#   P2PTOKENS_BIN    Directory to install the binaries into.
#                    Defaults to "$HOME/.local/bin".
#
# Installs two binaries: p2ptokens (the client) and p2p-coordinator.

set -eu

REPO="${P2PTOKENS_REPO:-p2ptokens/p2ptokens}"
INSTALL_DIR="${P2PTOKENS_BIN:-$HOME/.local/bin}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
info() { printf '%s\n' "$*"; }
err() { printf 'error: %s\n' "$*" >&2; }
die() { err "$*"; exit 1; }

# ---------------------------------------------------------------------------
# Detect OS
# ---------------------------------------------------------------------------
os_raw="$(uname -s)"
case "$os_raw" in
    Darwin) OS="macos" ;;
    Linux)  OS="linux" ;;
    *) die "unsupported operating system: '$os_raw' (only macOS and Linux are supported). Please build from source." ;;
esac

# ---------------------------------------------------------------------------
# Detect architecture
# ---------------------------------------------------------------------------
arch_raw="$(uname -m)"
case "$arch_raw" in
    arm64|aarch64) ARCH="arm64" ;;
    x86_64|amd64)  ARCH="x64" ;;
    *) die "unsupported architecture: '$arch_raw'. Please build from source." ;;
esac

# ---------------------------------------------------------------------------
# Choose the matching asset name
# ---------------------------------------------------------------------------
# Linux only ships an x64 build.
if [ "$OS" = "linux" ] && [ "$ARCH" = "arm64" ]; then
    die "no prebuilt Linux arm64 binary is available. Please build from source."
fi

ASSET="p2ptokens-${OS}-${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

# ---------------------------------------------------------------------------
# Temp dir with cleanup trap
# ---------------------------------------------------------------------------
TMPDIR_P2P="$(mktemp -d 2>/dev/null || mktemp -d -t p2ptokens)"
cleanup() { rm -rf "$TMPDIR_P2P"; }
trap cleanup EXIT INT TERM

ARCHIVE="${TMPDIR_P2P}/${ASSET}"

# ---------------------------------------------------------------------------
# Download (curl, fall back to wget)
# ---------------------------------------------------------------------------
info "Installing p2ptokens"
info "  repo:    ${REPO}"
info "  os/arch: ${OS}/${ARCH}"
info "  asset:   ${ASSET}"
info ""
info "Downloading ${URL} ..."

if command -v curl >/dev/null 2>&1; then
    curl -fSL --progress-bar "$URL" -o "$ARCHIVE" \
        || die "download failed. Check that a release exists at https://github.com/${REPO}/releases/latest"
elif command -v wget >/dev/null 2>&1; then
    wget -q --show-progress -O "$ARCHIVE" "$URL" \
        || die "download failed. Check that a release exists at https://github.com/${REPO}/releases/latest"
else
    die "neither curl nor wget is installed. Please install one and retry."
fi

# ---------------------------------------------------------------------------
# Extract
# ---------------------------------------------------------------------------
info "Extracting ..."
EXTRACT_DIR="${TMPDIR_P2P}/extract"
mkdir -p "$EXTRACT_DIR"
tar -xzf "$ARCHIVE" -C "$EXTRACT_DIR" || die "failed to extract archive."

# ---------------------------------------------------------------------------
# Install
# ---------------------------------------------------------------------------
mkdir -p "$INSTALL_DIR" || die "could not create install dir: $INSTALL_DIR"

for bin in p2ptokens p2p-coordinator; do
    src="${EXTRACT_DIR}/${bin}"
    [ -f "$src" ] || die "expected binary '${bin}' not found in archive."
    cp "$src" "${INSTALL_DIR}/${bin}" || die "failed to copy ${bin} to ${INSTALL_DIR}"
    chmod +x "${INSTALL_DIR}/${bin}" || die "failed to chmod ${bin}"
    info "Installed ${bin} -> ${INSTALL_DIR}/${bin}"
done

info ""
info "Success! p2ptokens and p2p-coordinator were installed to ${INSTALL_DIR}"

# ---------------------------------------------------------------------------
# PATH check
# ---------------------------------------------------------------------------
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
        info "You can now run: p2ptokens"
        ;;
    *)
        info ""
        info "WARNING: ${INSTALL_DIR} is not on your PATH."
        info "Add it by appending this line to your shell profile"
        info "(e.g. ~/.profile, ~/.bashrc, or ~/.zshrc):"
        info ""
        info "    export PATH=\"${INSTALL_DIR}:\$PATH\""
        info ""
        info "Then restart your shell or run: export PATH=\"${INSTALL_DIR}:\$PATH\""
        ;;
esac
