use eframe::egui::{self, Response, Ui, Widget};

use crate::application::AppPage;

pub struct WizardStart {}

impl WizardStart {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut WizardStart {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("WizardStart");
            if ui.button("Exit").clicked() {
                ui.memory_mut(|mem| {
                    mem.data
                        .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                });
            }
        })
        .response
    }
}
