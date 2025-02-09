use std::{
    fs::{self},
    sync::{Arc, Mutex},
};

use eframe::egui::{self, Align, Color32, Layout, Response, RichText, Ui, Widget};
use libalembic::settings::AlembicSettings;

use crate::application::{AppPage, WizardPage};

pub struct Wizard {}

impl Wizard {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut Wizard {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut current_wizard_page = WizardPage::Start;

        ui.memory_mut(|mem| {
            if let Some(val) = mem
                .data
                .get_persisted::<WizardPage>(egui::Id::new("wizard_page"))
            {
                current_wizard_page = val;
            }
        });

        ui.vertical(|ui| match current_wizard_page {
            WizardPage::Start => {
                egui::TopBottomPanel::bottom("wizard_controls")
                    .resizable(false)
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            if ui.button("Exit Setup").clicked() {
                                // Update and save settings now
                                ui.data_mut(|data| {
                                    if let Some(settings_ref) =
                                        data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings")) {
                                            let mut settings = settings_ref.lock().unwrap();
                                            settings.general.is_configured = true;
                                            settings.save().expect("Unhandled error: Failed to save settings.")
                                        }
                                });

                                // Then progress the view
                                ui.memory_mut(|mem| {
                                    mem.data
                                        .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                                });
                            }
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        // TODO: Is this really as good as egui can do to center things?
                        ui.add_space(ui.available_height() / 2.0);
                        ui.heading("Welcome to Alembic!");
                        ui.add_space(16.0);
                        if ui.button("Get started...").clicked() {
                            ui.memory_mut(|mem| {
                                mem.data.insert_persisted(
                                    egui::Id::new("wizard_page"),
                                    WizardPage::Client,
                                )
                            });
                        }
                    });
                });
            }
            WizardPage::Client => {
                egui::TopBottomPanel::bottom("wizard_controls")
                    .resizable(false)
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {

                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            if ui.button("Exit Setup").clicked() {
                                // Update and save settings now
                                ui.data_mut(|data| {
                                    if let Some(settings_ref) =
                                        data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings")) {
                                            let mut settings = settings_ref.lock().unwrap();
                                            settings.general.is_configured = true;
                                            settings.save().expect("Unhandled error: Failed to save settings.")
                                        }
                                });

                                // Then progress the view
                                ui.memory_mut(|mem| {
                                    mem.data
                                        .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                                });
                            }
                            });

                            ui.add_space(ui.available_width());

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                                if ui.button("Continue").clicked() {
                                    // Save settings now
                                    ui.data_mut(|data| {
                                        if let Some(settings_ref) =
                                            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings")) {
                                                let settings = settings_ref.lock().unwrap();
                                                settings.save().expect("Unhandled error: Failed to save settings.")
                                            }
                                    });

                                    // Then progress the view
                                    ui.memory_mut(|mem| {
                                        mem.data
                                            .insert_persisted(egui::Id::new("wizard_page"), WizardPage::Done)
                                    });
                                }
                            });
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading("Setup");
                    ui.add_space(16.0);

                    // Client Path
                    ui.label("Game Client Path");

                    if let Some(s) = ui.data_mut(|data| {
                        data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                    }) {
                        let mut settings = s.lock().unwrap();
                        ui.text_edit_singleline(&mut settings.client.client_path);

                        // Indicator
                        match fs::exists(settings.client.client_path.clone()) {
                            Ok(result) => match result {
                                true => {
                                    ui.label(RichText::new("Path exists.").color(Color32::GREEN))
                                }
                                false => ui.label(
                                    RichText::new(
                                        "Path does not exist. Please enter a valid path.",
                                    )
                                    .color(Color32::YELLOW),
                                ),
                            },
                            Err(_) => ui.label(
                                RichText::new("Error determining whether path exists. Please report this as a bug.")
                                    .color(Color32::RED),
                            ),
                        };
                    } else {
                        ui.label("Failed to get backend.");
                    }

                    ui.add_space(16.0);

                    // DLL Path
                    ui.label("Alembic DLL Path");

                    if let Some(s) = ui.data_mut(|data| {
                        data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                    }) {
                        let mut settings = s.lock().unwrap();
                        ui.text_edit_singleline(&mut settings.dll.dll_path);

                        // Indicator
                        match fs::exists(settings.dll.dll_path.clone()) {
                            Ok(result) => match result {
                                true => {
                                    ui.label(RichText::new("Path exists.").color(Color32::GREEN))
                                }
                                false => ui.label(
                                    RichText::new(
                                        "Path does not exist. Please enter a valid path.",
                                    )
                                    .color(Color32::YELLOW),
                                ),
                            },
                            Err(_) => ui.label(
                                RichText::new("Error determining whether path exists. Please report this as a bug.")
                                    .color(Color32::RED),
                            ),
                        };
                    } else {
                        ui.label("Failed to get backend.");
                    }
                });
            }
            WizardPage::Done => {
                egui::TopBottomPanel::bottom("wizard_controls")
                    .resizable(false)
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            if ui.button("Exit Setup").clicked() {
                                // Update and save settings now
                                ui.data_mut(|data| {
                                    if let Some(settings_ref) =
                                        data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings")) {
                                            let mut settings = settings_ref.lock().unwrap();
                                            settings.general.is_configured = true;
                                            settings.save().expect("Unhandled error: Failed to save settings.")
                                        }
                                });

                                // Then progress the view
                                ui.memory_mut(|mem| {
                                    mem.data
                                        .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                                });
                            }
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading("Setup");
                    ui.add_space(16.0);

                    ui.heading("Finish Setup");
                    if ui.button("Finish").clicked() {
                        // Update and save settings now
                        ui.data_mut(|data| {
                            if let Some(settings_ref) =
                                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings")) {
                                    let mut settings = settings_ref.lock().unwrap();
                                    settings.general.is_configured = true;
                                    settings.save().expect("Unhandled error: Failed to save settings.")
                                }
                        });

                        // Then progress the view
                        ui.memory_mut(|mem| {
                            mem.data
                                .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                        });
                    }
                });
            }
        })
        .response
    }
}
