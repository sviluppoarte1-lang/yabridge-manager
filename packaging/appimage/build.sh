#!/bin/bash
set -e

APP_NAME="yabridge-manager"
VERSION="1.0.0"
APPDIR="build/AppDir"

echo "=== Building AppImage ==="

# Clean
rm -rf build/AppDir build/*.AppImage

# Build release binary
echo "Compiling release binary..."
cargo build --release

# Create AppDir structure
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/yabridge-manager/yabridge"

cp target/release/yabridge-manager "$APPDIR/usr/bin/"

# Copy bundled yabridge binaries
echo "Copying bundled yabridge binaries..."
cp bundled/yabridge/* "$APPDIR/usr/share/yabridge-manager/yabridge/"

# Desktop file
cat > "$APPDIR/usr/share/applications/$APP_NAME.desktop" << 'EOF'
[Desktop Entry]
Type=Application
Name=yabridge-manager
Comment=GUI manager for yabridge - Wine plugin bridge
Exec=yabridge-manager
Icon=yabridge-manager
Terminal=false
Categories=AudioVideo;Audio;
EOF

cp "$APPDIR/usr/share/applications/$APP_NAME.desktop" "$APPDIR/"

# Icon
cat > "$APPDIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.svg" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <rect width="256" height="256" rx="32" fill="#2D2D2D"/>
  <text x="128" y="110" text-anchor="middle" fill="#E0E0E0" font-family="sans-serif" font-size="36" font-weight="bold">yabridge</text>
  <text x="128" y="160" text-anchor="middle" fill="#4FC3F7" font-family="sans-serif" font-size="48" font-weight="bold">YM</text>
</svg>
EOF

cp "$APPDIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.svg" "$APPDIR/"

# AppRun script
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "$HERE/usr/bin/yabridge-manager" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# Download appimagetool if not present
APPIMAGETOOL="build/appimagetool"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo "Downloading appimagetool..."
    ARCH=$(uname -m)
    curl -L -o "$APPIMAGETOOL" \
        "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-${ARCH}.AppImage"
    chmod +x "$APPIMAGETOOL"
fi

# Build AppImage
ARCH=$(uname -m)
OUTPUT="${APP_NAME}-${VERSION}-${ARCH}.AppImage"
ARCH=$ARCH "$APPIMAGETOOL" "$APPDIR" "$OUTPUT"

echo "=== Done: $OUTPUT ==="
echo "Total size:"
du -sh "$OUTPUT"
