use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::{
    client_config::{LaunchCommand, WineClientConfig},
    settings::{AlembicSettings, ClientConfigType},
};

use super::components::centered_text;

pub struct SettingsClientsTab {}

impl Widget for &mut SettingsClientsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Client Installations");

            ui.add_space(8.0);

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let mut settings = s.lock().unwrap();

                ui.vertical(|ui| {
                    // Add buttons for different client types
                    ui.horizontal(|ui| {
                        if ui.button("New Wine Client").clicked() {
                            let mut env = std::collections::HashMap::new();
                            env.insert("WINEPREFIX".to_string(), "/path/to/prefix".to_string());

                            let new_client = ClientConfigType::Wine(WineClientConfig {
                                name: "Wine Client".to_string(),
                                client_path: std::path::PathBuf::from(
                                    "C:\\Turbine\\Asheron's Call\\acclient.exe",
                                ),
                                launch_command: LaunchCommand {
                                    program: std::path::PathBuf::from("/usr/local/bin/wine64"),
                                    args: Vec::new(),
                                    env,
                                },
                            });

                            settings.clients.push(new_client);
                            let _ = settings.save();
                        }

                        #[cfg(target_os = "windows")]
                        if ui.button("New Windows Client").clicked() {
                            let new_client = ClientConfigType::Windows(WindowsClientConfig {
                                name: "Windows Client".to_string(),
                                client_path: std::path::PathBuf::from(
                                    "C:\\Turbine\\Asheron's Call\\acclient.exe",
                                ),
                                env: std::collections::HashMap::new(),
                            });

                            settings.clients.push(new_client);
                            let _ = settings.save();
                        }
                    });

                    ui.add_space(8.0);

                    // Clients listing
                    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                    let mut did_update = false; // Dirty checking for saving settings

                    let mut n_clients = 0;

                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto()) // Type
                        .column(Column::auto()) // Name
                        .column(Column::remainder()) // Client Path
                        .column(Column::auto()) // Delete
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.strong("Type");
                            });
                            header.col(|ui| {
                                ui.strong("Name");
                            });
                            header.col(|ui| {
                                ui.strong("Client Path");
                            });
                            header.col(|ui| {
                                ui.strong("Delete");
                            });
                        })
                        .body(|mut body| {
                            let indices: Vec<usize> = (0..settings.clients.len()).collect();

                            for i in indices {
                                n_clients += 1;

                                body.row(text_height, |mut table_row| {
                                    // Type (non-editable)
                                    table_row.col(|ui| {
                                        let type_str = match &settings.clients[i] {
                                            ClientConfigType::Wine(_) => "Wine",
                                            ClientConfigType::Windows(_) => "Windows",
                                        };
                                        ui.label(type_str);
                                    });

                                    // Display Name (editable)
                                    table_row.col(|ui| {
                                        let current_name = match &settings.clients[i] {
                                            ClientConfigType::Wine(c) => c.name.clone(),
                                            ClientConfigType::Windows(c) => c.name.clone(),
                                        };
                                        let mut name = current_name;

                                        if ui.text_edit_singleline(&mut name).changed() {
                                            match &mut settings.clients[i] {
                                                ClientConfigType::Wine(c) => c.name = name,
                                                ClientConfigType::Windows(c) => c.name = name,
                                            }
                                            did_update = true;
                                        }
                                    });

                                    // Client Path (editable)
                                    table_row.col(|ui| {
                                        let current_path = match &settings.clients[i] {
                                            ClientConfigType::Wine(c) => {
                                                c.client_path.display().to_string()
                                            }
                                            ClientConfigType::Windows(c) => {
                                                c.client_path.display().to_string()
                                            }
                                        };
                                        let mut path_string = current_path;

                                        if ui.text_edit_singleline(&mut path_string).changed() {
                                            match &mut settings.clients[i] {
                                                ClientConfigType::Wine(c) => {
                                                    c.client_path =
                                                        std::path::PathBuf::from(&path_string)
                                                }
                                                ClientConfigType::Windows(c) => {
                                                    c.client_path =
                                                        std::path::PathBuf::from(&path_string)
                                                }
                                            }
                                            did_update = true;
                                        }
                                    });

                                    // Delete button
                                    table_row.col(|ui| {
                                        if ui.button("Delete").clicked() {
                                            settings.clients.remove(i);
                                            did_update = true;
                                        }
                                    });
                                });
                            }
                        });

                    if n_clients == 0 {
                        ui.label("No clients. Click \"New Wine Client\" to add your first one.");
                    }

                    // Save but only if we need to
                    if did_update {
                        let _ = settings.save();
                    }
                })
                .response
            } else {
                ui.group(|ui| centered_text(ui, "Failed to reach application backend."))
                    .response
            }
        })
        .response
    }
}
