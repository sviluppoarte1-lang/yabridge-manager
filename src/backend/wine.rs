use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::system::WineInstall;

#[derive(Debug, Clone)]
pub struct WineConfig {
    pub graphics_driver: String,
    pub prefix: PathBuf,
    pub version: String,
}

pub fn get_wine_config(wine_bin: &PathBuf, prefix: &PathBuf) -> Result<WineConfig> {
    let version = Command::new(wine_bin)
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let graphics_driver = get_graphics_driver(wine_bin, prefix)?;

    Ok(WineConfig {
        graphics_driver,
        prefix: prefix.clone(),
        version,
    })
}

pub fn get_graphics_driver(_wine_bin: &PathBuf, prefix: &PathBuf) -> Result<String> {
    let reg_path = prefix.join("user.reg");
    if !reg_path.exists() {
        return Ok("x11".to_string());
    }

    let content = fs::read_to_string(&reg_path)?;
    if content.contains("\"Graphics\"=\"x11,wayland\"") {
        Ok("x11,wayland".to_string())
    } else if content.contains("\"Graphics\"=\"wayland\"") {
        Ok("wayland".to_string())
    } else {
        Ok("x11".to_string())
    }
}

pub fn set_graphics_driver(
    wine_bin: &PathBuf,
    _prefix: &PathBuf,
    driver: &str,
) -> Result<()> {
    let reg_content = format!(
        "Windows Registry Editor Version 5.00\n\n\
         [HKEY_CURRENT_USER\\Software\\Wine\\Drivers]\n\
         \"Graphics\"=\"{}\"\n",
        driver
    );

    let tmp_reg = std::env::temp_dir().join("yabridge_graphics_driver.reg");
    fs::write(&tmp_reg, &reg_content)?;

    let output = Command::new(wine_bin)
        .arg("regedit")
        .arg(&tmp_reg)
        .output()
        .context("Failed to run wine regedit")?;

    let _ = fs::remove_file(&tmp_reg);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to set graphics driver: {}", stderr);
    }

    Ok(())
}

pub fn init_wine_prefix(wine_bin: &PathBuf, _prefix: &PathBuf) -> Result<()> {
    let output = Command::new(wine_bin)
        .arg("boot")
        .arg("-u")
        .output()
        .context("Failed to run wineboot")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to initialize Wine prefix: {}", stderr);
    }

    Ok(())
}

pub fn download_and_install_wine(
    version: &str,
    install_dir: &PathBuf,
    progress_cb: Option<Box<dyn Fn(f32, String) + Send>>,
) -> Result<WineInstall> {
    let url = format!(
        "https://github.com/Kron4ek/Wine-Builds/releases/download/{}/wine-{}-staging-amd64.tar.xz",
        version, version
    );

    if let Some(cb) = &progress_cb {
        cb(0.0, format!("Downloading Wine {}...", version));
    }

    let tmp_dir = std::env::temp_dir().join("yabridge-wine-download");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir)?;

    let tarball = tmp_dir.join(format!("wine-{}.tar.xz", version));

    let mut curl = Command::new("curl");
    curl.arg("-L")
        .arg("-o")
        .arg(&tarball)
        .arg(&url)
        .arg("--progress-bar");

    let output = curl.output().context("Failed to download Wine")?;
    if !output.status.success() {
        anyhow::bail!("Failed to download Wine {}", version);
    }

    if let Some(cb) = &progress_cb {
        cb(0.5, format!("Extracting Wine {}...", version));
    }

    fs::create_dir_all(install_dir)?;

    let output = Command::new("tar")
        .args(["-xf", tarball.to_str().unwrap(), "-C", install_dir.to_str().unwrap(), "--strip-components=1"])
        .output()
        .context("Failed to extract Wine")?;

    if !output.status.success() {
        anyhow::bail!("Failed to extract Wine {}", version);
    }

    let _ = fs::remove_dir_all(&tmp_dir);

    if let Some(cb) = &progress_cb {
        cb(1.0, format!("Wine {} installed!", version));
    }

    let wine_bin = install_dir.join("bin/wine");
    Ok(WineInstall {
        version: Command::new(&wine_bin)
            .arg("--version")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| version.to_string()),
        path: install_dir.join("bin"),
        is_staging: true,
        winegpp_path: install_dir.join("bin/wineg++"),
        has_wayland: install_dir.join("lib/wine/x86_64-unix/winewayland.drv.so").exists(),
    })
}
