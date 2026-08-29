use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub distro: String,
    pub kernel: String,
    pub arch: String,
    pub desktop: String,
    pub wayland_display: Option<String>,
    pub x11_display: Option<String>,
    pub display_server: String,
    pub ram_mb: u64,
    pub cpu_cores: usize,
}

#[derive(Debug, Clone)]
pub struct WineInstall {
    pub version: String,
    pub path: PathBuf,
    pub is_staging: bool,
    pub winegpp_path: PathBuf,
    pub has_wayland: bool,
}

#[derive(Debug, Clone)]
pub struct YabridgeInstall {
    pub version: String,
    pub host_exe: PathBuf,
    pub lib_vst2: PathBuf,
    pub lib_vst3: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl SystemInfo {
    pub fn detect() -> Result<Self> {
        let distro = fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("PRETTY_NAME="))
                    .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let kernel = Command::new("uname")
            .arg("-r")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let arch = Command::new("uname")
            .arg("-m")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let desktop = std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| "Unknown".to_string());

        let wayland_display = std::env::var("WAYLAND_DISPLAY").ok();
        let x11_display = std::env::var("DISPLAY").ok();

        let display_server = if wayland_display.is_some() {
            "Wayland".to_string()
        } else if x11_display.is_some() {
            "X11".to_string()
        } else {
            "Unknown".to_string()
        };

        let ram_mb = fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(|v| v / 1024)
            })
            .unwrap_or(0);

        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        Ok(SystemInfo {
            distro,
            kernel,
            arch,
            desktop,
            wayland_display,
            x11_display,
            display_server,
            ram_mb,
            cpu_cores,
        })
    }
}

pub fn find_wine_installations() -> Vec<WineInstall> {
    let mut installs = Vec::new();

    let search_paths = vec![
        PathBuf::from("/opt/wine-11.16/bin"),
        PathBuf::from("/opt/wine-staging/bin"),
        PathBuf::from("/opt/wine-devel/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
    ];

    let mut seen_versions = std::collections::HashSet::new();

    for bin_dir in &search_paths {
        let wine_bin = bin_dir.join("wine");
        if !wine_bin.exists() {
            continue;
        }

        if let Ok(output) = Command::new(&wine_bin).arg("--version").output() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if seen_versions.contains(&version) {
                continue;
            }
            seen_versions.insert(version.clone());

            let winegpp = bin_dir.join("wineg++");
            let has_wayland = bin_dir.join("lib/wine/x86_64-unix/winewayland.drv.so").exists()
                || bin_dir.join("../lib/wine/x86_64-unix/winewayland.drv.so").exists();

            installs.push(WineInstall {
                version,
                path: bin_dir.clone(),
                is_staging: wine_bin.to_string_lossy().contains("staging"),
                winegpp_path: winegpp,
                has_wayland,
            });
        }
    }

    installs.sort_by(|a, b| b.version.cmp(&a.version));
    installs
}

pub fn find_yabridge_install() -> Option<YabridgeInstall> {
    let data_dirs = vec![
        dirs::home_dir()?.join(".local/share/yabridge"),
        PathBuf::from("/usr/share/yabridge"),
        PathBuf::from("/usr/local/share/yabridge"),
    ];

    for dir in data_dirs {
        let host_exe = dir.join("yabridge-host.exe");
        if host_exe.exists() {
            let version = get_yabridge_version().unwrap_or_else(|| "unknown".to_string());
            let config_dir = dirs::config_dir()
                .map(|d| d.join("yabridge"))
                .unwrap_or_default();

            return Some(YabridgeInstall {
                version,
                host_exe: dir.join("yabridge-host.exe"),
                lib_vst2: dir.join("libyabridge-vst2.so"),
                lib_vst3: dir.join("libyabridge-vst3.so"),
                config_dir,
                data_dir: dir,
            });
        }
    }

    None
}

pub fn get_yabridge_version() -> Option<String> {
    let yabridgectl = find_yabridgectl()?;
    let output = Command::new(&yabridgectl)
        .arg("--version")
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.trim().to_string())
}

pub fn find_yabridgectl() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let search = vec![
        home.join(".local/share/yabridge/yabridgectl"),
        PathBuf::from("/usr/local/bin/yabridgectl"),
        home.join(".cargo/bin/yabridgectl"),
        PathBuf::from("/usr/bin/yabridgectl"),
    ];

    search.into_iter().find(|p| p.exists())
}

pub fn is_wine_prefix_initialized(prefix: &PathBuf) -> bool {
    prefix.join("system.reg").exists() || prefix.join("user.reg").exists()
}
