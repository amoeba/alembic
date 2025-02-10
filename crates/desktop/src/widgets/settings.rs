use eframe::egui::{self, Align, Layout, Response, Ui, Widget};

use crate::application::AppPage;

use super::components::{SettingsDLLPathEdit, SettingsGameClientPathEdit};

pub struct Settings {}

impl Settings {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for &mut Settings {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            egui::TopBottomPanel::bottom("settings_controls")
                .resizable(false)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Done").clicked() {
                            ui.memory_mut(|mem| {
                                mem.data
                                    .insert_persisted(egui::Id::new("app_page"), AppPage::Main)
                            });
                        }
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    ui.heading("Settings");
                    ui.add_space(16.0);
                    ui.add(&mut SettingsGameClientPathEdit {});
                    ui.add_space(16.0);
                    ui.add(&mut SettingsDLLPathEdit {});
                });
            });
        })
        .response
    }
}
