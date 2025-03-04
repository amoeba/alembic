use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::settings::{AlembicSettings, ServerInfo};

use super::components::centered_text;

pub struct AccountsServersTab {}

impl Widget for &mut AccountsServersTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Servers");

            ui.add_space(8.0);

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let mut settings = s.lock().unwrap();

                ui.vertical(|ui| {
                    // Add
                    if ui.button("New Server").clicked() {
                        let new_server = ServerInfo {
                            name: "Server".to_string(),
                            hostname: "hostname or IP address".to_string(),
                            port: "9000".to_string(),
                        };

                        settings.servers.push(new_server);
                        let _ = settings.save();
                    }

                    ui.add_space(8.0);

                    // Servers listing
                    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                    let mut did_update = false; // Dirty checking for saving settings

                    // Easy way to get a count from the above iterator
                    let mut n_accounts = 0;

                    TableBuilder::new(ui)
                        .striped(true) // Enable striped rows for readability
                        .resizable(true) // Allow column resizing
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center)) // Cell layout
                        .column(Column::auto()) // Name column
                        .column(Column::auto()) // Address column
                        .column(Column::auto()) // Port column
                        .column(Column::auto()) // Delete
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.strong("Name");
                            });
                            header.col(|ui| {
                                ui.strong("Address");
                            });
                            header.col(|ui| {
                                ui.strong("Port");
                            });
                            header.col(|ui| {
                                ui.strong("Delete");
                            });
                        })
                        .body(|mut body| {
                            let indices: Vec<usize> = (0..settings.servers.len()).collect();

                            for i in indices {
                                n_accounts += 1;

                                body.row(text_height, |mut table_row| {
                                    // Editable Name field
                                    table_row.col(|ui| {
                                        did_update |= ui
                                            .text_edit_singleline(&mut settings.servers[i].name)
                                            .changed();
                                    });

                                    // Editable Address field
                                    table_row.col(|ui| {
                                        did_update |= ui
                                            .text_edit_singleline(&mut settings.servers[i].hostname)
                                            .changed();
                                    });

                                    // Editable Port field
                                    // todo
                                    table_row.col(|ui| {
                                        did_update |= ui
                                            .text_edit_singleline(&mut settings.servers[i].port)
                                            .changed();
                                    });

                                    table_row.col(|ui| {
                                        if ui.button("Delete").clicked() {
                                            settings.servers.remove(i);
                                            did_update = true;
                                        }
                                    });
                                });
                            }
                        });

                    if n_accounts == 0 {
                        ui.label("No servers. Click \"New Server\" to add your first one.");
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
