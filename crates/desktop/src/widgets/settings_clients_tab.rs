use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::{
    client_config::{ClientConfig, ClientConfiguration},
    settings::AlembicSettings,
};

#[cfg(not(target_os = "windows"))]
use libalembic::client_config::WineClientConfig;

#[cfg(target_os = "windows")]
use libalembic::client_config::{WineClientConfig, WindowsClientConfig};

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
                            let new_client = ClientConfig::Wine(WineClientConfig {
                                display_name: "Wine Client".to_string(),
                                install_path: std::path::PathBuf::from("C:\\Turbine\\Asheron's Call"),
                                wine_executable: std::path::PathBuf::from("/usr/local/bin/wine64"),
                                prefix_path: std::path::PathBuf::from("/path/to/prefix"),
                                additional_env: std::collections::HashMap::new(),
                            });

                            settings.clients.push(new_client);
                            let _ = settings.save();
                        }

                        #[cfg(target_os = "windows")]
                        if ui.button("New Windows Client").clicked() {
                            let new_client = ClientConfig::Windows(WindowsClientConfig {
                                display_name: "Windows Client".to_string(),
                                install_path: std::path::PathBuf::from("C:\\Turbine\\Asheron's Call"),
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
                        .column(Column::remainder()) // Install Path
                        .column(Column::auto()) // Delete
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.strong("Type");
                            });
                            header.col(|ui| {
                                ui.strong("Name");
                            });
                            header.col(|ui| {
                                ui.strong("Install Path");
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
                                            ClientConfig::Wine(_) => "Wine",
                                            ClientConfig::Windows(_) => "Windows",
                                        };
                                        ui.label(type_str);
                                    });

                                    // Display Name (editable)
                                    table_row.col(|ui| {
                                        let mut name = settings.clients[i].display_name().to_string();

                                        if ui.text_edit_singleline(&mut name).changed() {
                                            match &mut settings.clients[i] {
                                                ClientConfig::Wine(config) => {
                                                    config.display_name = name;
                                                }
                                                ClientConfig::Windows(config) => {
                                                    config.display_name = name;
                                                }
                                            }
                                            did_update = true;
                                        }
                                    });

                                    // Install Path (editable)
                                    table_row.col(|ui| {
                                        let mut path_string = settings.clients[i].install_path().display().to_string();

                                        if ui.text_edit_singleline(&mut path_string).changed() {
                                            match &mut settings.clients[i] {
                                                ClientConfig::Wine(config) => {
                                                    config.install_path =
                                                        std::path::PathBuf::from(&path_string);
                                                }
                                                ClientConfig::Windows(config) => {
                                                    config.install_path =
                                                        std::path::PathBuf::from(&path_string);
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
