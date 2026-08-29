use eframe::egui;
use crate::backend::wine;

#[derive(Default)]
pub struct ConfigPage {
    pub wine_bin_path: Option<std::path::PathBuf>,
    pub prefix_path: Option<std::path::PathBuf>,
    pub graphics_driver: String,
    pub custom_scan_dirs: Vec<String>,
    pub new_scan_dir: String,
    pub status_message: String,
    pub is_error: bool,
}

impl ConfigPage {
    pub fn new() -> Self {
        Self {
            graphics_driver: "x11,wayland".to_string(),
            ..Default::default()
        }
    }

    pub fn refresh(&mut self) {
        let home = dirs::home_dir().unwrap_or_default();
        self.prefix_path = Some(home.join(".wine"));

        if let Some(ref prefix) = self.prefix_path {
            if let Ok(config) = wine::get_wine_config(
                &std::path::PathBuf::from("/opt/wine-11.16/bin/wine"),
                prefix,
            ) {
                self.graphics_driver = config.graphics_driver;
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");
        ui.separator();

        if self.prefix_path.is_none() {
            self.refresh();
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_wine_config(ui);
            ui.add_space(12.0);
            self.show_scan_dirs(ui);
            ui.add_space(12.0);
            self.show_status(ui);
        });
    }

    fn show_wine_config(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Wine Settings", |ui| {
            egui::Grid::new("wine_config_grid").show(ui, |ui| {
                ui.label("Wine Prefix:");
                ui.label(
                    self.prefix_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "~/.wine".to_string()),
                );
                ui.end_row();

                ui.label("Graphics Driver:");
                egui::ComboBox::from_id_salt("graphics_driver")
                    .selected_text(&self.graphics_driver)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.graphics_driver, "x11".to_string(), "X11");
                        ui.selectable_value(&mut self.graphics_driver, "x11,wayland".to_string(), "X11 + Wayland (Recommended)");
                        ui.selectable_value(&mut self.graphics_driver, "wayland".to_string(), "Wayland Only");
                    });
                ui.end_row();
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Apply Graphics Driver").clicked() {
                    if let Some(ref prefix) = self.prefix_path {
                        let wine_bin = std::path::PathBuf::from("/opt/wine-11.16/bin/wine");
                        match wine::set_graphics_driver(&wine_bin, prefix, &self.graphics_driver) {
                            Ok(()) => {
                                self.status_message = format!(
                                    "Graphics driver set to '{}'. Restart Wine applications to apply.",
                                    self.graphics_driver
                                );
                                self.is_error = false;
                            }
                            Err(e) => {
                                self.status_message = format!("Failed: {}", e);
                                self.is_error = true;
                            }
                        }
                    }
                }

                if ui.button("Reset to Default").clicked() {
                    self.graphics_driver = "x11,wayland".to_string();
                }
            });

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Tip: 'X11 + Wayland' lets Wine choose the best driver.").italics());
        });
    }

    fn show_scan_dirs(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Plugin Scan Directories", |ui| {
            let mut to_remove = None;
            for (i, dir) in self.custom_scan_dirs.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(dir);
                    if ui.button("Remove").clicked() {
                        to_remove = Some(i);
                    }
                });
            }

            if let Some(idx) = to_remove {
                self.custom_scan_dirs.remove(idx);
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_scan_dir);

                if ui.button("Add").clicked() && !self.new_scan_dir.is_empty() {
                    self.custom_scan_dirs.push(self.new_scan_dir.clone());
                    self.new_scan_dir.clear();
                }

                if ui.button("Browse...").clicked() {
                    if let Some(dir) = rfd::FileDialog::new()
                        .set_title("Select Plugin Directory")
                        .pick_folder()
                    {
                        self.custom_scan_dirs.push(dir.display().to_string());
                    }
                }
            });
        });
    }

    fn show_status(&self, ui: &mut egui::Ui) {
        if !self.status_message.is_empty() {
            ui.separator();
            if self.is_error {
                ui.colored_label(egui::Color32::RED, &self.status_message);
            } else {
                ui.colored_label(egui::Color32::GREEN, &self.status_message);
            }
        }
    }
}
