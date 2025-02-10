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

            if let Some(settings) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                ui.vertical(|ui| {
                    // Add
                    if ui.button("New Server").clicked() {
                        let new_server = ServerInfo {
                            name: "Server".to_string(),
                            hostname: "hostname or IP address".to_string(),
                            port: "9000".to_string(),
                        };

                        settings.lock().unwrap().servers.push(new_server);
                        let _ = settings.lock().unwrap().save();
                    }

                    ui.add_space(8.0);

                    // Servers listing
                    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                    let mut did_update = false; // Dirty checking for saving settings

                    TableBuilder::new(ui)
                        .striped(true) // Enable striped rows for readability
                        .resizable(true) // Allow column resizing
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center)) // Cell layout
                        .column(Column::auto()) // Name column
                        .column(Column::auto()) // Address column
                        .column(Column::auto()) // Port column
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.label("Name");
                            });
                            header.col(|ui| {
                                ui.label("Address");
                            });
                            header.col(|ui| {
                                ui.label("Port");
                            });
                        })
                        .body(|mut body| {
                            for server in &mut settings.lock().unwrap().servers {
                                body.row(text_height, |mut table_row| {
                                    // Editable Name field
                                    table_row.col(|ui| {
                                        did_update |=
                                            ui.text_edit_singleline(&mut server.name).changed();
                                    });

                                    // Editable Address field
                                    table_row.col(|ui| {
                                        did_update |=
                                            ui.text_edit_singleline(&mut server.hostname).changed();
                                    });

                                    // Editable Port field
                                    // todo
                                    table_row.col(|ui| {
                                        did_update |=
                                            ui.text_edit_singleline(&mut server.port).changed();
                                    });
                                });
                            }
                        });

                    // Save but only if we need to
                    if did_update {
                        let _ = settings.lock().unwrap().save();
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
