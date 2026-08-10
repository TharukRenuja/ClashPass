#!/bin/bash
set -e

REPO="TharukRenuja/ClashPass"
BIN="clashpass"
DESKTOP="clashpass.desktop"
APP_NAME="ClashPass"
EXEC_PATH="/usr/local/bin/$BIN"
ICON_BASE="/usr/share/icons/hicolor"
APPS_DIR="/usr/share/applications"

detect_arch() {
    case "$(uname -m)" in
        x86_64)  echo "x86" ;;
        aarch64) echo "arm" ;;
        *)       echo "unsupported" ;;
    esac
}

ARCH=$(detect_arch)
if [ "$ARCH" = "unsupported" ]; then
    echo "Error: Unsupported architecture $(uname -m)"
    exit 1
fi

echo "Detected architecture: $ARCH"

LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$LATEST" ]; then
    echo "Error: Could not fetch latest release tag."
    exit 1
fi

TARBALL="clashpass-${LATEST}-${ARCH}-linux.tar.gz"
URL="https://github.com/$REPO/releases/download/${LATEST}/${TARBALL}"

echo "Downloading $TARBALL..."
TMPDIR=$(mktemp -d)
curl -fSL "$URL" -o "$TMPDIR/$TARBALL"

echo "Extracting..."
tar xzf "$TMPDIR/$TARBALL" -C "$TMPDIR"

echo "Installing (sudo required)..."
sudo cp "$TMPDIR/$BIN" "$EXEC_PATH"
sudo chmod +x "$EXEC_PATH"
echo "  Binary -> $EXEC_PATH"

for size in 16 32 128 256; do
    src="$TMPDIR/icons/${size}x${size}.png"
    if [ -f "$src" ]; then
        dest="$ICON_BASE/${size}x${size}/apps/$BIN.png"
        sudo mkdir -p "$(dirname "$dest")"
        sudo cp "$src" "$dest"
        echo "  Icon ${size}x${size} -> $dest"
    fi
done

sudo tee "$APPS_DIR/$DESKTOP" > /dev/null << EOF
[Desktop Entry]
Name=$APP_NAME
Comment=Password Conflict Resolver
Exec=$EXEC_PATH
Icon=$BIN
Type=Application
Categories=Utility;Security;PasswordManager;
Terminal=false
StartupNotify=true
EOF
echo "  Desktop entry -> $APPS_DIR/$DESKTOP"

sudo gtk-update-icon-cache -f -t "$ICON_BASE" 2>/dev/null || true
echo "  Icon cache updated"

rm -rf "$TMPDIR"

echo ""
echo "Done! $APP_NAME installed. Launch from your app menu or run '$BIN'."
