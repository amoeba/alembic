use std::sync::{Arc, Mutex};

use eframe::egui::{self, Layout, Response, Ui, Widget};
use libalembic::settings::AlembicSettings;

pub fn centered_text(ui: &mut Ui, text: &str) {
    ui.with_layout(
        Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| ui.label(text),
    );
}

pub struct AccountPicker {}

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
