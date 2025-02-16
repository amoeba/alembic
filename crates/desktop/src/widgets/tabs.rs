use eframe::egui::{Response, Ui, Widget};

use super::{
    accounts_community_servers_tab::AccountsCommunityServersTab,
    accounts_main_tab::AccountsMainTab,
    accounts_servers_tab::AccountsServersTab,
    accounts_tab::{AccountsTab, AccountsTabContent},
    developer_logs_tab::DeveloperLogsTab,
    developer_main_tab_::DeveloperMainTab,
    developer_network_incoming_tab::DeveloperNetworkIncomingTab,
    developer_network_outgoing_tab::DeveloperNetworkOutgoingTab,
    developer_network_tab::{DeveloperNetworkTab, DeveloperNetworkTabContent},
    developer_tab::{DeveloperTab, DeveloperTabContent},
    game_chat_tab::GameChatTab,
    game_main_tab::GameMainTab,
    game_tab::{GameTab, GameTabContent},
    main_tab::MainTab,
};

pub enum TabContent {
    Main(MainTab),
    Accounts(AccountsTab),
    Game(GameTab),
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
                TabContent::Main(MainTab::default()),
                TabContent::Accounts(AccountsTab {
                    selected_tab: 0,
                    tabs: vec![
                        AccountsTabContent::Main(AccountsMainTab {
                            selected_server: None,
                        }),
                        AccountsTabContent::Servers(AccountsServersTab {}),
                        AccountsTabContent::CommunityServers(AccountsCommunityServersTab::default()),
                    ],
                }),
                TabContent::Game(GameTab {
                    selected_tab: 0,
                    tabs: vec![
                        GameTabContent::Main(GameMainTab {}),
                        GameTabContent::Chat(GameChatTab {}),
                    ],
                }),
                TabContent::Developer(DeveloperTab {
                    selected_tab: 0,
                    tabs: vec![
                        DeveloperTabContent::Main(DeveloperMainTab {}),
                        DeveloperTabContent::Network(DeveloperNetworkTab {
                            selected_tab: 0,
                            tabs: vec![
                                DeveloperNetworkTabContent::Incoming(
                                    DeveloperNetworkIncomingTab::default(),
                                ),
                                DeveloperNetworkTabContent::Outgoing(
                                    DeveloperNetworkOutgoingTab::default(),
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
                        TabContent::Main(_) => "Main",
                        TabContent::Accounts(_) => "Accounts",
                        TabContent::Game(_) => "Game",
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
                    TabContent::Accounts(tab) => {
                        ui.add(tab);
                    }
                    TabContent::Game(tab) => {
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
