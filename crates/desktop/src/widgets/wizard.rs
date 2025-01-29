use std::sync::{Arc, Mutex};

use eframe::egui::{self, Align, Layout, Response, Ui, Widget};
use libalembic::settings::AlembicSettings;

use crate::application::{AppPage, WizardPage};

use super::components::centered_text;

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
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    // TODO: Is this really as good as egui can do to center things?
                    ui.add_space(ui.available_height() / 2.0);
                    ui.heading("Welcome to Alembic!");
                    if ui.button("Get started...").clicked() {
                        ui.memory_mut(|mem| {
                            mem.data
                                .insert_persisted(egui::Id::new("wizard_page"), WizardPage::Client)
                        });
                    }
                });
            }
            WizardPage::Client => {
                ui.heading("Game Client Setup");
                ui.add_space(16.0);

                ui.label("Game Client Path");
                if let Some(s) = ui.data_mut(|data| {
                    data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                }) {
                    let settings = s.lock().unwrap();
                    // TODO: Fix this so it works
                    // ui.text_edit_singleline(&mut settings.client_info.client_path);
                } else {
                    ui.label("Failed to get backend.");
                }

                ui.add_space(16.0);
                if ui.button("Continue").clicked() {
                    ui.memory_mut(|mem| {
                        mem.data
                            .insert_persisted(egui::Id::new("wizard_page"), WizardPage::Done)
                    });
                }
            }
            WizardPage::Done => {
                ui.label("Done");
                if ui.button("Exit").clicked() {
                    ui.memory_mut(|mem| {
                        mem.data
                            .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                    });
                }
            }
        })
        .response
    }
}
