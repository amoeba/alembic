use std::sync::{Arc, Mutex};

use eframe::egui::{self, Align, Layout, Response, Ui, Widget};
use libalembic::settings::AlembicSettings;

use crate::application::{AppPage, WizardPage};

use super::components::{SettingsDLLPathEdit, SettingsGameClientPathEdit};

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
                                // Save is_configured
                                if let Some(s) = ui.data_mut(|data| {
                                    data.get_persisted::<Arc<Mutex<AlembicSettings>>>(
                                        egui::Id::new("settings"),
                                    )
                                }) {
                                    let mut settings = s.lock().unwrap();
                                    settings.is_configured = true;
                                    let _ = settings.save();
                                };

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
                                    // Save is_configured
                                    if let Some(s) = ui.data_mut(|data| {
                                        data.get_persisted::<Arc<Mutex<AlembicSettings>>>(
                                            egui::Id::new("settings"),
                                        )
                                    }) {
                                        let mut settings = s.lock().unwrap();
                                        settings.is_configured = true;
                                        let _ = settings.save();
                                    };

                                    ui.memory_mut(|mem| {
                                        mem.data.insert_persisted(
                                            egui::Id::new("app_page"),
                                            AppPage::Main,
                                        )
                                    });
                                }
                            });

                            ui.add_space(ui.available_width());

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                                if ui.button("Continue").clicked() {
                                    ui.memory_mut(|mem| {
                                        mem.data.insert_persisted(
                                            egui::Id::new("wizard_page"),
                                            WizardPage::Done,
                                        )
                                    });
                                }
                            });
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading("Setup");
                    ui.add_space(16.0);
                    ui.add(&mut SettingsGameClientPathEdit {});
                    ui.add_space(16.0);
                    ui.add(&mut SettingsDLLPathEdit {});
                });
            }
            WizardPage::Done => {
                egui::TopBottomPanel::bottom("wizard_controls")
                    .resizable(false)
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            if ui.button("Exit Setup").clicked() {
                                // Save is_configured
                                if let Some(s) = ui.data_mut(|data| {
                                    data.get_persisted::<Arc<Mutex<AlembicSettings>>>(
                                        egui::Id::new("settings"),
                                    )
                                }) {
                                    let mut settings = s.lock().unwrap();
                                    settings.is_configured = true;
                                    let _ = settings.save();
                                };

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
                        // Save is_configured
                        if let Some(s) = ui.data_mut(|data| {
                            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new(
                                "settings",
                            ))
                        }) {
                            let mut settings = s.lock().unwrap();
                            settings.is_configured = true;
                            let _ = settings.save();
                        };

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
