#!/bin/bash
set -e

BIN="clashpass"
DESKTOP="clashpass.desktop"
APP_NAME="ClashPass"
EXEC_PATH="/usr/local/bin/$BIN"
ICON_BASE="/usr/share/icons/hicolor"
APPS_DIR="/usr/share/applications"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ "$(id -u)" -ne 0 ]; then
    SUDO="sudo"
else
    SUDO=""
fi

if [ ! -f "$SCRIPT_DIR/$BIN" ]; then
    echo "Error: $BIN not found in current directory."
    exit 1
fi

echo "Installing $APP_NAME..."

$SUDO cp "$SCRIPT_DIR/$BIN" "$EXEC_PATH"
$SUDO chmod +x "$EXEC_PATH"
echo "  Binary -> $EXEC_PATH"

for size in 16 32 128 256; do
    src="$SCRIPT_DIR/icons/${size}x${size}.png"
    if [ -f "$src" ]; then
        dest="$ICON_BASE/${size}x${size}/apps/$BIN.png"
        $SUDO mkdir -p "$(dirname "$dest")"
        $SUDO cp "$src" "$dest"
        echo "  Icon ${size}x${size} -> $dest"
    fi
done

$SUDO tee "$APPS_DIR/$DESKTOP" > /dev/null << EOF
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

$SUDO gtk-update-icon-cache -f -t "$ICON_BASE" 2>/dev/null || true
echo "  Icon cache updated"

echo ""
echo "Done! $APP_NAME installed. Launch from your app menu or run '$BIN'."
