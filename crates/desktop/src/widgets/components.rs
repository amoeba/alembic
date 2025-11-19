use std::sync::{Arc, Mutex};

use eframe::egui::{self, Layout, Response, RichText, Ui, Widget};
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
                            .selectable_value(&mut settings.selected_dll, Some(index), name.clone())
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
        use egui_extras::{Column, TableBuilder};

        ui.vertical(|ui| {
            ui.label("Game Clients");
            ui.add_space(8.0);

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let mut settings = s.lock().unwrap();
                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                // Scan button
                if ui.button("Scan for Clients").clicked() {
                    match libalembic::scanner::scan_all() {
                        Ok(discovered) => {
                            let had_no_clients = settings.clients.is_empty();
                            let mut added_count = 0;

                            for config in discovered {
                                // Check if already exists
                                let already_exists = settings.clients.iter().any(|existing| {
                                    existing.install_path() == config.install_path()
                                });

                                if !already_exists {
                                    let should_select = had_no_clients && added_count == 0;
                                    settings.add_client(config, should_select);
                                    settings.is_configured = true;
                                    added_count += 1;
                                }
                            }

                            if added_count > 0 {
                                let _ = settings.save();
                                println!("Added {} new client(s)", added_count);
                            } else {
                                println!("No new clients found");
                            }
                        }
                        Err(e) => {
                            eprintln!("Error scanning for clients: {}", e);
                        }
                    }
                }

                ui.add_space(8.0);

                if settings.clients.is_empty() {
                    ui.label(
                        "No clients configured. Click 'Scan for Clients' to discover clients.",
                    );
                } else {
                    let mut to_remove: Option<usize> = None;

                    TableBuilder::new(ui)
                        .id_salt("clients_table")
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto()) // Index
                        .column(Column::remainder()) // Path
                        .column(Column::auto()) // Type
                        .column(Column::auto()) // Delete button
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.strong("#");
                            });
                            header.col(|ui| {
                                ui.strong("Path");
                            });
                            header.col(|ui| {
                                ui.strong("Type");
                            });
                            header.col(|ui| {
                                ui.strong("");
                            });
                        })
                        .body(|mut body| {
                            let selected_client = settings.selected_client;
                            let mut new_selection: Option<usize> = None;

                            for (index, client) in settings.clients.iter().enumerate() {
                                let is_selected = Some(index) == selected_client;

                                body.row(text_height, |mut row| {
                                    // Index column
                                    row.col(|ui| {
                                        let text = if is_selected {
                                            RichText::new(format!("* {}", index)).strong()
                                        } else {
                                            RichText::new(format!("  {}", index))
                                        };
                                        if ui.selectable_label(is_selected, text).clicked() {
                                            new_selection = Some(index);
                                        }
                                    });

                                    // Path column
                                    row.col(|ui| {
                                        ui.label(client.install_path().display().to_string());
                                    });

                                    // Type column
                                    row.col(|ui| {
                                        let client_type =
                                            if client.is_wine() { "wine" } else { "Windows" };
                                        ui.label(client_type);
                                    });

                                    // Delete button column
                                    row.col(|ui| {
                                        if ui.button("Delete").clicked() {
                                            to_remove = Some(index);
                                        }
                                    });
                                });
                            }

                            // Handle selection change
                            if let Some(index) = new_selection {
                                settings.selected_client = Some(index);
                                let _ = settings.save();
                            }
                        });

                    // Handle deletion
                    if let Some(index) = to_remove {
                        settings.clients.remove(index);

                        // Update selected_client if needed
                        if let Some(selected) = settings.selected_client {
                            if selected == index {
                                settings.selected_client = None;
                            } else if selected > index {
                                settings.selected_client = Some(selected - 1);
                            }
                        }

                        let _ = settings.save();
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
        use egui_extras::{Column, TableBuilder};

        ui.vertical(|ui| {
            ui.label("DLLs");
            ui.add_space(8.0);

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let mut settings = s.lock().unwrap();
                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                // Scan button
                if ui.button("Scan for DLLs").clicked() {
                    match libalembic::scanner::scan_for_decal_dlls() {
                        Ok(discovered_dlls) => {
                            if discovered_dlls.is_empty() {
                                println!("No Decal installations found");
                            } else {
                                let had_no_dlls = settings.discovered_dlls.is_empty();

                                for dll in discovered_dlls {
                                    settings.add_or_update_dll(dll);
                                }

                                // Auto-select first DLL if there were no DLLs before
                                if had_no_dlls
                                    && !settings.discovered_dlls.is_empty()
                                    && settings.selected_dll.is_none()
                                {
                                    settings.selected_dll = Some(0);
                                }

                                let _ = settings.save();
                                println!("DLL scan complete");
                            }
                        }
                        Err(e) => {
                            eprintln!("Error scanning for DLLs: {}", e);
                        }
                    }
                }

                ui.add_space(8.0);

                if settings.discovered_dlls.is_empty() {
                    ui.label("No DLLs configured. Click 'Scan for DLLs' to discover DLLs.");
                } else {
                    let mut to_remove: Option<usize> = None;

                    TableBuilder::new(ui)
                        .id_salt("dlls_table")
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto()) // Index
                        .column(Column::remainder()) // Path
                        .column(Column::auto()) // Type
                        .column(Column::auto()) // Delete button
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.strong("#");
                            });
                            header.col(|ui| {
                                ui.strong("Path");
                            });
                            header.col(|ui| {
                                ui.strong("Type");
                            });
                            header.col(|ui| {
                                ui.strong("");
                            });
                        })
                        .body(|mut body| {
                            let selected_dll = settings.selected_dll;
                            let mut new_selection: Option<usize> = None;

                            for (index, dll) in settings.discovered_dlls.iter().enumerate() {
                                let is_selected = Some(index) == selected_dll;

                                body.row(text_height, |mut row| {
                                    // Index column
                                    row.col(|ui| {
                                        let text = if is_selected {
                                            RichText::new(format!("* {}", index)).strong()
                                        } else {
                                            RichText::new(format!("  {}", index))
                                        };
                                        if ui.selectable_label(is_selected, text).clicked() {
                                            new_selection = Some(index);
                                        }
                                    });

                                    // Path column
                                    row.col(|ui| {
                                        ui.label(dll.dll_path().display().to_string());
                                    });

                                    // Type column
                                    row.col(|ui| {
                                        let dll_variant = match dll {
                                            libalembic::client_config::InjectConfig::Wine(_) => {
                                                "wine"
                                            }
                                            libalembic::client_config::InjectConfig::Windows(_) => {
                                                "Windows"
                                            }
                                        };
                                        ui.label(dll_variant);
                                    });

                                    // Delete button column
                                    row.col(|ui| {
                                        if ui.button("Delete").clicked() {
                                            to_remove = Some(index);
                                        }
                                    });
                                });
                            }

                            // Handle selection change
                            if let Some(index) = new_selection {
                                settings.selected_dll = Some(index);
                                let _ = settings.save();
                            }
                        });

                    // Handle deletion
                    if let Some(index) = to_remove {
                        settings.discovered_dlls.remove(index);

                        // Update selected_dll if needed
                        if let Some(selected) = settings.selected_dll {
                            if selected == index {
                                settings.selected_dll = None;
                            } else if selected > index {
                                settings.selected_dll = Some(selected - 1);
                            }
                        }

                        let _ = settings.save();
                    }
                }
            } else {
                ui.label("Failed to get settings.");
            }
        })
        .response
    }
}
