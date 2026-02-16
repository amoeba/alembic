use std::sync::{Arc, Mutex};

use eframe::egui::{self, Layout, Response, RichText, Ui, Widget};
use libalembic::inject_config::{DllType, InjectConfig};
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

            let selected_text = if !account_names.is_empty() {
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
                .map(|client| client.name().to_string())
                .collect();

            let selected_text = if !client_names.is_empty() {
                settings
                    .selected_client
                    .and_then(|index| client_names.get(index).cloned())
                    .unwrap_or_else(|| "Pick a client".to_string())
            } else {
                "No clients".to_string()
            };

            let truncated_text = if selected_text.len() > 20 {
                format!("{}...", &selected_text[..17])
            } else {
                selected_text
            };

            egui::ComboBox::from_id_salt("Client")
                .selected_text(truncated_text)
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

            // Get the currently selected client
            if let Some(client_idx) = settings.selected_client {
                // Get DLLs for the selected client
                let dll_names: Vec<String> = settings
                    .get_client_dlls(client_idx)
                    .map(|dlls| dlls.iter().map(|dll| format!("{}", dll.dll_type)).collect())
                    .unwrap_or_default();

                let current_selected_dll =
                    settings
                        .clients
                        .get(client_idx)
                        .and_then(|client| match client {
                            libalembic::settings::ClientConfigType::Windows(c) => c.selected_dll,
                            libalembic::settings::ClientConfigType::Wine(c) => c.selected_dll,
                        });

                let selected_text = if !dll_names.is_empty() {
                    current_selected_dll
                        .and_then(|index| dll_names.get(index).cloned())
                        .unwrap_or_else(|| "No DLL selected".to_string())
                } else {
                    "No DLLs".to_string()
                };

                egui::ComboBox::from_id_salt("DLL")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        // Add option to deselect
                        let mut selected_none = current_selected_dll.is_none();
                        if ui
                            .selectable_value(&mut selected_none, true, "None")
                            .changed()
                        {
                            settings.select_dll_for_client(client_idx, None);
                            let _ = settings.save();
                        };

                        for (index, name) in dll_names.iter().enumerate() {
                            let mut selected = current_selected_dll == Some(index);
                            if ui
                                .selectable_value(&mut selected, true, name.clone())
                                .changed()
                            {
                                settings.select_dll_for_client(client_idx, Some(index));
                                let _ = settings.save();
                            };
                        }
                    })
                    .response
            } else {
                ui.label("Pick a client first".to_string())
            }
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

            let selected_text = if !server_names.is_empty() {
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
                                            if client.is_wine() { "Wine" } else { "Windows" };
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
            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let mut settings = s.lock().unwrap();

                // Get the selected client
                let client_idx = settings.selected_client;
                if client_idx.is_none() {
                    return;
                }

                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                ui.label("DLLs");
                ui.add_space(8.0);

                let client_idx = client_idx.unwrap();

                // Scan button
                ui.horizontal(|ui| {
                    if ui.button("Discover DLLs").clicked() {
                        match libalembic::scanner::scan_for_decal_dlls() {
                            Ok(discovered_dlls) => {
                                if discovered_dlls.is_empty() {
                                    println!("No Decal installations found");
                                } else {
                                    let had_no_dlls = settings
                                        .get_client_dlls(client_idx)
                                        .map(|dlls| dlls.is_empty())
                                        .unwrap_or(true);

                                    for dll in discovered_dlls {
                                        settings.add_dll_to_client(client_idx, dll);
                                    }

                                    // Auto-select first DLL if there were no DLLs before
                                    if had_no_dlls {
                                        settings.select_dll_for_client(client_idx, Some(0));
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

                    if ui.button("New DLL").clicked() {
                        let new_dll = InjectConfig {
                            dll_path: std::path::PathBuf::from(
                                "C:\\Program Files\\Alembic\\Alembic.dll",
                            ),
                            dll_type: DllType::Alembic,
                            startup_function: None,
                        };
                        settings.add_dll_to_client(client_idx, new_dll);
                        let dlls = settings
                            .get_client_dlls(client_idx)
                            .map(|d| d.len())
                            .unwrap_or(0);
                        if dlls == 1 {
                            settings.select_dll_for_client(client_idx, Some(0));
                        }
                        let _ = settings.save();
                    }
                });

                ui.add_space(8.0);

                let dlls_for_client = settings
                    .get_client_dlls(client_idx)
                    .map(|d| d.len())
                    .unwrap_or(0);
                if dlls_for_client == 0 {
                    ui.label("No DLLs configured. Click 'Discover DLLs' to scan, or 'New DLL' to add one manually.");
                } else {
                    let mut to_remove: Option<usize> = None;

                    // Clone dlls and current_selected_dll to avoid borrow conflicts
                    let (dlls_cloned, current_selected_dll) =
                        if let Some(dlls) = settings.get_client_dlls(client_idx) {
                            let selected =
                                settings
                                    .clients
                                    .get(client_idx)
                                    .and_then(|client| match client {
                                        libalembic::settings::ClientConfigType::Windows(c) => {
                                            c.selected_dll
                                        }
                                        libalembic::settings::ClientConfigType::Wine(c) => {
                                            c.selected_dll
                                        }
                                    });
                            (dlls.clone(), selected)
                        } else {
                            (vec![], None)
                        };

                    if !dlls_cloned.is_empty() {
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
                                let mut new_selection: Option<usize> = None;

                                for (index, dll) in dlls_cloned.iter().enumerate() {
                                    let is_selected = current_selected_dll == Some(index);

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
                                            ui.label(dll.dll_path.display().to_string());
                                        });

                                        // Type column
                                        row.col(|ui| {
                                            ui.label(dll.dll_type.to_string());
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
                                    settings.select_dll_for_client(client_idx, Some(index));
                                    let _ = settings.save();
                                }
                            });
                    }

                    // Handle deletion
                    if let Some(index) = to_remove {
                        settings.remove_dll_from_client(client_idx, index);
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
