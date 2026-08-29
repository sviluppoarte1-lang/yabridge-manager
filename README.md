# yabridge-manager

A native Linux GUI for managing [yabridge](https://github.com/robbert-vdh/yabridge) — the Wine plugin bridge that lets you use Windows VST2, VST3, and CLAP audio plugins in Linux DAWs.

Built with Rust and [egui](https://github.com/emilk/egui), yabridge-manager provides a complete setup experience: from installing Wine 11.16 staging to scanning plugins and configuring display drivers. Ships with a **custom yabridge build** pre-bundled — no terminal required.

## Why?

Setting up yabridge manually involves multiple terminal commands: downloading Wine, cross-compiling yabridge with `wineg++`, running `yabridgectl sync`, and configuring Wine registry keys for Wayland support. yabridge-manager wraps all of this into a single graphical interface that works out of the box, including support for **Wayland** and **X11** display servers.

## Features

### Setup & Installation
- **One-click Wine install** — Downloads and installs Wine 11.16 staging (Kron4ek builds) to `~/.local/share/yabridge/wine-11.16/`
- **One-click yabridge install** — Deploys the bundled custom yabridge build (Wayland-compatible) to `~/.local/share/yabridge/`
- **Multi-Wine detection** — Scans `/opt/wine-*`, `/usr/local/bin`, `/usr/bin` and shows all found Wine installations
- **yabridgectl sync** — Runs `yabridgectl sync` with the selected Wine binary to register plugins
- **Automatic PATH detection** — Finds `yabridgectl` in `~/.local/share/yabridge/`, `~/.cargo/bin/`, `/usr/local/bin/`, `/usr/bin/`

### Plugin Manager
- **Scans VST2, VST3, and CLAP** plugins across configured directories
- **Filter by format** — Toggle VST2 / VST3 / CLAP visibility
- **Text search** — Filter plugins by name or vendor in real-time
- **Vendor detection** — Recognizes 20+ known vendors (FabFilter, Soundtoys, Waves, Valhalla, iZotope, Arturia, etc.)
- **Custom scan directories** — Add/remove plugin directories via file browser
- **Size reporting** — Shows total plugin library size

### Configuration
- **Graphics driver switching** — Set Wine's `Graphics` registry key to `x11`, `x11,wayland` (recommended), or `wayland`
- **Applies via `wine regedit`** — Uses Wine's own registry editor to write the change
- **Plugin directory management** — Add custom scan paths for yabridgectl

### Diagnostics
- **System check** — Detects display server (Wayland/X11), `WAYLAND_DISPLAY`, `DISPLAY`, distro, kernel, architecture, RAM, CPU cores
- **yabridgectl status** — Loads full `yabridgectl status` output for debugging
- **Wine debug info** — Shows Wine version, prefix location, and prefix contents
- **Log viewer** — Scrollable monospace log output with clear button

## Bundled yabridge

This project ships with a **custom yabridge build** compiled from a modified source tree with the following changes:

- **Wayland embedding support** — Plugin windows can be embedded in DAWs running under XWayland
- **Wine 11.16 compatibility** — Built using Wine 11.16 staging's clang-based `wineg++` with a custom wrapper script that resolves clang/LLVM linker incompatibilities
- **XDG Foreign Protocol v2** — Uses `zxdg_exporter_v2` for cross-process window handle sharing on Wayland
- **Graphics driver auto-detection** — Reads Wine registry to select the correct backend

The bundled binaries are tracked with **Git LFS** and include:

| File | Description |
|------|-------------|
| `yabridge-host.exe` | Wine PE loader script |
| `yabridge-host.exe.so` | Native host library (98 MB) |
| `yabridgectl` | CLI management tool |
| `libyabridge-vst2.so` | VST2 bridge library |
| `libyabridge-vst3.so` | VST3 bridge library (83 MB) |
| `libyabridge-clap.so` | CLAP bridge library |
| `libyabridge-chainloader-vst2.so` | VST2 chainloader |
| `libyabridge-chainloader-vst3.so` | VST3 chainloader |
| `libyabridge-chainloader-clap.so` | CLAP chainloader |

## Installation

### .deb (Ubuntu, Debian, Linux Mint, Pop!_OS)

```bash
make deb
sudo dpkg -i yabridge-manager_1.0.0_amd64.deb
sudo apt install -f  # install dependencies if needed
```

### AppImage (any Linux distro)

```bash
make appimage
chmod +x yabridge-manager-1.0.0-x86_64.AppImage
./yabridge-manager-1.0.0-x86_64.AppImage
```

The AppImage is self-contained and works on any distro with glibc >= 2.17 (Ubuntu 18.04+, Fedora 27+, Debian 10+, Arch, etc.).

### Build from source

```bash
# Requirements: Rust 1.75+, pkg-config, libgtk-3-dev (or equivalent)
git clone https://github.com/sviluppoarte1-lang/yabridge-manager.git
cd yabridge-manager
cargo build --release
./target/release/yabridge-manager
```

**Note:** Git LFS is required to clone the bundled yabridge binaries:
```bash
git lfs install
git clone https://github.com/sviluppoarte1-lang/yabridge-manager.git
```

## Requirements

- **Linux** with X11 or Wayland display server
- **Wine 11.16 staging** (can be installed via the app)
- **yabridge** (bundled, installed via the app)
- **DAW** that supports VST2, VST3, or CLAP plugins (Reaper, Ardour, Bitwig, Mixbus, etc.)

## Project Structure

```
yabridge-manager/
├── Cargo.toml                          # Rust project manifest
├── Makefile                            # make deb / make appimage / make clean
├── bundled/yabridge/                   # Pre-built yabridge binaries (Git LFS)
├── packaging/
│   ├── deb/build.sh                    # .deb package builder
│   └── appimage/build.sh              # AppImage builder
└── src/
    ├── main.rs                         # Entry point, window config
    ├── app.rs                          # Main App struct, page routing
    ├── backend/
    │   ├── system.rs                   # System detection, Wine/yabridge discovery
    │   ├── wine.rs                     # Wine config, registry, download/install
    │   ├── yabridge.rs                 # yabridge operations, sync, bundled detection
    │   └── plugin.rs                   # Plugin scanning, format detection, vendor lookup
    └── ui/
        ├── sidebar.rs                  # Navigation sidebar
        ├── setup.rs                    # Setup & Installation page
        ├── plugins.rs                  # Plugin Manager page
        ├── config.rs                   # Configuration page
        └── diagnostics.rs             # Diagnostics & Logs page
```

## Tech Stack

| Component | Technology |
|-----------|-----------|
| GUI framework | [egui](https://github.com/emilk/egui) 0.31 + eframe |
| Language | Rust 2021 edition |
| Async runtime | tokio |
| File dialogs | rfd (native file picker) |
| Plugin scanning | walkdir |
| Serialization | serde + serde_json + toml |
| Build system | Cargo (release profile: LTO, strip, single codegen unit) |

## License

GPL-3.0-or-later

## Acknowledgments

- [yabridge](https://github.com/robbert-vdh/yabridge) by Robbert van der Helm — the original Wine plugin bridge
- [Kron4ek Wine Builds](https://github.com/Kron4ek/Wine-Builds) — staging Wine builds used by this project
- [egui](https://github.com/emilk/egui) by Emil Ernerfeldt — the immediate mode GUI framework
