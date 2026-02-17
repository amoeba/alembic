use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use libalembic::{
    client_config::{LaunchCommand, WindowsClientConfig, WineClientConfig},
    inject_config::DllType,
    inject_config::InjectConfig,
    scanner,
    settings::{AlembicSettings, ClientConfigType},
};

use super::components::centered_text;
use egui_extras::{Column, TableBuilder};

pub struct SettingsClientsTab {
    pub selected_index: Option<usize>,
}

impl Widget for &mut SettingsClientsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Client Installations");

            ui.add_space(8.0);

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let mut settings = s.lock().unwrap();

                ui.horizontal(|ui| {
                    if ui.button("New Client").clicked() {
                        let mut env = std::collections::HashMap::new();
                        env.insert("WINEPREFIX".to_string(), "/path/to/prefix".to_string());

                        settings.clients.push(ClientConfigType::Wine(WineClientConfig {
                            name: "New Client".to_string(),
                            client_path: std::path::PathBuf::from(
                                "C:\\Turbine\\Asheron's Call\\acclient.exe",
                            ),
                            launch_command: LaunchCommand {
                                program: std::path::PathBuf::from("/usr/local/bin/wine64"),
                                args: Vec::new(),
                                env,
                            },
                            dlls: Vec::new(),
                            selected_dll: None,
                        }));

                        self.selected_index = Some(settings.clients.len() - 1);
                        let _ = settings.save();
                    }

                    if ui.button("Discover Clients").clicked() {
                        match scanner::scan_all() {
                            Ok(discovered) => {
                                let had_no_clients = settings.clients.is_empty();
                                let mut added_count = 0;

                                for config in discovered {
                                    let already_exists =
                                        settings.clients.iter().any(|existing| {
                                            existing.install_path() == config.install_path()
                                        });

                                    if !already_exists {
                                        let should_select =
                                            had_no_clients && added_count == 0;
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
                });

                ui.add_space(8.0);

                if settings.clients.is_empty() {
                    ui.label("No clients. Click \"Discover Clients\" to scan for installations, or \"New Client\" to add one manually.");
                    return;
                }

                // Clamp selected_index (early return above guarantees non-empty)
                if let Some(idx) = self.selected_index
                    && idx >= settings.clients.len()
                {
                    self.selected_index = Some(settings.clients.len() - 1);
                }

                let mut did_update = false;
                let mut delete_client = false;

                egui::SidePanel::left("clients_list")
                    .resizable(true)
                    .default_width(180.0)
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for i in 0..settings.clients.len() {
                                let client = &settings.clients[i];
                                let type_str = if client.is_wine() { "Wine" } else { "Windows" };
                                let selected = self.selected_index == Some(i);
                                let label = format!("{} ({})", client.name(), type_str);
                                if ui.selectable_label(selected, label).clicked() {
                                    self.selected_index = Some(i);
                                }
                            }
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let Some(idx) = self.selected_index else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Select a client from the list");
                        });
                        return;
                    };
                    if idx >= settings.clients.len() {
                        return;
                    }

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Delete button at top
                        if ui.button("Delete Client").clicked() {
                            delete_client = true;
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Type selector - FIRST
                        ui.label("Type");
                        let current_is_wine = settings.clients[idx].is_wine();
                        let mut selected_wine = current_is_wine;
                        egui::ComboBox::from_id_salt("client_type")
                            .selected_text(if selected_wine { "Wine" } else { "Windows" })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected_wine, true, "Wine");
                                ui.selectable_value(&mut selected_wine, false, "Windows");
                            });
                        if selected_wine != current_is_wine {
                            let old_name = settings.clients[idx].name().to_string();
                            let old_path = settings.clients[idx].client_path().to_path_buf();
                            if selected_wine {
                                let mut env = std::collections::HashMap::new();
                                env.insert("WINEPREFIX".to_string(), "/path/to/prefix".to_string());
                                settings.clients[idx] = ClientConfigType::Wine(WineClientConfig {
                                    name: old_name,
                                    client_path: old_path,
                                    launch_command: LaunchCommand {
                                        program: std::path::PathBuf::from("/usr/local/bin/wine64"),
                                        args: Vec::new(),
                                        env,
                                    },
                                    dlls: Vec::new(),
                                    selected_dll: None,
                                });
                            } else {
                                settings.clients[idx] = ClientConfigType::Windows(WindowsClientConfig {
                                    name: old_name,
                                    client_path: old_path,
                                    dlls: Vec::new(),
                                    selected_dll: None,
                                });
                            }
                            did_update = true;
                        }

                        ui.add_space(4.0);

                        // Name
                        ui.label("Name");
                        let mut name = settings.clients[idx].name().to_owned();
                        let w = ui.available_width();
                        if ui.add(egui::TextEdit::singleline(&mut name).desired_width(w)).changed() {
                            *settings.clients[idx].name_mut() = name;
                            did_update = true;
                        }

                        ui.add_space(4.0);

                        // Launch Command (Wine only)
                        if let Some(lc) = settings.clients[idx].launch_command_mut() {
                            ui.add_space(12.0);
                            ui.separator();
                            ui.strong("Launch Command");
                            ui.add_space(4.0);

                            // Program
                            ui.label("Program");
                            let mut program_str = lc.program.display().to_string();
                            let w = ui.available_width();
                            if ui.add(egui::TextEdit::singleline(&mut program_str).desired_width(w)).changed() {
                                lc.program = std::path::PathBuf::from(&program_str);
                                did_update = true;
                            }

                            ui.add_space(8.0);

                            // Arguments
                            ui.label("Arguments");
                            let mut remove_arg_idx = None;
                            for (i, arg) in lc.args.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    let btn_space = 30.0 + ui.spacing().item_spacing.x;
                                    let field_width = (ui.available_width() - btn_space).max(60.0);
                                    ui.add(egui::TextEdit::singleline(arg).desired_width(field_width));
                                    if ui.button("x").clicked() {
                                        remove_arg_idx = Some(i);
                                    }
                                });
                            }
                            if let Some(i) = remove_arg_idx {
                                lc.args.remove(i);
                                did_update = true;
                            }
                            if ui.button("+Add Argument").clicked() {
                                lc.args.push(String::new());
                                did_update = true;
                            }

                            ui.add_space(8.0);

                            // Environment Variables
                            ui.label("Environment Variables");
                            let mut env_vec: Vec<(String, String)> = lc.env.drain().collect();
                            env_vec.sort_by(|a, b| a.0.cmp(&b.0));

                            let mut remove_env_idx = None;
                            let mut env_changed = false;
                            for (i, (key, value)) in env_vec.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    let spacing = ui.spacing().item_spacing.x;
                                    // Reserve space for: spacing + key + spacing + value + spacing + button
                                    let btn_width = ui.spacing().interact_size.x;
                                    let available = ui.available_width() - btn_width - 2.0 * spacing;
                                    let field_width = (available / 2.0).max(60.0);
                                    if ui.add(egui::TextEdit::singleline(key).desired_width(field_width)).changed() {
                                        env_changed = true;
                                    }
                                    if ui.add(egui::TextEdit::singleline(value).desired_width(field_width)).changed() {
                                        env_changed = true;
                                    }
                                    if ui.button("x").clicked() {
                                        remove_env_idx = Some(i);
                                    }
                                });
                            }
                            if let Some(i) = remove_env_idx {
                                env_vec.remove(i);
                                env_changed = true;
                            }
                            // Always rebuild from vec since we drained
                            lc.env = env_vec.into_iter().collect();
                            if env_changed {
                                did_update = true;
                            }
                            if ui.button("+Add Env Var").clicked() {
                                // Use a unique placeholder key to avoid HashMap collision
                                let key = format!("VAR_{}", lc.env.len());
                                lc.env.insert(key, String::new());
                                did_update = true;
                            }
                        }

                        // Client Path
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label("Client Path");
                        let mut path_string = settings.clients[idx].client_path().display().to_string();
                        let w = ui.available_width();
                        if ui.add(egui::TextEdit::singleline(&mut path_string).desired_width(w)).changed() {
                            *settings.clients[idx].client_path_mut() = std::path::PathBuf::from(&path_string);
                            did_update = true;
                        }

                        // DLL section
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.label("DLLs");
                        ui.add_space(8.0);

                        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                        // Scan button
                        ui.horizontal(|ui| {
                            if ui.button("Discover DLLs").clicked() {
                                match libalembic::scanner::scan_for_decal_dlls(&settings.clients) {
                                    Ok(discovered_dlls) => {
                                        if !discovered_dlls.is_empty() {
                                            let had_no_dlls = settings
                                                .get_client_dlls(idx)
                                                .map(|dlls| dlls.is_empty())
                                                .unwrap_or(true);

                                            for dll in discovered_dlls {
                                                settings.add_dll_to_client(idx, dll);
                                            }

                                            if had_no_dlls {
                                                settings.select_dll_for_client(idx, Some(0));
                                            }

                                            let _ = settings.save();
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error scanning for DLLs: {}", e);
                                    }
                                }
                            }

                            if ui.button("New DLL").clicked() {
                                let new_dll = InjectConfig {
                                    dll_path: std::path::PathBuf::from("C:\\Program Files\\Alembic\\Alembic.dll"),
                                    dll_type: DllType::Alembic,
                                    startup_function: None,
                                };
                                settings.add_dll_to_client(idx, new_dll);
                                let dlls = settings.get_client_dlls(idx).map(|d| d.len()).unwrap_or(0);
                                if dlls == 1 {
                                    settings.select_dll_for_client(idx, Some(0));
                                }
                                let _ = settings.save();
                            }
                        });

                        ui.add_space(8.0);

                        let dlls_for_client = settings.get_client_dlls(idx).map(|d| d.len()).unwrap_or(0);
                        if dlls_for_client == 0 {
                            ui.label("No DLLs configured. Click 'Discover DLLs' to scan, or 'New DLL' to add one manually.");
                        } else {
                            let mut to_remove: Option<usize> = None;

                            let dlls_cloned = if let Some(dlls) = settings.get_client_dlls(idx) {
                                dlls.clone()
                            } else {
                                vec![]
                            };

                            if !dlls_cloned.is_empty() {
                                let mut path_updates: Vec<(usize, String)> = Vec::new();

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
                                        for (index, dll) in dlls_cloned.iter().enumerate() {
                                            body.row(text_height, |mut row| {
                                                // Index column
                                                row.col(|ui| {
                                                    ui.label(index.to_string());
                                                });

                                                // Path column (editable)
                                                row.col(|ui| {
                                                    let mut path_str = dll.dll_path.display().to_string();
                                                    let w = ui.available_width();
                                                    if ui.add(egui::TextEdit::singleline(&mut path_str).desired_width(w)).changed() {
                                                        path_updates.push((index, path_str));
                                                    }
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
                                    });

                                // Apply path edits back to settings
                                for (dll_idx, new_path) in path_updates {
                                    if let Some(dlls) = settings.get_client_dlls_mut(idx)
                                        && let Some(dll) = dlls.get_mut(dll_idx)
                                    {
                                        dll.dll_path = std::path::PathBuf::from(&new_path);
                                        did_update = true;
                                    }
                                }
                            }

                            // Handle deletion
                            if let Some(index) = to_remove {
                                settings.remove_dll_from_client(idx, index);
                                let _ = settings.save();
                            }
                        }
                    });
                });

                // Set selected_client before releasing the lock
                settings.selected_client = self.selected_index;

                // Handle deletion
                if delete_client
                    && let Some(idx) = self.selected_index
                    && idx < settings.clients.len()
                {
                    settings.clients.remove(idx);
                    self.selected_index = if settings.clients.is_empty() {
                        None
                    } else {
                        Some(idx.min(settings.clients.len() - 1))
                    };
                    did_update = true;
                }

                if did_update {
                    let _ = settings.save();
                }
                // Lock released here when settings goes out of scope
            } else {
                ui.group(|ui| centered_text(ui, "Failed to reach application backend."));
            }


        })
        .response
    }
}
