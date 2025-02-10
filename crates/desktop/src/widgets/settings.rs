use eframe::egui::{Response, Ui, Widget};

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
        })
        .response
    }
}
