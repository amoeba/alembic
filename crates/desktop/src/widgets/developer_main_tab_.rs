use eframe::egui::{Response, Ui, Widget};

pub struct DeveloperMainTab {}

impl Widget for &mut DeveloperMainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| ui.label("Developer Main")).response
    }
}
