use std::sync::{Arc, Mutex};

use eframe::egui::{self, Align, Response, Ui, Widget};

use crate::backend::Backend;

use super::{
    developer_network_incoming_tab::DeveloperNetworkIncomingTab,
    developer_network_outgoing_tab::DeveloperNetworkOutgoingTab,
};

pub enum DeveloperNetworkTabContent {
    Incoming(DeveloperNetworkIncomingTab),
    Outgoing(DeveloperNetworkOutgoingTab),
}

pub struct DeveloperNetworkTab {
    pub selected_tab: usize,
    pub tabs: Vec<DeveloperNetworkTabContent>,
}

impl Widget for &mut DeveloperNetworkTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Statistics
            if let Some(backend) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
            }) {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::top_down(Align::TOP), |ui| {
                        ui.label("Incoming Packets");
                        ui.label(
                            egui::RichText::new(
                                backend
                                    .lock()
                                    .unwrap()
                                    .statistics
                                    .network
                                    .incoming_count
                                    .to_string(),
                            )
                            .size(32.0),
                        );
                    });
                    ui.with_layout(egui::Layout::top_down(Align::TOP), |ui| {
                        ui.label("Outgoing Packets");
                        ui.label(
                            egui::RichText::new(
                                backend
                                    .lock()
                                    .unwrap()
                                    .statistics
                                    .network
                                    .outgoing_count
                                    .to_string(),
                            )
                            .size(32.0),
                        );
                    });
                });
            } else {
                ui.label("Failed to reach application backend.");
            }

            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        DeveloperNetworkTabContent::Incoming(_) => "Incoming",
                        DeveloperNetworkTabContent::Outgoing(_) => "Outgoing",
                    };

                    if ui
                        .selectable_label(self.selected_tab == index, label)
                        .clicked()
                    {
                        self.selected_tab = index;
                    }
                }
            });

            // Tab contents
            if let Some(tab) = self.tabs.get_mut(self.selected_tab) {
                match tab {
                    DeveloperNetworkTabContent::Incoming(tab) => {
                        ui.add(tab);
                    }
                    DeveloperNetworkTabContent::Outgoing(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}
