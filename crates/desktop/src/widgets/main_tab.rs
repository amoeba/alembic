use std::{
    fs,
    sync::{Arc, Mutex},
};

use crate::{
    backend::{AppModal, Backend, Client},
    launch::try_launch,
};
use eframe::egui::{self, style::DebugOptions, Align, Button, Layout, Response, Ui, Vec2, Widget};
use image::Luma;
use libalembic::{
    launcher::{
        launcher::{Launcher, LauncherImpl},
        windows::WindowsLauncher,
    },
    settings::{AlembicSettings, DllInfo},
};
use tarpc::client;

use super::{
    components::{AccountPicker, ServerPicker},
    news::News,
};

pub struct MainTab {
    sidebar_width: f32,
    news: News,
    current_launcher: Option<libalembic::launcher::launcher::LauncherImpl>,
}

impl MainTab {
    pub fn default() -> Self {
        Self {
            sidebar_width: 200.0,
            news: News::default(),
            current_launcher: None,
        }
    }
}

impl Widget for &mut MainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.allocate_ui(
                    Vec2::new(
                        ui.available_width() - self.sidebar_width,
                        ui.available_height(),
                    ),
                    |ui| ui.add(&mut self.news),
                );
            });
            ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                ui.set_max_width(self.sidebar_width);

                // TODO: Wrap this up somewhere else outside of the UI render code
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

                // Client Info
                let client_info = if let Some(s) = ui.data_mut(|data| {
                    data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                }) {
                    let settings = s.lock().unwrap();

                    Some(settings.client.clone())
                } else {
                    None
                };

                // Server Info
                let server_info = if let Some(s) = ui.data_mut(|data| {
                    data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                }) {
                    let settings = s.lock().unwrap();

                    match settings.selected_server {
                        Some(index) => Some(settings.servers[index].clone()),
                        None => None,
                    }
                } else {
                    None
                };

                // Account Info
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

                // Alembic DLL Path
                // let dll_info = if let Some(s) = ui.data_mut(|data| {
                //     data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                // }) {
                //     let settings = s.lock().unwrap();

                //     Some(settings.dll.clone())
                // } else {
                //     None
                // };
                let dll_info = Some(DllInfo {
                    dll_path: r"target\i686-pc-windows-msvc/debug/alembic.dll".to_string(),
                });

                if ui.button("Inject").clicked() {
                    println!("Inject clicked, launch is {:?}", self.current_launcher);

                    if self.current_launcher.is_none() {
                        println!("Must launch first.")
                    } else {
                        match self.current_launcher.as_mut().unwrap() {
                            libalembic::launcher::launcher::LauncherImpl::WindowsLauncher(
                                windows_launcher,
                            ) => match windows_launcher.inject() {
                                Ok(_) => println!("inject success"),
                                Err(e) => println!("inject error: {:?}", e),
                            },
                            libalembic::launcher::launcher::LauncherImpl::WineLauncher(
                                wine_launcher,
                            ) => todo!(),
                            libalembic::launcher::launcher::LauncherImpl::NoopLauncher(
                                noop_launcher,
                            ) => todo!(),
                        }
                    }
                }

                if ui.button("Eject").clicked() {
                    println!("Eject clicked");

                    if self.current_launcher.is_none() {
                        println!("Must launch first.")
                    } else {
                        match self.current_launcher.as_mut().unwrap() {
                            LauncherImpl::WindowsLauncher(windows_launcher) => {
                                match windows_launcher.eject() {
                                    Ok(_) => println!("Eject succes"),
                                    Err(e) => println!("Eject error: {:?}", e),
                                }
                            }
                            LauncherImpl::WineLauncher(wine_launcher) => todo!(),
                            LauncherImpl::NoopLauncher(noop_launcher) => todo!(),
                        }
                    }
                }

                if ui.button("FindOrLaunch").clicked() {
                    self.current_launcher =
                        Some(LauncherImpl::WindowsLauncher(WindowsLauncher::new(
                            client_info.clone().unwrap(),
                            server_info.clone().unwrap(),
                            account_info.clone().unwrap(),
                            dll_info.unwrap().clone(),
                        )));

                    println!(
                        "FindOrLaunch clicked with launcher {:?}",
                        self.current_launcher
                    );

                    match &mut self.current_launcher {
                        Some(launcher) => match launcher {
                            LauncherImpl::WindowsLauncher(windows_launcher) => {
                                match windows_launcher.find_or_launch() {
                                    Ok(info) => println!(
                                        "FindOrLaunch success, proc is {:?}, client is {:?}",
                                        info, windows_launcher.client
                                    ),
                                    Err(e) => println!("FindOrLaunch error: {:?}", e),
                                }
                            }
                            LauncherImpl::WineLauncher(wine_launcher) => todo!(),
                            LauncherImpl::NoopLauncher(noop_launcher) => todo!(),
                        },
                        None => todo!(),
                    }
                }

                // ui.add_enabled_ui(can_launch, |ui| {
                //     if ui
                //         .add_sized(Vec2::new(140.0, 70.0), Button::new("Launch"))
                //         .clicked()
                //     {
                //         println!("Launch clicked.");

                //         match try_launch(&client_info, &server_info, &account_info, dll_path) {
                //             Ok(val) => {
                //                 println!("Launch succeeded. Launched pid is {val}!");

                //                 if let Some(backend_ref) = ui.data_mut(|data| {
                //                     data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new(
                //                         "backend",
                //                     ))
                //                 }) {
                //                     let mut backend = backend_ref.lock().unwrap();

                //                     backend.client = Some(Client { pid: val });
                //                     backend.is_injected = true;
                //                 }
                //             }
                //             Err(error) => {
                //                 println!("Launch failed with error: {error}");

                //                 if let Some(backend_ref) = ui.data_mut(|data| {
                //                     data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new(
                //                         "backend",
                //                     ))
                //                 }) {
                //                     let mut backend = backend_ref.lock().unwrap();

                //                     backend.current_modal = Some(AppModal {
                //                         title: "Error Launching".to_string(),
                //                         text: format!("The following error was encountered when trying to launch:\n\n{}\n\nPlease check your settings and try again.", error.to_string()),
                //                     });
                //                 }
                //             }
                //         }

                //         println!("Launch process is over.");
                //     }
                // });

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
