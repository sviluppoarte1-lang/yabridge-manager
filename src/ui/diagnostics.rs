use eframe::egui;
use crate::backend::system;

#[derive(Default)]
pub struct DiagnosticsPage {
    pub system_info: Option<system::SystemInfo>,
    pub log_output: String,
    pub is_loading: bool,
}

impl DiagnosticsPage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Diagnostics");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_system_check(ui);
            ui.add_space(12.0);
            self.show_logs(ui);
        });
    }

    fn show_system_check(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("System Check", |ui| {
            if self.system_info.is_none() {
                self.system_info = system::SystemInfo::detect().ok();
            }

            if let Some(ref info) = self.system_info {
                let wayland_display = info.wayland_display.as_deref().unwrap_or("Not set");
                let x11_display = info.x11_display.as_deref().unwrap_or("Not set");
                let checks = vec![
                    ("Display Server", info.display_server.as_str(), info.display_server != "Unknown"),
                    ("WAYLAND_DISPLAY", wayland_display, info.wayland_display.is_some()),
                    ("DISPLAY", x11_display, info.x11_display.is_some()),
                ];

                egui::Grid::new("checks_grid").show(ui, |ui| {
                    for (name, value, ok) in &checks {
                        ui.label(*name);
                        ui.label(*value);
                        let color = if *ok { egui::Color32::GREEN } else { egui::Color32::YELLOW };
                        ui.colored_label(color, if *ok { "\u{2714}" } else { "\u{26A0}" });
                        ui.end_row();
                    }
                });

                ui.add_space(8.0);

                ui.label(format!("Distro: {}", info.distro));
                ui.label(format!("Kernel: {}", info.kernel));
                ui.label(format!("Arch: {}", info.arch));
                ui.label(format!("RAM: {} MB", info.ram_mb));
                ui.label(format!("CPU Cores: {}", info.cpu_cores));
            }
        });
    }

    fn show_logs(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Logs", |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load yabridgectl status").clicked() {
                    self.load_yabridgectl_status();
                }
                if ui.button("Load Wine debug log").clicked() {
                    self.load_wine_debug();
                }
                if ui.button("Clear").clicked() {
                    self.log_output.clear();
                }
            });

            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.log_output.as_str())
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });
        });
    }

    fn load_yabridgectl_status(&mut self) {
        self.is_loading = true;
        self.log_output = "Loading...".to_string();

        if let Some(yabridgectl) = system::find_yabridgectl() {
            match std::process::Command::new(&yabridgectl)
                .arg("status")
                .output()
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.log_output = format!("=== yabridgectl status ===\n{}", stdout);
                    if !stderr.is_empty() {
                        self.log_output.push_str(&format!("\nSTDERR:\n{}", stderr));
                    }
                }
                Err(e) => {
                    self.log_output = format!("Failed to run yabridgectl: {}", e);
                }
            }
        } else {
            self.log_output = "yabridgectl not found in PATH".to_string();
        }

        self.is_loading = false;
    }

    fn load_wine_debug(&mut self) {
        self.log_output = "Loading Wine debug info...".to_string();

        let home = dirs::home_dir().unwrap_or_default();
        let prefix = home.join(".wine");

        if let Ok(output) = std::process::Command::new("/opt/wine-11.16/bin/wine")
            .arg("--version")
            .output()
        {
            let version = String::from_utf8_lossy(&output.stdout);
            self.log_output = format!("=== Wine Version ===\n{}\n", version.trim());
        }

        if prefix.exists() {
            self.log_output.push_str(&format!(
                "\n=== Wine Prefix ===\n{}\n",
                prefix.display()
            ));

            if let Ok(entries) = std::fs::read_dir(&prefix) {
                self.log_output.push_str("\nPrefix contents:\n");
                for entry in entries.flatten() {
                    self.log_output.push_str(&format!(
                        "  {}\n",
                        entry.file_name().to_string_lossy()
                    ));
                }
            }
        }
    }
}
