use eframe::egui::{Response, Ui, Widget};

use super::{
    accounts_community_servers_tab::AccountsCommunityServersTab,
    accounts_main_tab::AccountsMainTab, accounts_servers_tab::AccountsServersTab,
};

pub enum AccountsTabContent {
    Main(AccountsMainTab),
    Servers(AccountsServersTab),
    CommunityServers(AccountsCommunityServersTab),
}

pub struct AccountsTab {
    pub tabs: Vec<AccountsTabContent>,
    pub selected_tab: usize,
}

impl Widget for &mut AccountsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        AccountsTabContent::Main(_) => "Accounts",
                        AccountsTabContent::Servers(_) => "Servers",
                        AccountsTabContent::CommunityServers(_) => "Community Servers",
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
                    AccountsTabContent::Main(tab) => {
                        ui.add(tab);
                    }
                    AccountsTabContent::Servers(tab) => {
                        ui.add(tab);
                    }
                    AccountsTabContent::CommunityServers(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}
