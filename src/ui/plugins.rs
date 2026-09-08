use std::sync::mpsc::Receiver;

use eframe::egui;
use crate::backend::plugin::{self, Plugin, PluginFormat, PluginScanResult};

#[derive(Default)]
pub struct PluginsPage {
    pub scan_result: Option<PluginScanResult>,
    pub filter_text: String,
    pub filter_format: Option<PluginFormat>,
    pub selected_plugin: Option<usize>,
    pub scan_dirs: Vec<std::path::PathBuf>,
    pub is_scanning: bool,
    scan_rx: Option<Receiver<PluginScanResult>>,
}

impl PluginsPage {
    pub fn new() -> Self {
        Self {
            scan_dirs: plugin::default_plugin_dirs(),
            ..Default::default()
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Plugin Manager");
        ui.separator();

        // Pick up a finished background scan without blocking the UI.
        if self.scan_rx.is_some() {
            ui.ctx().request_repaint();
        }
        let mut finished: Option<PluginScanResult> = None;
        let mut scan_ended = false;
        if let Some(rx) = &self.scan_rx {
            match rx.try_recv() {
                Ok(result) => {
                    finished = Some(result);
                    scan_ended = true;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    scan_ended = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        if scan_ended {
            self.scan_rx = None;
            self.is_scanning = false;
            if let Some(result) = finished {
                self.scan_result = Some(result);
            }
        }

        self.show_toolbar(ui);
        ui.add_space(8.0);
        self.show_plugin_list(ui);
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);

            ui.separator();

            ui.label("Format:");
            let formats = vec![
                (None, "All"),
                (Some(PluginFormat::Vst2), "VST2"),
                (Some(PluginFormat::Vst3), "VST3"),
                (Some(PluginFormat::Clap), "CLAP"),
            ];

            for (fmt, label) in &formats {
                let is_selected = self.filter_format == *fmt;
                if ui.selectable_label(is_selected, *label).clicked() {
                    self.filter_format = *fmt;
                }
            }

            ui.separator();

            if ui.button("Scan Plugins").clicked() && !self.is_scanning {
                self.is_scanning = true;
                let dirs = self.scan_dirs.clone();
                let (tx, rx) = std::sync::mpsc::channel();
                self.scan_rx = Some(rx);
                std::thread::spawn(move || {
                    let result = plugin::scan_plugins(&dirs);
                    let _ = tx.send(result);
                });
            }

            if ui.button("Refresh").clicked() {
                let dirs = self.scan_dirs.clone();
                self.scan_result = Some(plugin::scan_plugins(&dirs));
            }
        });

        ui.horizontal(|ui| {
            ui.label("Scan directories:");

            if ui.button("+ Add Dir").clicked() {
                if let Some(dir) = rfd::FileDialog::new()
                    .set_title("Select Plugin Directory")
                    .pick_folder()
                {
                    self.scan_dirs.push(dir);
                }
            }

            if !self.scan_dirs.is_empty() {
                ui.label(format!("{} directories configured", self.scan_dirs.len()));
            }
        });

        if let Some(ref result) = self.scan_result {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Found {} plugins ({:.1} MB)",
                    result.plugins.len(),
                    result.total_size_kb as f64 / 1024.0
                ));
            });
        }
    }

    fn show_plugin_list(&mut self, ui: &mut egui::Ui) {
        let plugins = match &self.scan_result {
            Some(result) => &result.plugins,
            None => {
                ui.centered_and_justified(|ui| {
                    ui.label("Click 'Scan Plugins' to discover installed plugins.");
                });
                return;
            }
        };

        let filtered: Vec<(usize, &Plugin)> = plugins
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                let text_match = self.filter_text.is_empty()
                    || p.name.to_lowercase().contains(&self.filter_text.to_lowercase())
                    || p.vendor.to_lowercase().contains(&self.filter_text.to_lowercase());

                let format_match = match &self.filter_format {
                    Some(f) => p.format == *f,
                    None => true,
                };

                text_match && format_match
            })
            .collect();

        ui.label(format!("Showing {} of {} plugins", filtered.len(), plugins.len()));

        ui.add_space(4.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("plugin_grid")
                .num_columns(5)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Name").strong());
                    ui.label(egui::RichText::new("Format").strong());
                    ui.label(egui::RichText::new("Vendor").strong());
                    ui.label(egui::RichText::new("Size").strong());
                    ui.label(egui::RichText::new("Status").strong());
                    ui.end_row();

                    for (idx, plugin) in &filtered {
                        let is_selected = self.selected_plugin == Some(*idx);

                        if ui.selectable_label(is_selected, &plugin.name).clicked() {
                            self.selected_plugin = Some(*idx);
                        }

                        let format_color = match plugin.format {
                            PluginFormat::Vst2 => egui::Color32::from_rgb(100, 200, 255),
                            PluginFormat::Vst3 => egui::Color32::from_rgb(100, 255, 150),
                            PluginFormat::Clap => egui::Color32::from_rgb(255, 200, 100),
                        };
                        ui.colored_label(format_color, plugin.format.to_string());
                        ui.label(&plugin.vendor);
                        ui.label(format!("{:.1} MB", plugin.size_kb as f64 / 1024.0));

                        let status_color = if plugin.enabled {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };
                        ui.colored_label(status_color, if plugin.enabled { "Active" } else { "Disabled" });
                        ui.end_row();
                    }
                });
        });
    }
}
