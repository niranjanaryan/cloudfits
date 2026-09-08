#!/usr/bin/env bash
# cloudfits installer — builds from source or installs prebuilt from GitHub Releases.
set -euo pipefail

VERSION="${VERSION:-latest}"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"

usage() {
  echo "Usage: VERSION=<tag|latest> PREFIX=~/.local ./install.sh [--bin <name>]"
  exit 0
}

[ "${1:-}" = "--help" ] && usage

mkdir -p "$BIN_DIR"

fatal() { echo "error: $*" >&2; exit 1; }

if command -v cargo >/dev/null 2>&1; then
  echo "[cloudfits] building from source..."
  cargo build --release
  for bin in cloudfits cloudfits-tui; do
    cp "target/release/$bin" "$BIN_DIR/$bin"
    echo "[cloudfits] installed $bin -> $BIN_DIR/$bin"
  done
  exit 0
fi

REPO="niranjanaryan/cloudfits"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
TAG="${VERSION:-latest}"

if [ "$TAG" = "latest" ]; then
  URL="https://github.com/$REPO/releases/latest/download"
else
  URL="https://github.com/$REPO/releases/download/$TAG"
fi

for bin in cloudfits cloudfits-tui; do
  NAME="${bin}-${OS}-${ARCH}"
  echo "[cloudfits] downloading $NAME ..."
  curl -fsSL "$URL/$NAME" -o "$BIN_DIR/$bin" || fatal "download failed for $NAME"
  chmod +x "$BIN_DIR/$bin"
  echo "[cloudfits] installed $bin -> $BIN_DIR/$bin"
done

echo
echo "cloudfits installed. Add $BIN_DIR to your PATH."
