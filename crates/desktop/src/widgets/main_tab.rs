use crate::launch::try_launch;
use eframe::egui::{Align, Button, Layout, Response, Ui, Vec2, Widget};

use super::components::AccountPicker;

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
