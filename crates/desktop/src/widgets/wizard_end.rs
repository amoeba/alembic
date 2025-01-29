use eframe::egui::{self, Response, Ui, Widget};

use crate::application::AppPage;

pub struct WizardEnd {}

impl WizardEnd {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut WizardEnd {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("WizardEnd");
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
