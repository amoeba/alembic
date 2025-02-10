use eframe::egui::{self, Align, Layout, Response, Ui, Widget};

use crate::application::AppPage;

pub struct About {}

impl About {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut About {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("../../assets/logo.png"))
                            .max_width(32.0),
                    );
                    ui.heading("About Alembic");
                });
                ui.label("Version 0.1.0");
                ui.label("Copyright © 2025 Bryce Mecum");
                ui.add_space(8.0);
                use egui::special_emojis::GITHUB;
                ui.hyperlink_to(
                    format!("{GITHUB} alembic on GitHub"),
                    "https://github.com/amoeba/alembic",
                );
                ui.add_space(16.0);
                if ui.button("Close").clicked() {
                    ui.memory_mut(|mem| {
                        mem.data
                            .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                    });
                }
            });
        })
        .response
    }
}
