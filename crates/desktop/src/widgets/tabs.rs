use eframe::egui::{Response, Ui, Widget};

use super::{
    developer_logs_tab::DeveloperLogsTab,
    developer_main_tab_::DeveloperMainTab,
    developer_network_incoming_tab::DeveloperNetworkIncomingTab,
    developer_network_outgoing_tab::DeveloperNetworkOutgoingTab,
    developer_network_tab::{DeveloperNetworkTab, DeveloperNetworkTabContent},
    developer_tab::{DeveloperTab, DeveloperTabContent},
    main_tab::MainTab,
};

pub enum TabContent {
    Main(MainTab),
    Developer(DeveloperTab),
}

pub struct TabContainer {
    tabs: Vec<TabContent>,
    selected_tab: usize,
}

impl TabContainer {
    pub fn new() -> Self {
        Self {
            tabs: vec![
                TabContent::Main(MainTab {}),
                TabContent::Developer(DeveloperTab {
                    selected_tab: 0,
                    tabs: vec![
                        DeveloperTabContent::Main(DeveloperMainTab {}),
                        DeveloperTabContent::Network(DeveloperNetworkTab {
                            selected_tab: 0,
                            tabs: vec![
                                DeveloperNetworkTabContent::Incoming(
                                    DeveloperNetworkIncomingTab {},
                                ),
                                DeveloperNetworkTabContent::Outgoing(
                                    DeveloperNetworkOutgoingTab {},
                                ),
                            ],
                        }),
                        DeveloperTabContent::Logs(DeveloperLogsTab {}),
                    ],
                }),
            ],
            selected_tab: 0,
        }
    }
}

impl Widget for &mut TabContainer {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        TabContent::Main(_) => "Settings",
                        TabContent::Developer(_) => "Developer",
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
                    TabContent::Main(tab) => {
                        ui.add(tab);
                    }
                    TabContent::Developer(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}
