use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::{
    client_config::{DllType, InjectConfig, WineInjectConfig},
    settings::AlembicSettings,
};

#[cfg(target_os = "windows")]
use libalembic::client_config::WindowsInjectConfig;

use super::components::centered_text;

pub struct SettingsDllsTab {}

impl Widget for &mut SettingsDllsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("DLL Configurations");

            ui.add_space(8.0);

            if let Some(s) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            }) {
                let mut settings = s.lock().unwrap();

                ui.vertical(|ui| {
                    // Add buttons for different DLL types
                    ui.horizontal(|ui| {
                        if ui.button("New Decal").clicked() {
                            let new_dll = InjectConfig::Wine(WineInjectConfig {
                                dll_type: DllType::Decal,
                                wine_prefix: std::path::PathBuf::from("/path/to/prefix"),
                                dll_path: std::path::PathBuf::from(
                                    "C:\\Program Files (x86)\\Decal 3.0\\Inject.dll",
                                ),
                            });

                            settings.discovered_dlls.push(new_dll);
                            let _ = settings.save();
                        }

                        if ui.button("New Alembic").clicked() {
                            let new_dll = InjectConfig::Wine(WineInjectConfig {
                                dll_type: DllType::Alembic,
                                wine_prefix: std::path::PathBuf::from("/path/to/prefix"),
                                dll_path: std::path::PathBuf::from("C:\\path\\to\\alembic.dll"),
                            });

                            settings.discovered_dlls.push(new_dll);
                            let _ = settings.save();
                        }

                        #[cfg(target_os = "windows")]
                        if ui.button("New Decal").clicked() {
                            let new_dll = InjectConfig::Windows(WindowsInjectConfig {
                                dll_path: std::path::PathBuf::from(
                                    "C:\\Program Files (x86)\\Decal 3.0\\Inject.dll",
                                ),
                                dll_type: DllType::Decal,
                            });

                            settings.discovered_dlls.push(new_dll);
                            let _ = settings.save();
                        }

                        #[cfg(target_os = "windows")]
                        if ui.button("New Alembic").clicked() {
                            let new_dll = InjectConfig::Windows(WindowsInjectConfig {
                                dll_path: std::path::PathBuf::from("C:\\path\\to\\alembic.dll"),
                                dll_type: DllType::Alembic,
                            });

                            settings.discovered_dlls.push(new_dll);
                            let _ = settings.save();
                        }
                    });

                    ui.add_space(8.0);

                    // DLLs listing
                    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
                    let mut did_update = false; // Dirty checking for saving settings

                    let mut n_dlls = 0;

                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto()) // Platform
                        .column(Column::auto()) // Type
                        .column(Column::remainder()) // DLL Path
                        .column(Column::auto()) // Delete
                        .header(text_height, |mut header| {
                            header.col(|ui| {
                                ui.strong("Platform");
                            });
                            header.col(|ui| {
                                ui.strong("Type");
                            });
                            header.col(|ui| {
                                ui.strong("DLL Path");
                            });
                            header.col(|ui| {
                                ui.strong("Delete");
                            });
                        })
                        .body(|mut body| {
                            let indices: Vec<usize> = (0..settings.discovered_dlls.len()).collect();

                            for i in indices {
                                n_dlls += 1;

                                body.row(text_height, |mut table_row| {
                                    // Platform (non-editable)
                                    table_row.col(|ui| {
                                        let platform_str = match &settings.discovered_dlls[i] {
                                            InjectConfig::Wine(_) => "Wine",
                                            InjectConfig::Windows(_) => "Windows",
                                        };
                                        ui.label(platform_str);
                                    });

                                    // Type (non-editable)
                                    table_row.col(|ui| {
                                        let type_str =
                                            settings.discovered_dlls[i].dll_type().to_string();
                                        ui.label(type_str);
                                    });

                                    // DLL Path (editable)
                                    table_row.col(|ui| {
                                        let mut path_string = settings.discovered_dlls[i]
                                            .dll_path()
                                            .display()
                                            .to_string();

                                        if ui.text_edit_singleline(&mut path_string).changed() {
                                            let new_path = std::path::PathBuf::from(&path_string);
                                            match &mut settings.discovered_dlls[i] {
                                                InjectConfig::Wine(config) => {
                                                    config.dll_path = new_path;
                                                }
                                                InjectConfig::Windows(config) => {
                                                    config.dll_path = new_path;
                                                }
                                            }
                                            did_update = true;
                                        }
                                    });

                                    // Delete button
                                    table_row.col(|ui| {
                                        if ui.button("Delete").clicked() {
                                            settings.discovered_dlls.remove(i);
                                            did_update = true;
                                        }
                                    });
                                });
                            }
                        });

                    if n_dlls == 0 {
                        ui.label("No DLLs configured. Click a button above to add your first one.");
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
