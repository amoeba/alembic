use std::sync::{Arc, Mutex};

use super::components::centered_text;
use crate::backend::Backend;
use chrono::{DateTime, Local, TimeZone};
use eframe::egui::{self, Button, Response, ScrollArea, TextEdit, TextStyle, Ui, Widget};
use ringbuffer::RingBuffer;

pub struct GameChatTab {
    current_message: String,
}

impl GameChatTab {
    pub fn default() -> Self {
        Self {
            current_message: "".to_string(),
        }
    }
}

impl Widget for &mut GameChatTab {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            if backend.lock().unwrap().chat_messages.is_empty() {
                centered_text(ui, "No chat messages yet.")
            } else {
                ui.vertical(|ui| {
                    let backend_lock: std::sync::MutexGuard<'_, Backend> = backend.lock().unwrap();

                    let total_rows = backend_lock.chat_messages.len() as usize;
                    let text_style = TextStyle::Body;
                    let row_height = ui.text_style_height(&text_style);
                    let num_rows_to_show = 10;

                    egui::Frame::dark_canvas(ui.style())
                        .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                        .show(ui, |ui| {
                            ScrollArea::vertical()
                                .auto_shrink([false, true])
                                .min_scrolled_height(40.0)
                                .max_height(row_height * num_rows_to_show as f32)
                                .stick_to_bottom(true)
                                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                                    ui.set_min_height(row_height * num_rows_to_show as f32);
                                    for row in row_range {
                                        let datetime: DateTime<Local> = Local
                                            .timestamp_opt(
                                                backend_lock.chat_messages[row].timestamp as i64,
                                                0,
                                            )
                                            .unwrap();

                                        let text = format!(
                                            "[{}] {}",
                                            datetime.format("%Y-%m-%d %H:%M:%S"),
                                            backend_lock.chat_messages[row].text
                                        );
                                        ui.label(text);
                                    }
                                });
                        });

                    ui.horizontal(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut self.current_message)
                                .frame(true)
                                .hint_text("Type to chat... (currently disabled)"),
                        );
                        ui.add_enabled(false, Button::new("Send"))
                            .on_disabled_hover_text("Not hooked up yet. Come back later.");
                    });
                })
                .response
            }
        } else {
            ui.vertical(|ui| centered_text(ui, "Failed to reach application backend."))
                .response
        }
    }
}
