use super::components::centered_text;
use eframe::egui::{Response, Ui, Widget};

pub struct GameMainTab {}

impl Widget for &mut GameMainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| centered_text(ui, "Hello")).response
    }
}
