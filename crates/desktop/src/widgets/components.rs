use std::{
    fs,
    sync::{Arc, Mutex},
};

use eframe::egui::{self, Color32, Layout, Response, RichText, Ui, Widget};
use libalembic::settings::AlembicSettings;

pub fn centered_text(ui: &mut Ui, text: &str) -> Response {
    ui.with_layout(
        Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| ui.label(text),
    )
    .response
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
                .map(|account| account.username.clone())
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
                        if ui
                            .selectable_value(
                                &mut settings.selected_account,
                                Some(index),
                                name.clone(),
                            )
                            .changed()
                        {
                            let _ = settings.save();
                        };
                    }
                })
                .response
        } else {
            ui.label("TODO: Bug, please report.")
        }
    }
}

pub struct ClientPicker {}

impl Widget for &mut ClientPicker {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(s) = ui.data_mut(|data| {
            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
        }) {
            let mut settings = s.lock().unwrap();

            let client_names: Vec<String> = settings
                .clients
                .iter()
                .map(|client| client.display_name().to_string())
                .collect();

            let selected_text = if client_names.len() > 0 {
                settings
                    .selected_client
                    .and_then(|index| client_names.get(index).cloned())
                    .unwrap_or_else(|| "Pick a client".to_string())
            } else {
                "No clients".to_string()
            };

            egui::ComboBox::from_id_salt("Client")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (index, name) in client_names.iter().enumerate() {
                        if ui
                            .selectable_value(
                                &mut settings.selected_client,
                                Some(index),
                                name.clone(),
                            )
                            .changed()
                        {
                            let _ = settings.save();
                        };
                    }
                })
                .response
        } else {
            ui.label("TODO: Bug, please report.")
        }
    }
}

pub struct DllPicker {}

impl Widget for &mut DllPicker {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(s) = ui.data_mut(|data| {
            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
        }) {
            let mut settings = s.lock().unwrap();

            let dll_names: Vec<String> = settings
                .discovered_dlls
                .iter()
                .map(|dll| format!("{}", dll.dll_type()))
                .collect();

            let selected_text = if dll_names.len() > 0 {
                settings
                    .selected_dll
                    .and_then(|index| dll_names.get(index).cloned())
                    .unwrap_or_else(|| "Pick a DLL".to_string())
            } else {
                "No DLLs".to_string()
            };

            egui::ComboBox::from_id_salt("DLL")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (index, name) in dll_names.iter().enumerate() {
                        if ui
                            .selectable_value(
                                &mut settings.selected_dll,
                                Some(index),
                                name.clone(),
                            )
                            .changed()
                        {
                            let _ = settings.save();
                        };
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
                .map(|server| server.name.clone())
                .collect();

            let selected_text = if server_names.len() > 0 {
                settings
                    .selected_server
                    .and_then(|index| server_names.get(index).cloned())
                    .unwrap_or_else(|| "Pick a server".to_string())
            } else {
                "No servers".to_string()
            };

            egui::ComboBox::from_id_salt("Server")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (index, name) in server_names.iter().enumerate() {
                        if ui
                            .selectable_value(
                                &mut settings.selected_server,
                                Some(index),
                                name.clone(),
                            )
                            .changed()
                        {
                            let _ = settings.save();
                        };
                    }
                })
                .response
        } else {
            ui.label("TODO: Bug, please report.")
        }
    }
}

pub struct SettingsGameClientPathEdit {}

impl Widget for &mut SettingsGameClientPathEdit {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("Game Client Configuration");

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let settings = s.lock().unwrap();

                match settings.selected_client {
                    Some(index) => {
                        if let Some(client) = settings.clients.get(index) {
                            ui.label(format!("Name: {}", client.display_name()));
                            ui.label(format!("Path: {}", client.install_path().display()));
                            ui.label(format!("Type: {}", if client.is_wine() { "Wine" } else { "Windows" }));

                            // Indicator
                            match fs::exists(client.install_path()) {
                                Ok(result) => match result {
                                    true => ui.label(RichText::new("Path exists.").color(Color32::GREEN)),
                                    false => ui.label(
                                        RichText::new("Path does not exist.")
                                            .color(Color32::YELLOW),
                                    ),
                                },
                                Err(_) => ui.label(
                                    RichText::new("Error checking path.")
                                        .color(Color32::RED),
                                ),
                            };
                        } else {
                            ui.label("TODO: Bug, please report.");
                        }
                    }
                    None => {
                        ui.label("No client selected. Use 'client scan' or 'client select' to configure.");
                    }
                }
            } else {
                ui.label("Failed to get settings.");
            }
        })
        .response
    }
}
pub struct SettingsDLLPathEdit {}

impl Widget for &mut SettingsDLLPathEdit {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("DLL Configuration");

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let settings = s.lock().unwrap();

                match settings.selected_dll {
                    Some(index) => {
                        if let Some(dll) = settings.discovered_dlls.get(index) {
                            ui.label(format!("Type: {}", dll.dll_type()));
                            ui.label(format!("Path: {}", dll.dll_path().display()));

                            // Indicator
                            match fs::exists(dll.dll_path()) {
                                Ok(result) => match result {
                                    true => ui.label(RichText::new("Path exists.").color(Color32::GREEN)),
                                    false => ui.label(
                                        RichText::new("Path does not exist.")
                                            .color(Color32::YELLOW),
                                    ),
                                },
                                Err(_) => ui.label(
                                    RichText::new("Error checking path.")
                                        .color(Color32::RED),
                                ),
                            };
                        } else {
                            ui.label("TODO: Bug, please report.");
                        }
                    }
                    None => {
                        ui.label("No DLL selected. Use 'dll scan' or 'dll select' to configure.");
                    }
                }
            } else {
                ui.label("Failed to get settings.");
            }
        })
        .response
    }
}
