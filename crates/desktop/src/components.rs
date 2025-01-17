use eframe::egui::{self, Layout, Ui};

pub fn centered_text(ui: &mut Ui, text: &str) {
    ui.with_layout(
        Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| ui.label(text),
    );
}
