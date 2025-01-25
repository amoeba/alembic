use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};

use crate::backend::Backend;

use super::components::centered_text;

pub struct DeveloperNetworkIncomingTab {
    selected_item: Option<usize>,
    left_panel_width: f32,
}

impl Default for DeveloperNetworkIncomingTab {
    fn default() -> DeveloperNetworkIncomingTab {
        Self {
            selected_item: None,
            left_panel_width: 200.0,
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
                    egui::SidePanel::left("left_panel")
                        .resizable(true)
                        .default_width(self.left_panel_width)
                        .show_inside(ui, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (index, item) in
                                    backend.lock().unwrap().packets_incoming.iter().enumerate()
                                {
                                    if ui.button(item.timestamp.to_string()).clicked() {
                                        self.selected_item = Some(index);
                                    }
                                }
                            });
                            self.left_panel_width = ui.available_width();
                        });

                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        if let Some(item) = &self.selected_item {
                            ui.label(format!(
                                "{:?}",
                                backend.lock().unwrap().packets_incoming[*item].data
                            ));
                        } else {
                            centered_text(ui, "Select a packet.");
                        }
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
