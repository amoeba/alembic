use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, ScrollArea, Ui, Widget};

use crate::backend::Backend;

use super::components::centered_text;

pub struct DeveloperNetworkIncomingTab {
    selected_packet: Option<usize>,
}

impl Default for DeveloperNetworkIncomingTab {
    fn default() -> DeveloperNetworkIncomingTab {
        Self {
            selected_packet: None,
        }
    }
}
impl Widget for &mut DeveloperNetworkIncomingTab {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            ui.vertical(|ui| {
                if backend.lock().unwrap().packets_incoming.len() <= 0 {
                    centered_text(ui, "No incoming packets yet.");
                } else {
                    // TODO: Use show_rows() here too
                    ui.columns(2, |columns| {
                        columns[0].vertical(|ui| {
                            ScrollArea::vertical().show(ui, |ui| {
                                for (index, item) in
                                    backend.lock().unwrap().packets_incoming.iter().enumerate()
                                {
                                    if ui.button(item.timestamp.to_string()).clicked() {
                                        self.selected_packet = Some(index);
                                    }
                                }
                            });
                        });

                        columns[1].vertical(|ui| {
                            if let Some(index) = self.selected_packet {
                                ui.label(format!(
                                    "{:?}",
                                    backend.lock().unwrap().packets_incoming[index].data
                                ));
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
