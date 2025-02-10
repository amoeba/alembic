use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::settings::{Account, AlembicSettings, ServerInfo};

use super::components::centered_text;

pub struct AccountsMainTab {
    pub selected_server: Option<usize>,
}

impl Widget for &mut AccountsMainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            if let Some(settings) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                ui.group(|ui| {
                    // Decide what to show for the  ComboBox text
                    let selected_text = match self.selected_server {
                        Some(index) => "something",
                        None => "nothing",
                    };

                    let mut x = 0;

                    egui::ComboBox::from_id_salt("AccountServer")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (index, server) in
                                settings.lock().unwrap().servers.iter().enumerate()
                            {
                                ui.selectable_value(&mut x, index, server.hostname.clone());
                            }
                        });

                    // Testing
                    if ui.button("Testing add").clicked() {
                        let new_account = Account {
                            server_index: self.selected_server.unwrap_or_default(),
                            name: "Test".to_string(),
                            username: "Test".to_string(),
                            password: "Test".to_string(),
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
                        .column(Column::auto()) // Username column
                        .column(Column::auto()) // Password column
                        .header(text_height, |mut header| {
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
