use std::{
    fs,
    sync::{Arc, Mutex},
};

use crate::{
    backend::{Backend, Client},
    launch::try_launch,
};
use eframe::egui::{self, Align, Button, Layout, Response, Ui, Vec2, Widget};
use libalembic::settings::AlembicSettings;

use super::components::{AccountPicker, ServerPicker};

pub struct MainTab {}

impl Widget for &mut MainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
            ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                let have_client = if let Some(s) = ui.data_mut(|data| {
                    data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                }) {
                    let backend = s.lock().unwrap();

                    backend.client.is_some()
                } else {
                    false
                };
                let is_injected = if let Some(s) = ui.data_mut(|data| {
                    data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                }) {
                    let backend = s.lock().unwrap();

                    backend.is_injected
                } else {
                    false
                };

                let can_launch = if let Some(s) = ui.data_mut(|data| {
                    data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                }) {
                    let settings = s.lock().unwrap();

                    match settings.selected_account {
                        Some(_) => true,
                        None => false,
                    }
                } else {
                    false
                };

                ui.add_enabled_ui(can_launch, |ui| {
                    if ui
                        .add_sized(Vec2::new(140.0, 70.0), Button::new("Launch"))
                        .clicked()
                    {
                        println!("Launch clicked.");

                        // Get client info
                        let client_info = if let Some(s) = ui.data_mut(|data| {
                            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new(
                                "settings",
                            ))
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

                        // Get server info
                        let server_info = if let Some(s) = ui.data_mut(|data| {
                            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new(
                                "settings",
                            ))
                        }) {
                            let settings = s.lock().unwrap();

                            match settings.selected_server {
                                Some(index) => Some(settings.servers[index].clone()),
                                None => None,
                            }
                        } else {
                            None
                        };

                        if server_info.is_none() {
                            println!("Server info is none");
                            return;
                        }

                        // Get account info
                        let account_info = if let Some(s) = ui.data_mut(|data| {
                            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new(
                                "settings",
                            ))
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
                        let final_server_info: libalembic::settings::ServerInfo =
                            server_info.unwrap();
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

                        match try_launch(
                            &final_client_info,
                            &final_server_info,
                            &final_account_info,
                        ) {
                            Ok(val) => {
                                println!("Launch succeeded. Launched pid is {val}!");

                                if let Some(backend_ref) = ui.data_mut(|data| {
                                    data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new(
                                        "backend",
                                    ))
                                }) {
                                    let mut backend = backend_ref.lock().unwrap();

                                    backend.client = Some(Client { pid: val });
                                    backend.is_injected = true;
                                }
                            }
                            Err(error) => println!("Launch failed with error: {error}"),
                        }

                        println!("Launch process is over.");
                    }
                });

                let selected_server = if let Some(s) = ui.data_mut(|data| {
                    data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                }) {
                    s.lock().unwrap().selected_server
                } else {
                    None
                };

                ui.add(&mut AccountPicker {
                    selected_server: selected_server,
                });
                ui.add(&mut ServerPicker {});
            });
        })
        .response
    }
}
