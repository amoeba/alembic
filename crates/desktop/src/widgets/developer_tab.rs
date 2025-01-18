use eframe::egui::{Response, Ui, Widget};

use super::{
    developer_logs_tab::DeveloperLogsTab, developer_main_tab_::DeveloperMainTab,
    developer_network_tab::DeveloperNetworkTab,
};

pub enum DeveloperTabContent {
    Main(DeveloperMainTab),
    Network(DeveloperNetworkTab),
    Logs(DeveloperLogsTab),
}

pub struct DeveloperTab {
    pub tabs: Vec<DeveloperTabContent>,
    pub selected_tab: usize,
}

impl Widget for &mut DeveloperTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        DeveloperTabContent::Main(_) => "Main",
                        DeveloperTabContent::Network(_) => "Network",
                        DeveloperTabContent::Logs(_) => "Logs",
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
                    DeveloperTabContent::Main(tab) => {
                        ui.add(tab);
                    }
                    DeveloperTabContent::Network(tab) => {
                        ui.add(tab);
                    }
                    DeveloperTabContent::Logs(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}
