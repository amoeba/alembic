use eframe::egui::{self, Response, Ui, Widget};

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
                ui.label("Welcome!");
                if ui.button("Next").clicked() {
                    ui.memory_mut(|mem| {
                        mem.data
                            .insert_persisted(egui::Id::new("wizard_page"), WizardPage::Client)
                    });
                }
            }
            WizardPage::Client => {
                ui.label("Set up your client...");
                if ui.button("Next").clicked() {
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
