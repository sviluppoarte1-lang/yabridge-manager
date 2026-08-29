use eframe::egui;

use crate::ui::sidebar::{self, Page};
use crate::ui::setup::SetupPage;
use crate::ui::plugins::PluginsPage;
use crate::ui::config::ConfigPage;
use crate::ui::diagnostics::DiagnosticsPage;

pub struct App {
    pub current_page: Page,
    pub setup: SetupPage,
    pub plugins: PluginsPage,
    pub config: ConfigPage,
    pub diagnostics: DiagnosticsPage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_page: Page::Setup,
            setup: SetupPage::new(),
            plugins: PluginsPage::new(),
            config: ConfigPage::new(),
            diagnostics: DiagnosticsPage::new(),
        }
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        app.setup.refresh();
        app.config.refresh();
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("sidebar")
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.heading("yabridge");
                    ui.label(egui::RichText::new("Wine Plugin Manager").small().italics());
                    ui.add_space(16.0);
                    sidebar::show(ui, &mut self.current_page);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Setup => self.setup.show(ui),
                Page::Plugins => self.plugins.show(ui),
                Page::Config => self.config.show(ui),
                Page::Diagnostics => self.diagnostics.show(ui),
            }
        });
    }
}
