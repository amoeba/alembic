use std::{
    iter,
    sync::{Arc, Mutex},
};

use crate::launch::try_launch;
use eframe::egui::{self, Align, Button, Layout, Response, Ui, Vec2, Widget};
use libalembic::settings::AlembicSettings;

struct AccountPicker {}

impl Widget for &mut AccountPicker {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(s) = ui.data_mut(|data| {
            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
        }) {
            let mut settings = s.lock().unwrap();

            let account_names: Vec<String> = settings
                .accounts
                .iter()
                .map(|account| account.name.clone())
                .collect();

            let selected_text = settings
                .selected_account
                .and_then(|index| account_names.get(index).cloned())
                .unwrap_or_else(|| "Pick an account".to_string());

            egui::ComboBox::from_id_salt("Account")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (index, name) in account_names.iter().enumerate() {
                        ui.selectable_value(
                            &mut settings.selected_account,
                            Some(index),
                            name.clone(),
                        );
                    }
                })
                .response
        } else {
            ui.label("Bug, please report.")
        }
    }
}
pub struct MainTab {}

impl Widget for &mut MainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
            ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                if ui
                    .add_sized(Vec2::new(140.0, 35.0), Button::new("Inject"))
                    .clicked()
                {
                    // TODO
                }
                if ui
                    .add_sized(Vec2::new(140.0, 70.0), Button::new("Launch"))
                    .clicked()
                {
                    println!("Launch clicked.");

                    match try_launch() {
                        Ok(_) => println!("Launch succeeded."),
                        Err(error) => println!("Launch failed with error: {error}"),
                    }

                    println!("Launch completed.");
                }

                ui.add(&mut AccountPicker {});
            });
        })
        .response
    }
}
