use std::{
    fs,
    sync::{Arc, Mutex},
};

use crate::launch::try_launch;
use eframe::egui::{self, Align, Button, Layout, Response, Ui, Vec2, Widget};
use libalembic::settings::{Account, AlembicSettings, ClientInfo, ServerInfo};

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

                    // Get client info
                    let client_info = if let Some(s) = ui.data_mut(|data| {
                        data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                    }) {
                        let settings = s.lock().unwrap();

                        Some(settings.client.clone())
                    } else {
                        None
                    };

                    if client_info.is_none() {
                        println!("Client info is none");
                        return;
                    }

                    // Get account info
                    let account_info = if let Some(s) = ui.data_mut(|data| {
                        data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                    }) {
                        let settings = s.lock().unwrap();

                        match settings.selected_account {
                            Some(index) => Some(settings.accounts[index].clone()),
                            None => None,
                        }
                    } else {
                        None
                    };

                    if account_info.is_none() {
                        println!("Account info is none");
                        return;
                    }

                    let final_client_info = client_info.unwrap();
                    let final_account_info = account_info.unwrap();

                    // Verify client exists
                    // if final_client_info.client_path
                    match fs::exists(&final_client_info.client_path) {
                        Ok(does_exist) => {
                            if does_exist {
                                println!("client path does exist");
                            } else {
                                println!("client path does not exist");
                                return;
                            }
                        }
                        Err(err) => todo!(),
                    }

                    println!(
                        "Trying launch with client {:?} and account {:?}",
                        final_client_info, final_account_info
                    );

                    match try_launch(&final_client_info, &final_account_info) {
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
