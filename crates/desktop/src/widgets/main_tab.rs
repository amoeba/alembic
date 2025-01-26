use std::sync::{Arc, Mutex};

use crate::launch::try_launch;
use eframe::egui::{self, Align, Button, Layout, Response, Ui, Vec2, Widget};
use libalembic::settings::AlembicSettings;

use super::components::AccountPicker;

pub struct MainTab {}

impl Widget for &mut MainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
            if ui
                .add_sized(Vec2::new(100.0, 300.0), Button::new("Debug: Save Settings"))
                .clicked()
            {
                println!("clicked");
                if let Some(s) = ui.data_mut(|data| {
                    data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                }) {
                    let settings = s.lock().unwrap();

                    match settings.save() {
                        Ok(_) => println!("saved"),
                        Err(error) => eprintln!("Error saving: {error}."),
                    }
                } else {
                    println!("Failed");
                }
            }

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
