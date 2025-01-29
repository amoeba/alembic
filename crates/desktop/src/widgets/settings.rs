use eframe::egui::{self, Response, Ui, Widget};

use crate::application::AppPage;

pub struct Settings {}

impl Settings {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut Settings {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("Settings");
            if ui.button("next").clicked() {
                ui.memory_mut(|mem| {
                    mem.data
                        .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                });
            }
        })
        .response
    }
}
