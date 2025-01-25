use std::sync::{Arc, Mutex};

use super::components::centered_text;
use crate::backend::Backend;
use eframe::egui::{self, Response, ScrollArea, TextStyle, Ui, Widget};

pub struct GameChatTab {}

impl Widget for &mut GameChatTab {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            ui.vertical(|ui| {
                if backend.lock().unwrap().chat_messages.len() <= 0 {
                    centered_text(ui, "No chat messages yet.");
                } else {
                    let n_messages = backend.lock().unwrap().chat_messages.len();
                    let text_style = TextStyle::Body;
                    let total_rows = ui.text_style_height(&text_style);

                    egui::Frame::dark_canvas(ui.style())
                        .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                        .show(ui, |ui| {
                            ScrollArea::vertical().auto_shrink(false).show_rows(
                                ui,
                                total_rows,
                                n_messages,
                                |ui, row_range| {
                                    for row in row_range {
                                        let text = format!(
                                            "{}",
                                            backend.lock().unwrap().chat_messages[row].text
                                        );
                                        ui.label(text);
                                    }
                                },
                            );
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
