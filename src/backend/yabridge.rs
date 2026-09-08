use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::system::{find_yabridgectl, YabridgeInstall};

#[derive(Debug, Clone)]
pub struct YabridgeConfig {
    pub install: Option<YabridgeInstall>,
    pub plugin_dirs: Vec<PathBuf>,
    pub is_synced: bool,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub plugins_found: usize,
    pub plugins_synced: usize,
    pub errors: Vec<String>,
    pub output: String,
}

pub fn get_config() -> YabridgeConfig {
    let install = super::system::find_yabridge_install();
    let plugin_dirs = get_plugin_dirs();
    let is_synced = check_if_synced();

    YabridgeConfig {
        install,
        plugin_dirs,
        is_synced,
    }
}

pub fn get_plugin_dirs() -> Vec<PathBuf> {
    let yabridgectl = match find_yabridgectl() {
        Some(p) => p,
        None => return default_plugin_dirs(),
    };

    let output = Command::new(&yabridgectl)
        .arg("status")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut dirs = Vec::new();
    for line in output.lines() {
        if let Some(path_str) = line.strip_prefix("  Plugin directories:").or_else(|| {
            if line.starts_with('/') || line.starts_with("    ") {
                Some(line.trim())
            } else {
                None
            }
        }) {
            let path = PathBuf::from(path_str.trim());
            if path.exists() {
                dirs.push(path);
            }
        }
    }

    if dirs.is_empty() {
        default_plugin_dirs()
    } else {
        dirs
    }
}

pub fn default_plugin_dirs() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    vec![
        home.join(".wine/drive_c/Program Files/Common Files/VST2"),
        home.join(".wine/drive_c/Program Files/Common Files/VST3"),
        home.join(".wine/drive_c/Program Files/VSTPlugins"),
        home.join(".wine/drive_c/Program Files/Steinberg/VSTPlugins"),
        home.join(".wine/drive_c/Program Files/Cakewalk/VST Plugins"),
    ]
}

fn check_if_synced() -> bool {
    let install = match super::system::find_yabridge_install() {
        Some(i) => i,
        None => return false,
    };
    install.host_exe.exists() && install.lib_vst2.exists()
}

pub fn run_sync(wine_bin: &PathBuf) -> Result<SyncResult> {
    let yabridgectl = find_yabridgectl()
        .context("yabridgectl not found. Is yabridge installed?")?;

    let output = Command::new(&yabridgectl)
        .arg("sync")
        .env("PATH", format!("{}:{}", wine_bin.parent().unwrap().to_string_lossy(), std::env::var("PATH").unwrap_or_default()))
        .output()
        .context("Failed to run yabridgectl sync")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{}\n{}", stdout, stderr);

    let plugins_found = combined
        .lines()
        .filter(|l| l.contains("plugin") || l.contains("VST"))
        .count();

    let errors: Vec<String> = combined
        .lines()
        .filter(|l| l.contains("error") || l.contains("Error") || l.contains("failed"))
        .map(|l| l.trim().to_string())
        .collect();

    Ok(SyncResult {
        success: output.status.success(),
        plugins_found,
        plugins_synced: if output.status.success() { plugins_found } else { 0 },
        errors,
        output: combined,
    })
}

pub fn setup_wine_prefix_for_yabridge(
    wine_bin: &PathBuf,
    prefix: &PathBuf,
) -> Result<()> {
    let yabridgectl = find_yabridgectl()
        .context("yabridgectl not found")?;

    Command::new(&yabridgectl)
        .arg("setup")
        .arg("--wine-prefix")
        .arg(prefix)
        .env("PATH", format!("{}:{}", wine_bin.parent().unwrap().to_string_lossy(), std::env::var("PATH").unwrap_or_default()))
        .output()
        .context("Failed to run yabridgectl setup")?;

    Ok(())
}

pub fn copy_binaries(data_dir: &PathBuf, build_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(data_dir)?;

    let files = vec![
        "yabridge-host.exe",
        "yabridge-host.exe.so",
        "yabridgectl",
        "libyabridge-vst2.so",
        "libyabridge-vst3.so",
        "libyabridge-clap.so",
        "libyabridge-chainloader-vst2.so",
        "libyabridge-chainloader-vst3.so",
        "libyabridge-chainloader-clap.so",
    ];

    let mut copied = 0;

    for file in &files {
        let src_path = build_dir.join(file);
        let dst_path = data_dir.join(file);

        if src_path.exists() {
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy {}", file))?;
            copied += 1;
        }
    }

    if copied == 0 {
        anyhow::bail!("No files found in {}", build_dir.display())
    }

    Ok(())
}

pub fn find_bundled_yabridge_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let candidates = vec![
        exe_dir.join("../share/yabridge-manager/yabridge"),
        exe_dir.join("../share/yabridge"),
        exe_dir.join("bundled/yabridge"),
    ];

    candidates.into_iter().find(|p| {
        p.join("yabridge-host.exe").exists()
            && p.join("libyabridge-vst2.so").exists()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn unique_tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "yabridge-test-{}-{}",
            name,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copy_binaries_copies_all_files() {
        let src = unique_tmp("src");
        let dst = unique_tmp("dst");
        let files = [
            "yabridge-host.exe",
            "yabridge-host.exe.so",
            "yabridgectl",
            "libyabridge-vst2.so",
            "libyabridge-vst3.so",
            "libyabridge-clap.so",
            "libyabridge-chainloader-vst2.so",
            "libyabridge-chainloader-vst3.so",
            "libyabridge-chainloader-clap.so",
        ];
        for f in &files {
            fs::write(src.join(f), format!("contents-of-{}", f)).unwrap();
        }

        copy_binaries(&dst, &src).unwrap();

        for f in &files {
            let content = fs::read_to_string(dst.join(f)).unwrap();
            assert_eq!(content, format!("contents-of-{}", f));
        }

        fs::remove_dir_all(&src).ok();
        fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn copy_binaries_errors_on_empty_source() {
        let src = unique_tmp("empty");
        let dst = unique_tmp("dst2");
        assert!(copy_binaries(&dst, &src).is_err());
        fs::remove_dir_all(&src).ok();
        fs::remove_dir_all(&dst).ok();
    }

    #[test]
    fn copy_binaries_overwrites_older_files() {
        let src = unique_tmp("new");
        let dst = unique_tmp("old");
        fs::write(src.join("yabridge-host.exe.so"), "new-binary").unwrap();
        fs::write(src.join("yabridge-host.exe"), "new-exe").unwrap();
        fs::write(dst.join("yabridge-host.exe.so"), "old-binary").unwrap();

        copy_binaries(&dst, &src).unwrap();

        assert_eq!(
            fs::read_to_string(dst.join("yabridge-host.exe.so")).unwrap(),
            "new-binary"
        );

        fs::remove_dir_all(&src).ok();
        fs::remove_dir_all(&dst).ok();
    }
}

pub fn needs_update() -> Option<bool> {
    let bundled = find_bundled_yabridge_dir()?;
    let data_dir = dirs::home_dir()?.join(".local/share/yabridge");

    let bundled_bin = bundled.join("yabridge-host.exe.so");
    let installed_bin = data_dir.join("yabridge-host.exe.so");

    if !installed_bin.exists() {
        return Some(true);
    }

    let bundled_meta = fs::metadata(&bundled_bin).ok()?;
    let installed_meta = fs::metadata(&installed_bin).ok()?;

    let bundled_time = bundled_meta.modified().ok()?;
    let installed_time = installed_meta.modified().ok()?;

    Some(bundled_time > installed_time)
}

pub fn update_installed_binaries() -> Result<String> {
    let bundled = find_bundled_yabridge_dir()
        .context("Bundled yabridge binaries not found")?;
    let data_dir = dirs::home_dir()
        .context("Cannot determine home directory")?
        .join(".local/share/yabridge");

    copy_binaries(&data_dir, &bundled)?;

    Ok(format!(
        "yabridge updated successfully in {}",
        data_dir.display()
    ))
}
