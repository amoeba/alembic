use eframe::egui::{self, Response, Ui, Widget};

use crate::launch::try_launch;

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

            // let mut selected = ComboOptions::First;
            // egui::ComboBox::from_label("Select one!")
            //     .selected_text(format!("{:?}", selected))
            //     .show_ui(ui, |ui| {
            //         ui.selectable_value(&mut selected, ComboOptions::First, "First");
            //         ui.selectable_value(&mut selected, ComboOptions::Second, "Second");
            //         ui.selectable_value(&mut selected, ComboOptions::Third, "Third");
            //     });
        })
        .response
    }
}
