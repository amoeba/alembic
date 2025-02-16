use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use eframe::egui::{self, Id, Response, ScrollArea, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use libalembic::settings::{AlembicSettings, ServerInfo};

use crate::{
    backend::Backend,
    fetching::{BackgroundFetchRequest, FetchWrapper},
};

use super::components::centered_text;

pub struct AccountsCommunityServersTab {
    num_retry_attempts: usize,
    max_retry_attempts: usize,
    retry_interval: std::time::Duration,
    last_fetched: Option<std::time::SystemTime>,
    is_resolved: bool,
}

impl AccountsCommunityServersTab {
    pub fn default() -> Self {
        Self {
            num_retry_attempts: 0,
            max_retry_attempts: 3,
            retry_interval: std::time::Duration::from_secs(60),
            last_fetched: None,
            is_resolved: false,
        }
    }
}

impl Widget for &mut AccountsCommunityServersTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Handle fetching
            if !self.is_resolved {
                // First we check if the were were resolved externally
                if let Some(b) = ui
                    .data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(Id::new("backend")))
                {
                    let backend = b.lock().unwrap();

                    if matches!(
                        &backend.community_servers,
                        FetchWrapper::Success(_community_servers)
                    ) {
                        self.is_resolved = true;
                    }
                }

                // If we're due to check, check
                let retry_not_exhausted = self.max_retry_attempts > self.num_retry_attempts;
                let due_for_retry = self.last_fetched.is_none()
                    || self.last_fetched.unwrap() + self.retry_interval < SystemTime::now();

                if retry_not_exhausted && due_for_retry {
                    self.last_fetched = Some(SystemTime::now());

                    if let Some(sender) = ui.data_mut(|data| -> _ {
                        data.get_persisted::<std::sync::mpsc::Sender<BackgroundFetchRequest>>(
                            Id::new("background_fetch_sender"),
                        )
                    }) {
                        println!("Requesting we fetch servers list...");
                        sender
                            .send(BackgroundFetchRequest::FetchCommunityServers)
                            .expect("Failed to send fetch request");
                    }

                    self.num_retry_attempts += 1;
                }
            }

            // Regular UI code continues here
            ui.vertical(|ui| {
                ui.heading("Community Servers");
                ui.add_space(8.0);

                if let Some(b) = ui.data_mut(|data: &mut egui::util::IdTypeMap| {
                    data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                }) {
                    let mut backend = b.lock().unwrap();

                    match &backend.community_servers {
                        FetchWrapper::NotStarted => {
                            ui.label("Not yet fetched...");
                        }
                        FetchWrapper::Started => {
                            ui.label("Fetching...");
                        }
                        FetchWrapper::Retrying(_) => {
                            ui.label("Retrying...");
                        }
                        FetchWrapper::Success(servers) => {
                            if servers.servers.len() <= 0 {
                                ui.label("No servers to show");
                            } else {
                                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                                TableBuilder::new(ui)
                                    .striped(true) // Enable striped rows for readability
                                    .resizable(true) // Allow column resizing
                                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center)) // Cell layout
                                    .column(Column::auto()) // import
                                    .column(Column::auto()) // name
                                    .column(Column::auto()) // description
                                    .column(Column::auto()) // emu
                                    .column(Column::auto()) // host
                                    .column(Column::auto()) // port
                                    .column(Column::auto()) // type
                                    .column(Column::auto()) // status
                                    .column(Column::auto()) // website
                                    .column(Column::auto()) // discord
                                    .header(text_height, |mut header| {
                                        header.col(|ui| {
                                            ui.strong("Import");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Name");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Description");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Emulator");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Host");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Port");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Type");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Status");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Website");
                                        });

                                        header.col(|ui| {
                                            ui.strong("Discord");
                                        });
                                    })
                                    .body(|mut body| {
                                        for server in &servers.servers {
                                            body.row(text_height, |mut table_row| {
                                                table_row.col(|ui| {
                                                    if ui.button("Import").clicked() {
                                                        println!("TODO: Import");
                                                        if let Some(s) = ui.data_mut(|data: &mut egui::util::IdTypeMap| {
                                                            data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
                                                        }) {
                                                            let mut settings = s.lock().unwrap();

                                                            let to_import = ServerInfo {
                                                                name: server.name.clone(),
                                                                hostname: server.server_host.clone(),
                                                                port: server.server_port.clone(),
                                                            };

                                                            match settings.servers.iter().position(|s| s.name == server.name) {
                                                                Some(idx) => {
                                                                    settings.servers[idx] = to_import;
                                                                },
                                                                None =>  {
                                                                    settings.servers.push(to_import);
                                                                }
                                                            }

                                                            let _ = settings.save();
                                                        } else {

                                                        }
                                                    }
                                                });

                                                table_row.col(|ui| {
                                                    ui.label(&server.name);
                                                });

                                                table_row.col(|ui| {
                                                    ScrollArea::horizontal()
                                                        .min_scrolled_width(50.0)
                                                        .show(ui, |ui| {
                                                            ui.label(&server.description);
                                                        });
                                                });

                                                table_row.col(|ui| {
                                                    ui.label(&server.emu);
                                                });

                                                table_row.col(|ui| {
                                                    ui.label(&server.server_host);
                                                });

                                                table_row.col(|ui| {
                                                    ui.label(&server.server_port);
                                                });

                                                table_row.col(|ui| {
                                                    ui.label(&server.r#type);
                                                });

                                                table_row.col(|ui| {
                                                    ui.label(&server.status);
                                                });

                                                table_row.col(|ui| {
                                                    ui.hyperlink(
                                                        &server
                                                            .website_url
                                                            .clone()
                                                            .unwrap_or_else(|| "None".to_string()),
                                                    );
                                                });

                                                table_row.col(|ui| {
                                                    ui.hyperlink(
                                                        &server
                                                            .discord_url
                                                            .clone()
                                                            .unwrap_or_else(|| "None".to_string()),
                                                    );
                                                });
                                            });
                                        }
                                    });
                            }
                        }
                        FetchWrapper::Failed(error) => {
                            ui.label(format!("Error fetching community servers list: {}", error));
                        }
                    }
                } else {
                    ui.group(|ui| centered_text(ui, "Failed to reach application backend."));
                }
            })
            .response
        })
        .response
    }
}
