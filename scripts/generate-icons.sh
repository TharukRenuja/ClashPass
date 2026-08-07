#!/usr/bin/env bash
set -e
# Generate PNG icons from SVG at various sizes
# Requires ImageMagick (convert) or librsvg (rsvg-convert)

SVG="icons/clashpass.svg"
OUTDIR="icons"

if [ ! -f "$SVG" ]; then
    echo "Run from project root: $SVG not found"
    exit 1
fi

mkdir -p "$OUTDIR"

# Use ImageMagick if available, else rsvg-convert
if command -v convert &>/dev/null; then
    for size in 32 64 128 256; do
        convert "$SVG" -resize "${size}x${size}" "$OUTDIR/clashpass_${size}.png"
        echo "  $OUTDIR/clashpass_${size}.png"
    done
elif command -v rsvg-convert &>/dev/null; then
    for size in 32 64 128 256; do
        rsvg-convert "$SVG" -w "$size" -h "$size" -o "$OUTDIR/clashpass_${size}.png"
        echo "  $OUTDIR/clashpass_${size}.png"
    done
else
    echo "Need ImageMagick or librsvg installed"
    exit 1
fi

# Also copy 256px as default app icon
cp "$OUTDIR/clashpass_256.png" "$OUTDIR/clashpass.png"
echo "  $OUTDIR/clashpass.png"
