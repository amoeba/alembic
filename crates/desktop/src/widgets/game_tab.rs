use eframe::egui::{Response, Ui, Widget};

use super::{game_chat_tab::GameChatTab, game_main_tab::GameMainTab};

pub enum GameTabContent {
    Main(GameMainTab),
    Chat(GameChatTab),
}

pub struct GameTab {
    pub tabs: Vec<GameTabContent>,
    pub selected_tab: usize,
}

impl Widget for &mut GameTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        GameTabContent::Main(_) => "Main",
                        GameTabContent::Chat(_) => "Chat",
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
                    GameTabContent::Main(tab) => {
                        ui.add(tab);
                    }
                    GameTabContent::Chat(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}
