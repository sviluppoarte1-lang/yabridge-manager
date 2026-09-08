use std::sync::mpsc::Receiver;

use eframe::egui;
use crate::backend::{system, wine, yabridge};

/// Result of a background task, sent back to the UI thread.
pub struct BgTaskResult {
    pub message: String,
    pub is_error: bool,
}

#[derive(Default)]
pub struct SetupPage {
    pub wine_install: Option<system::WineInstall>,
    pub yabridge_install: Option<system::YabridgeInstall>,
    pub system_info: Option<system::SystemInfo>,
    pub wine_installs: Vec<system::WineInstall>,
    pub selected_wine: Option<usize>,
    pub status_message: String,
    pub is_error: bool,
    pub is_working: bool,
    pub progress: f32,
    pub wine_version_to_install: String,
    pub update_available: Option<bool>,
    bg_result_rx: Option<Receiver<BgTaskResult>>,
}

impl SetupPage {
    pub fn new() -> Self {
        Self {
            wine_version_to_install: "11.16".to_string(),
            ..Default::default()
        }
    }

    pub fn refresh(&mut self) {
        self.system_info = system::SystemInfo::detect().ok();
        self.wine_installs = system::find_wine_installations();
        self.yabridge_install = system::find_yabridge_install();
        self.wine_install = self.wine_installs.first().cloned();
        self.update_available = yabridge::needs_update();
    }

    /// Check for a finished background task (non-blocking) and apply its
    /// result to the UI state.
    fn poll_bg_tasks(&mut self) {
        let done = match &self.bg_result_rx {
            Some(rx) => match rx.try_recv() {
                Ok(res) => {
                    self.is_working = false;
                    self.is_error = res.is_error;
                    self.status_message = res.message;
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_working = false;
                    self.is_error = true;
                    self.status_message =
                        "Background task ended unexpectedly.".to_string();
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => false,
            },
            None => false,
        };
        if done {
            self.bg_result_rx = None;
            self.refresh();
        }
    }

    /// Run `task` on a worker thread. The UI stays responsive and `poll_bg_tasks`
    /// picks up the result when done (unblocks buttons, shows the message).
    fn run_bg(
        &mut self,
        start_msg: &str,
        task: impl FnOnce() -> BgTaskResult + Send + 'static,
    ) {
        self.is_working = true;
        self.is_error = false;
        self.status_message = start_msg.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        self.bg_result_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(task());
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Setup & Installation");
        ui.separator();

        if self.system_info.is_none() {
            self.refresh();
        }

        // Keep repainting while a background task runs so its result is
        // picked up as soon as it finishes, then apply it.
        if self.bg_result_rx.is_some() {
            ui.ctx().request_repaint();
        }
        self.poll_bg_tasks();

        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_system_info(ui);
            ui.add_space(12.0);
            self.show_wine_section(ui);
            ui.add_space(12.0);
            self.show_yabridge_section(ui);
            ui.add_space(12.0);
            self.show_status(ui);
        });
    }

    fn show_system_info(&mut self, ui: &mut egui::Ui) {
        if let Some(ref info) = self.system_info {
            ui.collapsing("System Information", |ui| {
                egui::Grid::new("system_info_grid").show(ui, |ui| {
                    ui.label("Distro:");
                    ui.label(&info.distro);
                    ui.end_row();

                    ui.label("Kernel:");
                    ui.label(&info.kernel);
                    ui.end_row();

                    ui.label("Architecture:");
                    ui.label(&info.arch);
                    ui.end_row();

                    ui.label("Desktop:");
                    ui.label(&info.desktop);
                    ui.end_row();

                    ui.label("Display Server:");
                    let color = match info.display_server.as_str() {
                        "Wayland" => egui::Color32::from_rgb(100, 200, 100),
                        "X11" => egui::Color32::from_rgb(100, 150, 255),
                        _ => egui::Color32::GRAY,
                    };
                    ui.colored_label(color, &info.display_server);
                    ui.end_row();

                    ui.label("RAM:");
                    ui.label(format!("{} MB", info.ram_mb));
                    ui.end_row();

                    ui.label("CPU Cores:");
                    ui.label(format!("{}", info.cpu_cores));
                    ui.end_row();
                });
            });
        }
    }

    fn show_wine_section(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Wine", |ui| {
            if self.wine_installs.is_empty() {
                ui.label("No Wine installation found.");

                ui.add_space(8.0);
                ui.label("Install Wine 11.16 Staging (recommended):");

                ui.horizontal(|ui| {
                    ui.label("Version:");
                    ui.text_edit_singleline(&mut self.wine_version_to_install);
                });

                if ui.button("Download & Install Wine").clicked() && !self.is_working {
                    let version = self.wine_version_to_install.clone();
                    let install_dir = dirs::home_dir()
                        .unwrap_or_default()
                        .join(format!(".local/share/yabridge/wine-{}", version));

                    self.run_bg("Starting download...", move || {
                        match wine::download_and_install_wine(
                            &version,
                            &install_dir,
                            None,
                        ) {
                            Ok(_) => BgTaskResult {
                                message: format!(
                                    "Wine {} installed successfully.",
                                    version
                                ),
                                is_error: false,
                            },
                            Err(e) => BgTaskResult {
                                message: format!(
                                    "Failed to install Wine: {:#}",
                                    e
                                ),
                                is_error: true,
                            },
                        }
                    });
                }
            } else {
                ui.label(format!("Found {} Wine installation(s):", self.wine_installs.len()));

                for (i, install) in self.wine_installs.iter().enumerate() {
                    let is_selected = self.selected_wine == Some(i);

                    ui.horizontal(|ui| {
                        if ui.selectable_label(is_selected, &install.version).clicked() {
                            self.selected_wine = Some(i);
                        }

                        if install.has_wayland {
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "Wayland");
                        }

                        ui.label(format!("({})", install.path.display()));
                    });
                }

                if let Some(ref install) = self.wine_install {
                    ui.add_space(8.0);
                    ui.label(format!("Selected: {} at {}", install.version, install.path.display()));
                }
            }
        });
    }

    fn show_yabridge_section(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("yabridge", |ui| {
            match &self.yabridge_install {
                Some(install) => {
                    egui::Grid::new("yabridge_info").show(ui, |ui| {
                        ui.label("Version:");
                        ui.label(&install.version);
                        ui.end_row();

                        ui.label("Host EXE:");
                        ui.label(install.host_exe.display().to_string());
                        ui.end_row();

                        ui.label("VST2 Library:");
                        let exists = install.lib_vst2.exists();
                        let color = if exists { egui::Color32::GREEN } else { egui::Color32::RED };
                        ui.colored_label(color, if exists { "OK" } else { "Missing" });
                        ui.end_row();

                        ui.label("VST3 Library:");
                        let exists = install.lib_vst3.exists();
                        let color = if exists { egui::Color32::GREEN } else { egui::Color32::RED };
                        ui.colored_label(color, if exists { "OK" } else { "Missing" });
                        ui.end_row();
                    });

                    ui.add_space(8.0);

                    if self.update_available == Some(true) {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 200, 50),
                                "Update available",
                            );
                            if ui.button("Update yabridge").clicked() && !self.is_working {
                                self.run_bg("Updating yabridge...", || {
                                    match yabridge::update_installed_binaries() {
                                        Ok(msg) => BgTaskResult {
                                            message: msg,
                                            is_error: false,
                                        },
                                        Err(e) => BgTaskResult {
                                            message: format!(
                                                "Failed to update yabridge: {:#}",
                                                e
                                            ),
                                            is_error: true,
                                        },
                                    }
                                });
                            }
                        });
                        ui.add_space(4.0);
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Sync Plugins").clicked() && !self.is_working {
                            match self.wine_install.clone() {
                                Some(wine_install) => {
                                    let wine_bin = wine_install.path.join("wine");
                                    self.run_bg("Syncing plugins...", move || {
                                        match yabridge::run_sync(&wine_bin) {
                                            Ok(result) => BgTaskResult {
                                                message: format!(
                                                    "Sync finished: {} plugins found/synced.\n{}",
                                                    result.plugins_synced,
                                                    result.output
                                                ),
                                                is_error: !result.success,
                                            },
                                            Err(e) => BgTaskResult {
                                                message: format!(
                                                    "Sync failed: {:#}",
                                                    e
                                                ),
                                                is_error: true,
                                            },
                                        }
                                    });
                                }
                                None => {
                                    self.is_error = true;
                                    self.status_message =
                                        "No Wine installation selected, cannot sync."
                                            .to_string();
                                }
                            }
                        }

                        if ui.button("Refresh").clicked() {
                            self.refresh();
                        }
                    });
                }
                None => {
                    ui.label("yabridge is not installed.");

                    let bundled = yabridge::find_bundled_yabridge_dir();

                    match &bundled {
                        Some(path) => {
                            ui.label(format!("Bundled binaries found at: {}", path.display()));
                            ui.add_space(4.0);

                            if ui.button("Install yabridge (custom build)").clicked()
                                && !self.is_working
                            {
                                let data_dir = dirs::home_dir()
                                    .unwrap_or_default()
                                    .join(".local/share/yabridge");
                                let build_dir = path.clone();
                                self.run_bg("Installing yabridge...", move || {
                                    match yabridge::copy_binaries(&data_dir, &build_dir) {
                                        Ok(_) => BgTaskResult {
                                            message: format!(
                                                "yabridge installed successfully in {}.",
                                                data_dir.display()
                                            ),
                                            is_error: false,
                                        },
                                        Err(e) => BgTaskResult {
                                            message: format!(
                                                "Failed to install yabridge: {:#}",
                                                e
                                            ),
                                            is_error: true,
                                        },
                                    }
                                });
                            }
                        }
                        None => {
                            ui.colored_label(
                                egui::Color32::RED,
                                "Bundled yabridge binaries not found. Please reinstall yabridge-manager.",
                            );
                        }
                    }
                }
            }
        });
    }

    fn show_status(&self, ui: &mut egui::Ui) {
        if !self.status_message.is_empty() {
            ui.separator();
            if self.is_error {
                ui.colored_label(egui::Color32::RED, &self.status_message);
            } else {
                ui.label(&self.status_message);
            }
        }
    }
}
