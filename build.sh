#!/usr/bin/env bash
set -e

APP="clashpass"
SRC="$(cd "$(dirname "$0")" && pwd)"
TMP="/tmp/$APP-build"

echo "==> Syncing source to $TMP (executable tmpfs)..."
mkdir -p "$TMP"
rsync -a --exclude target "$SRC/" "$TMP/"

echo "==> Building release binary..."
cd "$TMP"
cargo build --release

echo "==> Stripping .eh_frame sections..."
objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr "$TMP/target/release/$APP" "$SRC/$APP" 2>/dev/null || cp "$TMP/target/release/$APP" "$SRC/$APP"

echo "==> Installing desktop integration..."
mkdir -p "$HOME/.local/share/applications"
mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"
cp "$SRC/clashpass.desktop" "$HOME/.local/share/applications/"
cp "$SRC/icons/clashpass_256.png" "$HOME/.local/share/icons/hicolor/256x256/apps/clashpass.png"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

echo "==> Done! Binary at $SRC/$APP"
echo "    Run: $SRC/$APP"
echo ""
echo "    Or install to your PATH:"
echo "    cp $SRC/$APP ~/.local/bin/"
