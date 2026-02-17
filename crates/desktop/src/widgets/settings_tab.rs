use eframe::egui::{Response, Ui, Widget};

use super::{settings_clients_tab::SettingsClientsTab, settings_dlls_tab::SettingsDllsTab};

pub enum SettingsTabContent {
    Clients(SettingsClientsTab),
    Dlls(SettingsDllsTab),
}

pub struct SettingsTab {
    pub tabs: Vec<SettingsTabContent>,
    pub selected_tab: usize,
}

impl Widget for &mut SettingsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        SettingsTabContent::Clients(_) => "Clients",
                        SettingsTabContent::Dlls(_) => "DLLs",
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
                    SettingsTabContent::Clients(tab) => {
                        ui.add(tab);
                    }
                    SettingsTabContent::Dlls(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}
