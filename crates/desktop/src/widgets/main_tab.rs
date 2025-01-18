use std::sync::{Arc, Mutex};

use eframe::egui::{self, Response, Ui, Widget};

use crate::{backend::Backend, launch::try_launch};

use super::components::centered_text;

struct AccountPicker {}

impl Widget for &mut AccountPicker {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            let mut backend = backend.lock().unwrap();

            // TODO: Not sure if we have to do this immutable borrow
            let account_names: Vec<String> =
                backend.accounts.iter().map(|a| a.name.clone()).collect();

            let selected_text = backend
                .selected_account
                .and_then(|index| account_names.get(index).cloned())
                .unwrap_or_else(|| "Choose an account".to_string());

            ui.group(|ui| {
                egui::ComboBox::from_id_salt("Account")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for (index, name) in account_names.iter().enumerate() {
                            ui.selectable_value(
                                &mut backend.selected_account,
                                Some(index),
                                name.clone(),
                            );
                        }
                    })
                    .response
            })
            .response
        } else {
            ui.group(|ui| centered_text(ui, "Failed to reach application backend."))
                .response
        }
    }
}
pub struct MainTab {}

impl Widget for &mut MainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            if ui.add(egui::Button::new("Launch")).clicked() {
                println!("Launch clicked.");

                match try_launch() {
                    Ok(_) => println!("Launch succeeded."),
                    Err(error) => println!("Launch failed with error: {error}"),
                }

                println!("Launch completed.");
            }

            ui.add(&mut AccountPicker {})
        })
        .response
    }
}
