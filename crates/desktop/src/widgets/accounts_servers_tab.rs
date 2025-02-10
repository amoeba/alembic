use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::settings::{AlembicSettings, ServerInfo};

use super::components::centered_text;

pub struct AccountsServersTab {}

impl Widget for &mut AccountsServersTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("Servers");

            if let Some(settings) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                ui.group(|ui| {
                    // Testing
                    if ui.button("Teting add").clicked() {
                        let new_server = ServerInfo {
                            hostname: "test".to_string(),
                            port: "9000".to_string(),
                        };

                        settings.lock().unwrap().servers.push(new_server);
                        let _ = settings.lock().unwrap().save();
                    }

                    // List servers
                    let n_servers = settings.lock().unwrap().servers.iter().len().to_string();
                    ui.label(format!("n servers: {}", n_servers));

                    // WIP Servers Table
                    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                    let mut settings_mut = settings.lock().unwrap();

                    // Dirty checking
                    let mut did_update = false;

                    TableBuilder::new(ui)
                        .striped(true) // Enable striped rows for readability
                        .resizable(true) // Allow column resizing
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center)) // Cell layout
                        .column(Column::auto()) // Address column
                        .column(Column::auto()) // Port column
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.label("Address");
                            });
                            header.col(|ui| {
                                ui.label("Port");
                            });
                        })
                        .body(|mut body| {
                            for server in &mut settings_mut.servers {
                                body.row(text_height, |mut table_row| {
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

                    // WIP: Handle save
                    if did_update {
                        println!("Should save...");
                        // This is deadlocking me, figure it out.
                        // let _ = settings.lock().unwrap().save();
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
