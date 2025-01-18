use std::sync::{Arc, Mutex};

use super::components::centered_text;
use crate::backend::Backend;
use eframe::egui::{self, Response, ScrollArea, TextStyle, Ui, Widget};

pub struct DeveloperLogsTab {}

impl Widget for &mut DeveloperLogsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            ui.group(|ui| {
                if backend.lock().unwrap().logs.len() <= 0 {
                    centered_text(ui, "No logs yet.");
                } else {
                    let n_logs = backend.lock().unwrap().logs.len();
                    let text_style = TextStyle::Body;
                    let total_rows = ui.text_style_height(&text_style);

                    ui.vertical(|ui| {
                        ScrollArea::vertical().auto_shrink(false).show_rows(
                            ui,
                            total_rows,
                            n_logs,
                            |ui, row_range| {
                                for row in row_range {
                                    let text =
                                        format!("{}", backend.lock().unwrap().logs[row].message);
                                    ui.label(text);
                                }
                            },
                        );
                    });
                }
            })
            .response
        } else {
            ui.group(|ui| centered_text(ui, "Failed to reach application backend."))
                .response
        }
    }
}
