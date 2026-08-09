#!/bin/bash
set -e

TARBALL=$(ls clashpass-*-linux-amd64.tar.gz 2>/dev/null | head -1)

if [ -z "$TARBALL" ]; then
    echo "Error: No clashpass tarball found in current directory."
    echo "Download from: https://github.com/TharukRenuja/ClashPass/releases/latest"
    exit 1
fi

echo "Extracting $TARBALL..."
tar xzf "$TARBALL"

DIR=$(tar tzf "$TARBALL" | head -1 | cut -d'/' -f1)
echo "Installing..."
cd "$DIR"
./clashpass

echo ""
echo "Done! ClashPass installed. You can now launch it from your app launcher."
