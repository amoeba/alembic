use eframe::egui::{Response, Ui, Widget};

pub struct AccountsTab {}

impl Widget for &mut AccountsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("Accounts");
        })
        .response
    }
}
