# yabridge-manager

GUI manager for yabridge - the Wine plugin bridge for Linux DAWs.

## Features

- Wine 11.16 staging setup and configuration
- Automatic VST2/VST3/CLAP plugin scanning
- One-click sync and setup
- X11/Wayland display driver configuration
- System diagnostics

## Build from source

```bash
cargo build --release
./target/release/yabridge-manager
```

## Install packages

### .deb (Ubuntu/Debian)
```bash
make deb
sudo dpkg -i yabridge-manager_1.0.0_amd64.deb
```

### AppImage (any distro)
```bash
make appimage
chmod +x yabridge-manager-1.0.0-*.AppImage
./yabridge-manager-1.0.0-*.AppImage
```

## Dependencies

- Wine 11.16 staging (for plugin hosting)
- yabridge (for plugin bridging)
- ALSA or PipeWire (for audio)

## License

GPL-3.0-or-later
