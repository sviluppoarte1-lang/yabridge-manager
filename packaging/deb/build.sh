#!/bin/bash
set -e

PKG_NAME="yabridge-manager"
VERSION="1.0.0"
ARCH="amd64"
BUILD_DIR="build/deb"
PREFIX="/usr"

echo "=== Building .deb package ==="

# Clean
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/$PREFIX/bin"
mkdir -p "$BUILD_DIR/$PREFIX/share/applications"
mkdir -p "$BUILD_DIR/$PREFIX/share/icons/hicolor/256x256/apps"
mkdir -p "$BUILD_DIR/$PREFIX/share/yabridge-manager/yabridge"

# Build release binary
echo "Compiling release binary..."
cargo build --release
cp target/release/yabridge-manager "$BUILD_DIR/$PREFIX/bin/"

# Copy bundled yabridge binaries
echo "Copying bundled yabridge binaries..."
cp bundled/yabridge/* "$BUILD_DIR/$PREFIX/share/yabridge-manager/yabridge/"

# Desktop file
cat > "$BUILD_DIR/$PREFIX/share/applications/yabridge-manager.desktop" << 'EOF'
[Desktop Entry]
Type=Application
Name=yabridge-manager
Comment=GUI manager for yabridge - Wine plugin bridge
Exec=yabridge-manager
Icon=yabridge-manager
Terminal=false
Categories=AudioVideo;Audio;
Keywords=audio;vst;wine;plugin;
EOF

# Icon
cat > "$BUILD_DIR/$PREFIX/share/icons/hicolor/256x256/apps/yabridge-manager.svg" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <rect width="256" height="256" rx="32" fill="#2D2D2D"/>
  <text x="128" y="110" text-anchor="middle" fill="#E0E0E0" font-family="sans-serif" font-size="36" font-weight="bold">yabridge</text>
  <text x="128" y="160" text-anchor="middle" fill="#4FC3F7" font-family="sans-serif" font-size="48" font-weight="bold">YM</text>
</svg>
EOF

# Control file
mkdir -p "$BUILD_DIR/DEBIAN"
cat > "$BUILD_DIR/DEBIAN/control" << EOF
Package: $PKG_NAME
Version: $VERSION
Architecture: $ARCH
Maintainer: yabridge-manager contributors
Depends: libgtk-3-0 (>= 3.24), libxcb-render0, libxcb-shape0, libxcb-xfixes0, libxcb1, libxkbcommon0
Section: sound
Priority: optional
Homepage: https://github.com/robbert-vdh/yabridge
Description: GUI manager for yabridge
 A graphical interface to manage yabridge, the Wine plugin bridge
 for Linux DAWs. Supports VST2, VST3, and CLAP plugin formats.
Features:
 - Wine 11.16 staging setup and configuration
 - Automatic plugin scanning and categorization
 - One-click sync and setup
 - X11/Wayland display driver configuration
EOF

# Build .deb
echo "Building .deb..."
dpkg-deb --build "$BUILD_DIR" "${PKG_NAME}_${VERSION}_${ARCH}.deb"

echo "=== Done: ${PKG_NAME}_${VERSION}_${ARCH}.deb ==="
echo "Total size:"
du -sh "${PKG_NAME}_${VERSION}_${ARCH}.deb"
