use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PluginFormat {
    Vst2,
    Vst3,
    Clap,
}

impl std::fmt::Display for PluginFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginFormat::Vst2 => write!(f, "VST2"),
            PluginFormat::Vst3 => write!(f, "VST3"),
            PluginFormat::Clap => write!(f, "CLAP"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub path: PathBuf,
    pub format: PluginFormat,
    pub vendor: String,
    pub enabled: bool,
    pub size_kb: u64,
}

#[derive(Debug, Clone)]
pub struct PluginScanResult {
    pub plugins: Vec<Plugin>,
    pub total_size_kb: u64,
    pub scan_dirs: Vec<PathBuf>,
}

pub fn scan_plugins(dirs: &[PathBuf]) -> PluginScanResult {
    let mut plugins = Vec::new();
    let mut total_size_kb = 0u64;

    for dir in dirs {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            let format = match ext {
                "dll" | "so" if is_vst2(path) => PluginFormat::Vst2,
                "vst3" => PluginFormat::Vst3,
                "clap" => PluginFormat::Clap,
                _ => continue,
            };

            let metadata = fs::metadata(path).ok();
            let size_kb = metadata.as_ref().map(|m| m.len() / 1024).unwrap_or(0);
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let vendor = extract_vendor(&name, path);

            plugins.push(Plugin {
                name,
                path: path.to_path_buf(),
                format,
                vendor,
                enabled: true,
                size_kb,
            });

            total_size_kb += size_kb;
        }
    }

    plugins.sort_by(|a, b| a.name.cmp(&b.name));

    PluginScanResult {
        plugins,
        total_size_kb,
        scan_dirs: dirs.to_vec(),
    }
}

fn is_vst2(path: &Path) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    name.ends_with(".dll") || name.ends_with(".so") || name.ends_with(".vst")
}

fn extract_vendor(name: &str, path: &Path) -> String {
    let parent_name = path.parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if !parent_name.is_empty() && parent_name != name {
        return parent_name.to_string();
    }

    let known_vendors = vec![
        ("FabFilter", "FabFilter"),
        ("Soundtoys", "Soundtoys"),
        ("Waves", "Waves"),
        ("Valhalla", "ValhallaDSP"),
        ("iZotope", "iZotope"),
        ("Slate", "Slate Digital"),
        ("Arturia", "Arturia"),
        ("Native Instruments", "NI"),
        ("Plugin Alliance", "Plugin Alliance"),
        ("U-he", "U-he"),
        ("Kilohearts", "Kilohearts"),
        ("MeldaProduction", "MeldaProduction"),
        ("TDR", "Tokyo Dawn Records"),
        ("TAL", "Togu Audio Line"),
        ("Voxengo", "Voxengo"),
        ("LennarDigital", "LennarDigital"),
        ("Xfer", "Xfer Records"),
        ("Cableguys", "Cableguys"),
        ("Devious", "Devious Machines"),
    ];

    for (pattern, vendor) in &known_vendors {
        if name.to_lowercase().contains(&pattern.to_lowercase()) {
            return vendor.to_string();
        }
    }

    "Unknown".to_string()
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
