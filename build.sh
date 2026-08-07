#!/usr/bin/env bash
set -e

APP="passconf"
SRC="/mnt/data/Projects/$APP"
TMP="/tmp/$APP-build"

echo "==> Syncing source to $TMP (executable tmpfs)..."
mkdir -p "$TMP"
rsync -a --exclude target "$SRC/" "$TMP/"

echo "==> Building release binary..."
cd "$TMP"
cargo build --release

echo "==> Copying binary back to project..."
cp "$TMP/target/release/$APP" "$SRC/$APP"

echo "==> Done! Binary at $SRC/$APP"
echo "    Run: $SRC/$APP"
echo ""
echo "    Or copy to your PATH:"
echo "    cp $SRC/$APP ~/.local/bin/"
