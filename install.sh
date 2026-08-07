#!/usr/bin/env bash
set -e

APP="passconf"
SRC="/mnt/data/Projects/$APP"
TMP="/tmp/$APP-build"
DEST="${DESTDIR:-$HOME/.local}"

echo "==> Building $APP..."
mkdir -p "$TMP"
rsync -a --exclude target "$SRC/" "$TMP/"
cd "$TMP"
cargo build --release

echo "==> Installing to $DEST/bin/$APP..."
mkdir -p "$DEST/bin"
cp "$TMP/target/release/$APP" "$DEST/bin/$APP"
chmod +x "$DEST/bin/$APP"

echo "==> Installing .desktop file..."
mkdir -p "$DEST/share/applications"
cp "$SRC/$APP.desktop" "$DEST/share/applications/"

echo ""
echo "==> Installed! Run with: $DEST/bin/$APP"
echo "    Or add to PATH: export PATH=\"\$PATH:$DEST/bin\""
echo "    Or find in your app launcher as 'PassConf'"
