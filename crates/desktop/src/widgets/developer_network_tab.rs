use eframe::egui::{Response, Ui, Widget};

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

            ui.separator();

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
