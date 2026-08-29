use egui::Ui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    Setup,
    Plugins,
    Config,
    Diagnostics,
}

impl Page {
    pub fn label(&self) -> &str {
        match self {
            Page::Setup => "Setup",
            Page::Plugins => "Plugins",
            Page::Config => "Config",
            Page::Diagnostics => "Diagnostics",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Page::Setup => "\u{2699}",
            Page::Plugins => "\u{1F3B5}",
            Page::Config => "\u{1F4C1}",
            Page::Diagnostics => "\u{1F50D}",
        }
    }
}

pub fn show(ui: &mut Ui, current: &mut Page) {
    ui.vertical(|ui| {
        ui.add_space(8.0);

        let pages = vec![
            Page::Setup,
            Page::Plugins,
            Page::Config,
            Page::Diagnostics,
        ];

        for page in &pages {
            let is_selected = *current == *page;

            let btn = if is_selected {
                ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} {}", page.icon(), page.label()))
                            .strong()
                            .size(15.0),
                    )
                    .fill(egui::Color32::from_rgb(70, 130, 180))
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
                )
            } else {
                ui.add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} {}", page.icon(), page.label()))
                            .size(15.0),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
                )
            };

            if btn.clicked() {
                *current = *page;
            }

            ui.add_space(2.0);
        }
    });
}
