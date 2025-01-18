use eframe::egui::{Response, Ui, Widget};

pub struct DeveloperNetworkOutgoingTab {}

impl Widget for &mut DeveloperNetworkOutgoingTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.group(|ui| ui.label("NetworkOutgoing")).response
    }
}
