#!/bin/sh
# lazygitrs installer — downloads the prebuilt binary from this repo's
# GitHub releases and installs it to ~/.local/bin (or $INSTALL_DIR).
#
#   curl --proto '=https' --tlsv1.2 -LsSf \
#     https://github.com/tmih06/lazygitrs/releases/latest/download/lazygitrs-installer.sh | sh
#
# Optional version arg (default: latest):
#   ... | sh -s -- v0.0.38
set -eu

REPO="tmih06/lazygitrs"
BINARY_NAME="lazygitrs"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${1:-latest}"

# --- platform detection -----------------------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  os="linux" ;;
    Darwin) os="macos" ;;
    *)      echo "error: unsupported OS: $OS" >&2; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)   arch="x86_64" ;;
    aarch64|arm64)  arch="aarch64" ;;
    *)              echo "error: unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

asset="${BINARY_NAME}-${os}-${arch}"

if [ -n "${LAZYGITRS_BASE_URL:-}" ]; then
    # Test/CI override: point the installer at a local mirror of the release
    # assets instead of GitHub.
    base="$LAZYGITRS_BASE_URL"
elif [ "$VERSION" = "latest" ]; then
    base="https://github.com/${REPO}/releases/latest/download"
else
    # Accept both "0.0.38" and "v0.0.38".
    case "$VERSION" in v*) tag="$VERSION" ;; *) tag="v$VERSION" ;; esac
    base="https://github.com/${REPO}/releases/download/${tag}"
fi

# --- download ---------------------------------------------------------------
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "→ Downloading ${asset} (${VERSION})…"
# Pin https for the real release; the CI override may point at a local http mirror.
case "$base" in https://*) proto="--proto" ; opt="=https" ;; *) proto="" ; opt="" ;; esac
# shellcheck disable=SC2086
curl $proto $opt --tlsv1.2 -fsSL "$base/$asset" -o "$tmp/$BINARY_NAME"
# shellcheck disable=SC2086
curl $proto $opt --tlsv1.2 -fsSL "$base/checksums.txt" -o "$tmp/checksums.txt" 2>/dev/null || true
# --- verify checksum (best-effort: skipped if asset has no checksum) --------
if [ -s "$tmp/checksums.txt" ]; then
    expected="$(grep " ${asset}\$" "$tmp/checksums.txt" | awk '{print $1}' || true)"
    if [ -n "$expected" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            actual="$(sha256sum "$tmp/$BINARY_NAME" | awk '{print $1}')"
        elif command -v shasum >/dev/null 2>&1; then
            actual="$(shasum -a 256 "$tmp/$BINARY_NAME" | awk '{print $1}')"
        else
            actual=""
        fi
        if [ -n "$actual" ] && [ "$actual" != "$expected" ]; then
            echo "error: checksum mismatch for $asset" >&2
            echo "  expected: $expected" >&2
            echo "  actual:   $actual" >&2
            exit 1
        fi
        [ -n "$actual" ] && echo "→ Checksum verified."
    fi
fi

# --- install ----------------------------------------------------------------
mkdir -p "$INSTALL_DIR"
install -m 0755 "$tmp/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

echo "✓ lazygitrs installed to $INSTALL_DIR/$BINARY_NAME"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo "(add that line to ~/.bashrc, ~/.zshrc, or your shell's rc file)"
        ;;
esac

echo ""
echo "Run: lazygitrs"
