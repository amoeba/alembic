use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::settings::{Account, AlembicSettings, ServerInfo};
use tarpc::server::Serve;

use crate::backend::Backend;

use super::components::centered_text;

#[derive(Default)]
struct TableRow {
    address: String,
    port: String,
    username: String,
    password: String,
}

pub struct AccountsTab {}

impl Widget for &mut AccountsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("Accounts");

            if let Some(settings) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                ui.group(|ui| {
                    // Testing
                    if ui.button("Teting add").clicked() {
                        let new_account = Account {
                            name: "Test".to_string(),
                            username: "Test".to_string(),
                            password: "Test".to_string(),
                            server_info: ServerInfo {
                                hostname: "test".to_string(),
                                port: 9000,
                            },
                        };

                        settings.lock().unwrap().accounts.push(new_account);
                        let _ = settings.lock().unwrap().save();
                    }

                    // List accounts
                    let n_accounts = settings.lock().unwrap().accounts.iter().len().to_string();
                    ui.label(format!("n accounts: {}", n_accounts));

                    // WIP Accounts Table
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
                        .column(Column::auto()) // Username column
                        .column(Column::auto()) // Password column
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.label("Address");
                            });
                            header.col(|ui| {
                                ui.label("Port");
                            });
                            header.col(|ui| {
                                ui.label("Username");
                            });
                            header.col(|ui| {
                                ui.label("Password");
                            });
                        })
                        .body(|mut body| {
                            for account in &mut settings_mut.accounts {
                                body.row(text_height, |mut table_row| {
                                    // Editable Address field
                                    table_row.col(|ui| {
                                        did_update |= ui
                                            .text_edit_singleline(&mut account.server_info.hostname)
                                            .changed();
                                    });

                                    // Editable Port field
                                    // todo
                                    table_row.col(|ui| {
                                        did_update |= ui
                                            .text_edit_singleline(&mut account.server_info.hostname)
                                            .changed();
                                    });

                                    // Editable Username field
                                    table_row.col(|ui| {
                                        did_update |= ui
                                            .text_edit_singleline(&mut account.username)
                                            .changed();
                                    });

                                    // TODO
                                    // TODO: Handle save on dirty
                                    // Editable Password field (masked by default)
                                    table_row.col(|ui| {
                                        let mut password_edit =
                                            egui::TextEdit::singleline(&mut account.password)
                                                .password(true); // Mask input

                                        // if ui
                                        // .memory()
                                        // .has_focus(ui.id().with(account.password.clone()))
                                        // {
                                        password_edit = password_edit.password(false);
                                        // Show password when focused
                                        // }

                                        ui.add(password_edit);
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
