use eframe::egui::{self, Response, Ui, Widget};

pub struct News {
    last_fetched: Option<std::time::SystemTime>,
}

impl News {
    pub fn default() -> Self {
        Self { last_fetched: None }
    }
}
impl Widget for &mut News {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
            .auto_shrink(true)
            .max_height(105.0) // Help last line be cut off to indicate
                                          // the need to scroll
            .show(ui, |ui| {
                ui.heading("Community Updates");

                ui.label("2024/11/18 5:38PM");
                let message = "Have any old hard drives laying around that might have old installs of Asheron's Call, decal plugins, screenshots, or other files that might be otherwise lost to time?  Uploading them to us could help us re-create old landmasses and content that you thought you'd never be able to see again. Or are you just interested in seeing what the community has already put together (old and current emulators, dat files, decal plugins, high res scans of posters and much much more), check out the Asheron’s Call Community Preservation Project today!\n\nBrought to you by Immortalbob and myself.";

                ui.label(message)
            });
        })
        .response
    }
}
