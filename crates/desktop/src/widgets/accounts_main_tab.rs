use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::settings::{Account, AlembicSettings};

use super::components::centered_text;

pub struct AccountsMainTab {
    pub selected_server: Option<usize>,
}

impl Widget for &mut AccountsMainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Accounts");
            ui.add_space(8.0);

            if let Some(settings) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                if settings.lock().unwrap().servers.len() == 0 {
                    ui.vertical(|ui| {
                        ui.label("No servers. Switch to the Servers tab to add your first one.");
                    })
                    .response
                } else {
                    ui.vertical(|ui| {
                        // Server Picker
                        ui.horizontal(|ui| {
                            ui.label("Server:");
                            let selected_text = match self.selected_server {
                                Some(index) => settings.lock().unwrap().servers[index].name.clone(),
                                None => "Pick a server".to_string(),
                            };

                            egui::ComboBox::from_id_salt("AccountServer")
                                .selected_text(selected_text)
                                .show_ui(ui, |ui| {
                                    for (index, server) in
                                        settings.lock().unwrap().servers.iter().enumerate()
                                    {
                                        ui.selectable_value(
                                            &mut self.selected_server,
                                            Some(index),
                                            server.name.clone(),
                                        );
                                    }
                                });
                        });

                        ui.add_space(8.0);

                        // Add Account
                        if self.selected_server.is_some() {
                            if ui.button("New Account").clicked() {
                                let new_account = Account {
                                    server_index: self.selected_server.unwrap_or_default(),
                                    username: "username".to_string(),
                                    password: "password".to_string(),
                                };

                                settings.lock().unwrap().accounts.push(new_account);
                                let _ = settings.lock().unwrap().save();
                            }

                            ui.add_space(8.0);
                        }

                        if self.selected_server.is_some() {
                            // Accounts Listing
                            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                            let mut did_update = false; // Dirty checking for saving settings

                            let mut settings_guard = settings.lock().unwrap();
                            let accounts = settings_guard.accounts.iter_mut().enumerate().filter(
                                |(_, account)| {
                                    self.selected_server.is_some()
                                        && account.server_index == self.selected_server.unwrap()
                                },
                            );

                            // Easy way to get a count from the above iterator
                            let mut n_accounts = 0;

                            TableBuilder::new(ui)
                                .striped(true) // Enable striped rows for readability
                                .resizable(true) // Allow column resizing
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center)) // Cell layout
                                .column(Column::auto()) // Username column
                                .column(Column::auto()) // Password column
                                .header(text_height, |mut header| {
                                    header.col(|ui| {
                                        ui.strong("Username");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Password");
                                    });
                                })
                                .body(|mut body| {
                                    for (_index, account) in accounts {
                                        n_accounts += 1;

                                        body.row(text_height, |mut table_row| {
                                            // Editable Username field
                                            table_row.col(|ui| {
                                                did_update |= ui
                                                    .text_edit_singleline(&mut account.username)
                                                    .changed();
                                            });

                                            // Editable Password field (masked by default)
                                            table_row.col(|ui| {
                                                let password_id = ui.make_persistent_id(format!(
                                                    "password_{}",
                                                    account.username
                                                ));
                                                let is_focused =
                                                    ui.memory(|m| m.has_focus(password_id));

                                                let password_edit = egui::TextEdit::singleline(
                                                    &mut account.password,
                                                )
                                                .id(password_id)
                                                .password(!is_focused);

                                                if ui.add(password_edit).changed() {
                                                    did_update = true;
                                                }
                                            });
                                        });
                                    }
                                });

                            if n_accounts == 0 {
                                ui.label(
                                    "No accounts. Click \"New Account\" to add your first one.",
                                );
                            }

                            // Save but only if we need to
                            if did_update {
                                let _ = settings.lock().unwrap().save();
                            }
                        }
                    })
                    .response
                }
            } else {
                ui.group(|ui| centered_text(ui, "Failed to reach application backend."))
                    .response
            }
        })
        .response
    }
}
