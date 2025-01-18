use eframe::egui::{Response, Ui, Widget};

pub struct DeveloperNetworkIncomingTab {}

impl Widget for &mut DeveloperNetworkIncomingTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.group(|ui| ui.label("NetworkIncoming")).response
    }
}
