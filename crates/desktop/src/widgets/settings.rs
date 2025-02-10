use eframe::egui::{self, Response, Ui, Widget};

use crate::application::AppPage;

use super::components::{SettingsDLLPathEdit, SettingsGameClientPathEdit};

pub struct Settings {}

impl Settings {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut Settings {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Settings");
            ui.add_space(16.0);
            ui.add(&mut SettingsGameClientPathEdit {});
            ui.add_space(16.0);
            ui.add(&mut SettingsDLLPathEdit {});

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
