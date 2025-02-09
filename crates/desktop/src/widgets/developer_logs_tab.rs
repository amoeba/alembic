use std::sync::{Arc, Mutex};

use super::components::centered_text;
use crate::backend::Backend;
use eframe::egui::{self, Response, ScrollArea, TextStyle, Ui, Widget};
use ringbuffer::RingBuffer;

pub struct DeveloperLogsTab {}

impl Widget for &mut DeveloperLogsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            ui.vertical(|ui| {
                if backend.lock().unwrap().logs.is_empty() {
                    centered_text(ui, "No logs yet.");
                } else {
                    let n_logs = backend.lock().unwrap().logs.len();
                    let text_style = TextStyle::Body;
                    let total_rows = ui.text_style_height(&text_style);

                    egui::Frame::dark_canvas(ui.style())
                        .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                        .show(ui, |ui| {
                            ScrollArea::vertical()
                                .auto_shrink(false)
                                .stick_to_bottom(true)
                                .show_rows(ui, total_rows, n_logs, |ui, row_range| {
                                    for row in row_range {
                                        ui.label(format!("{}", backend.lock().unwrap().logs[row]));
                                    }
                                });
                        });
                }
            })
            .response
        } else {
            ui.vertical(|ui| centered_text(ui, "Failed to reach application backend."))
                .response
        }
    }
}
