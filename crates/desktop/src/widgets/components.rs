use std::sync::{Arc, Mutex};

use eframe::egui::{self, Layout, Response, Ui, Widget};
use libalembic::settings::AlembicSettings;

pub fn centered_text(ui: &mut Ui, text: &str) {
    ui.with_layout(
        Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| ui.label(text),
    );
}

pub struct AccountPicker {
    pub selected_server: Option<usize>,
}

impl Widget for &mut AccountPicker {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(s) = ui.data_mut(|data| {
            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
        }) {
            let mut settings = s.lock().unwrap();

            let account_names: Vec<String> = settings
                .accounts
                .iter()
                .filter(|account| {
                    self.selected_server.is_some()
                        && account.server_index == self.selected_server.unwrap()
                })
                .map(|account| account.name.clone())
                .collect();

            let selected_text = if account_names.len() > 0 {
                settings
                    .selected_account
                    .and_then(|index| account_names.get(index).cloned())
                    .unwrap_or_else(|| "Pick an account".to_string())
            } else {
                "Pick a server".to_string()
            };

            egui::ComboBox::from_id_salt("Account")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (index, name) in account_names.iter().enumerate() {
                        ui.selectable_value(
                            &mut settings.selected_account,
                            Some(index),
                            name.clone(),
                        );
                    }
                })
                .response
        } else {
            ui.label("TODO: Bug, please report.")
        }
    }
}

pub struct ServerPicker {}

impl Widget for &mut ServerPicker {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(s) = ui.data_mut(|data| {
            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
        }) {
            let mut settings = s.lock().unwrap();

            let server_names: Vec<String> = settings
                .servers
                .iter()
                .map(|server| server.hostname.clone())
                .collect();

            let selected_text = if server_names.len() > 0 {
                settings
                    .selected_account
                    .and_then(|index| server_names.get(index).cloned())
                    .unwrap_or_else(|| "Pick a server".to_string())
            } else {
                "No servers".to_string()
            };

            egui::ComboBox::from_id_salt("Server")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (index, name) in server_names.iter().enumerate() {
                        ui.selectable_value(
                            &mut settings.selected_server,
                            Some(index),
                            name.clone(),
                        );
                    }
                })
                .response
        } else {
            ui.label("TODO: Bug, please report.")
        }
    }
}
