use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use libalembic::{
    inject_config::{DllType, InjectConfig},
    scanner, settings::AlembicSettings,
};

use super::components::centered_text;

pub struct SettingsDllsTab {
    pub selected_index: Option<usize>,
}

impl Widget for &mut SettingsDllsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("DLL Configurations");

            ui.add_space(8.0);

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let mut settings = s.lock().unwrap();

                // Get the selected client
                let client_idx = settings.selected_client;
                if client_idx.is_none() {
                    return;
                }

                let client_idx = client_idx.unwrap();

                // Add buttons for different DLL types
                ui.horizontal(|ui| {
                    if ui.button("New Decal").clicked() {
                        let new_dll = InjectConfig {
                            dll_type: DllType::Decal,
                            dll_path: std::path::PathBuf::from(
                                "C:\\Program Files (x86)\\Decal 3.0\\Inject.dll",
                            ),
                            startup_function: Some("DecalStartup".to_string()),
                        };

                        if settings.add_dll_to_client(client_idx, new_dll) {
                            if let Some(dlls) = settings.get_client_dlls(client_idx) {
                                self.selected_index = Some(dlls.len() - 1);
                            }
                            let _ = settings.save();
                        }
                    }

                    if ui.button("New Alembic").clicked() {
                        let new_dll = InjectConfig {
                            dll_type: DllType::Alembic,
                            dll_path: std::path::PathBuf::from("C:\\path\\to\\alembic.dll"),
                            startup_function: None,
                        };

                        if settings.add_dll_to_client(client_idx, new_dll) {
                            if let Some(dlls) = settings.get_client_dlls(client_idx) {
                                self.selected_index = Some(dlls.len() - 1);
                            }
                            let _ = settings.save();
                        }
                    }

                    if ui.button("Discover DLLs").clicked() {
                        match scanner::scan_for_decal_dlls() {
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
                });

                ui.add_space(8.0);

                let dlls_for_client = settings.get_client_dlls(client_idx).map(|d| d.len()).unwrap_or(0);
                if dlls_for_client == 0 {
                    ui.label("No DLLs configured for this client. Click \"Discover DLLs\" to scan for installations, or add one manually.");
                    return;
                }

                // Clamp selected_index
                if let Some(idx) = self.selected_index {
                    if idx >= dlls_for_client {
                        self.selected_index = Some(dlls_for_client - 1);
                    }
                }

                let mut did_update = false;
                let mut delete_dll = false;

                // Two-pane layout
                egui::SidePanel::left("dlls_list")
                    .resizable(true)
                    .default_width(180.0)
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let dll_count = settings.get_client_dlls(client_idx).map(|d| d.len()).unwrap_or(0);
                            for i in 0..dll_count {
                                let dll_type_str = settings.get_client_dlls(client_idx)
                                    .and_then(|dlls| dlls.get(i))
                                    .map(|dll| format!("{}", dll.dll_type))
                                    .unwrap_or_else(|| "Unknown".to_string());

                                let selected = self.selected_index == Some(i);
                                let response = ui.selectable_label(selected, &dll_type_str);
                                if response.clicked() {
                                    self.selected_index = Some(i);
                                }
                            }
                        });
                    });

                // Right pane: detail editor
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let Some(idx) = self.selected_index else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Select a DLL from the list");
                        });
                        return;
                    };
                    let dll_count = settings.get_client_dlls(client_idx).map(|d| d.len()).unwrap_or(0);
                    if idx >= dll_count {
                        return;
                    }

                    // Read current values (clone to avoid borrow conflicts with closure)
                    let (dll_type, dll_path, startup_function) = settings
                        .get_client_dlls(client_idx)
                        .and_then(|dlls| dlls.get(idx).map(|d| (d.dll_type.clone(), d.dll_path.clone(), d.startup_function.clone())))
                        .unwrap_or((DllType::Alembic, std::path::PathBuf::new(), None));

                    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        // Delete button at top
                        if ui.button("Delete DLL").clicked() {
                            delete_dll = true;
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Type (read-only)
                        ui.label("Type");
                        ui.label(egui::RichText::new(dll_type.to_string()).strong());

                        ui.add_space(8.0);

                        // DLL Path
                        ui.label("DLL Path");
                        let mut path_string = dll_path.display().to_string();
                        if ui.text_edit_singleline(&mut path_string).changed() {
                            if let Some(dlls) = settings.get_client_dlls_mut(client_idx) {
                                if let Some(dll) = dlls.get_mut(idx) {
                                    dll.dll_path = std::path::PathBuf::from(&path_string);
                                }
                            }
                            did_update = true;
                        }

                        ui.add_space(8.0);

                        // Startup Function (only editable for non-Decal DLLs)
                        if dll_type != DllType::Decal {
                            ui.label("Startup Function");
                            let mut startup_str = startup_function.unwrap_or_default();
                            if ui.text_edit_singleline(&mut startup_str).changed() {
                                if let Some(dlls) = settings.get_client_dlls_mut(client_idx) {
                                    if let Some(dll) = dlls.get_mut(idx) {
                                        dll.startup_function = if startup_str.is_empty() {
                                            None
                                        } else {
                                            Some(startup_str)
                                        };
                                    }
                                }
                                did_update = true;
                            }
                        }
                     });
                 });

                // Handle deletion outside the borrow
                if delete_dll {
                    if let Some(idx) = self.selected_index {
                        let dlls_len = settings.get_client_dlls(client_idx).map(|d| d.len()).unwrap_or(0);
                        if idx < dlls_len {
                            settings.remove_dll_from_client(client_idx, idx);
                            self.selected_index = if dlls_len <= 1 {
                                None
                            } else {
                                Some(idx.min(dlls_len - 2))
                            };
                            did_update = true;
                        }
                    }
                }

                if did_update {
                    let _ = settings.save();
                }
            } else {
                ui.group(|ui| centered_text(ui, "Failed to reach application backend."));
            }
        })
        .response
    }
}
