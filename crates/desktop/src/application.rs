use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    backend::{Backend, ChatMessage, LogEntry, PacketInfo},
    fetching::{BackgroundFetchRequest, BackgroundFetchUpdateMessage},
    widgets::{about::About, settings::Settings, tabs::TabContainer, wizard::Wizard},
};

use eframe::egui::{self, vec2, Align, Align2, Layout};
use libalembic::{msg::client_server::ClientServerMessage, settings::AlembicSettings};
use ringbuffer::RingBuffer;
use tokio::sync::mpsc::{error::TryRecvError, Receiver};

// Main tabs
#[derive(Clone)]
pub enum AppPage {
    Wizard,
    Main,
    About,
    Settings,
}

#[derive(Clone)]
pub enum WizardPage {
    Start,
    Client,
    Done,
}
pub struct Application {
    tab_container: TabContainer,
    wizard: Wizard,
    about: About,
    settings: Settings,
    client_server_rx: Arc<tokio::sync::Mutex<Receiver<ClientServerMessage>>>,
    background_fetch_sender: std::sync::mpsc::Sender<BackgroundFetchRequest>,
    background_update_receiver: std::sync::mpsc::Receiver<BackgroundFetchUpdateMessage>,
}

impl Application {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        client_server_rx: Arc<tokio::sync::Mutex<Receiver<ClientServerMessage>>>,
        background_fetch_sender: std::sync::mpsc::Sender<BackgroundFetchRequest>,
        background_update_receiver: std::sync::mpsc::Receiver<BackgroundFetchUpdateMessage>,
    ) -> Self {
        // Inject a new, shared Backend object into the egui_ctx (Context)
        let backend: Arc<Mutex<Backend>> = Arc::new(Mutex::new(Backend::new()));
        cc.egui_ctx
            .data_mut(|data| data.insert_persisted(egui::Id::new("backend"), backend));

        // Inject a new, shared Settings object into the egui_ctx (Context)
        let alembic_settings: Arc<Mutex<AlembicSettings>> =
            Arc::new(Mutex::new(AlembicSettings::new()));
        match alembic_settings.lock().unwrap().load() {
            Ok(_) => {}
            Err(error) => eprintln!("Error loading settings: {error}"),
        }

        cc.egui_ctx
            .data_mut(|data| data.insert_persisted(egui::Id::new("settings"), alembic_settings));

        // Background data fetching
        cc.egui_ctx.data_mut(|data| {
            data.insert_persisted(
                egui::Id::new("background_fetch_sender"),
                background_fetch_sender.clone(),
            );
        });

        // Set up view state
        //
        // Determine if we should show the Wizard page or just jump straight into
        // TODO: Do I really have to pull from the data context? Anything else
        // ends up giving me borrow-checker errors
        let is_configured = cc.egui_ctx.data_mut(|data| {
            if let Some(val) =
                data.get_persisted::<Arc<Mutex<AlembicSettings>>>(egui::Id::new("settings"))
            {
                val.lock().unwrap().is_configured
            } else {
                false
            }
        });

        match is_configured {
            true => {
                cc.egui_ctx.memory_mut(|mem| {
                    mem.data
                        .insert_persisted(egui::Id::new("app_page"), AppPage::Main);
                });
            }
            false => {
                cc.egui_ctx.memory_mut(|mem| {
                    mem.data
                        .insert_persisted(egui::Id::new("app_page"), AppPage::Wizard);
                    mem.data
                        .insert_persisted(egui::Id::new("wizard_page"), WizardPage::Start);
                });
            }
        }

        // Fonts
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "InterVariable".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/InterVariable.ttf"
            ))),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "InterVariable".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            tab_container: TabContainer::new(),
            wizard: Wizard::new(),
            about: About::new(),
            settings: Settings::new(),
            client_server_rx: client_server_rx,
            background_fetch_sender,
            background_update_receiver,
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        let mut current_app_page = AppPage::Wizard;

        ctx.memory_mut(|mem| {
            if let Some(val) = mem.data.get_persisted::<AppPage>(egui::Id::new("app_page")) {
                current_app_page = val;
            }
        });

        match current_app_page {
            AppPage::Main => {
                egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.add(egui::Button::new("Settings")).clicked() {
                                ui.memory_mut(|mem| {
                                    mem.data.insert_persisted(
                                        egui::Id::new("app_page"),
                                        AppPage::Settings,
                                    )
                                });
                                ui.close_menu();
                            }
                            if ui.add(egui::Button::new("Exit")).clicked() {
                                ui.close_menu();
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });

                        ui.menu_button("Help", |ui: &mut egui::Ui| {
                            if ui.add(egui::Button::new("About")).clicked() {
                                ui.close_menu();
                                ui.memory_mut(|mem| {
                                    mem.data
                                        .insert_persisted(egui::Id::new("app_page"), AppPage::About)
                                });
                            }
                        });
                    });
                });

                egui::TopBottomPanel::bottom("status")
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            if let Some(backend) = ui.data_mut(|data| {
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            }) {
                                let b = backend.lock().unwrap();

                                if let Some(msg) = &b.status_message {
                                    ui.label(msg)
                                } else {
                                    ui.label("Ready".to_string())
                                }
                            } else {
                                ui.label("Failed to reach application backend.")
                            }
                        });
                    });

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.tab_container);
                });
            }
            AppPage::Settings => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.settings);
                });
            }
            AppPage::Wizard => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.wizard);
                });
            }
            AppPage::About => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.about);
                });
            }
        }

        let current_modal = if let Some(backend_ref) =
            ctx.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            let backend = backend_ref.lock().unwrap();

            backend.current_modal.clone()
        } else {
            None
        };

        if current_modal.is_some() {
            let modal = current_modal.unwrap();

            egui::Window::new(modal.title)
                .enabled(true)
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_max_width(240.0); // Adjust this value as needed

                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        ui.label(modal.text);
                        ui.add_space(16.0);
                        ui.with_layout(Layout::top_down(Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                if let Some(backend_ref) = ctx.data_mut(|data| {
                                    data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new(
                                        "backend",
                                    ))
                                }) {
                                    let mut backend = backend_ref.lock().unwrap();

                                    backend.current_modal = None;
                                }
                            }
                        });
                    });
                });
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle channel
        loop {
            match self.client_server_rx.try_lock().unwrap().try_recv() {
                Ok(msg) => match msg {
                    ClientServerMessage::ClientInjected() => {
                        ctx.data_mut(|data| {
                            if let Some(backend) =
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            {
                                if let Ok(mut backend) = backend.lock() {
                                    backend.injected = true;
                                }
                            }
                        });
                    }
                    ClientServerMessage::ClientEjected() => {
                        ctx.data_mut(|data| {
                            if let Some(backend) =
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            {
                                if let Ok(mut backend) = backend.lock() {
                                    backend.is_injected = false;
                                }
                            }
                        });
                    }

                    ClientServerMessage::AppendLog(value) => {
                        let log = LogEntry {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            message: value,
                        };
                        ctx.data_mut(|data| {
                            if let Some(backend) =
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            {
                                if let Ok(mut backend) = backend.lock() {
                                    backend.logs.push(log);
                                }
                            }
                        });
                    }
                    ClientServerMessage::HandleSendTo(vec) => {
                        ctx.data_mut(|data| {
                            if let Some(backend) =
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            {
                                if let Ok(mut backend) = backend.lock() {
                                    // Increment statistics
                                    backend.statistics.network.outgoing_count += 1;

                                    // Append new packet
                                    let packet = PacketInfo {
                                        index: backend.statistics.network.outgoing_count,
                                        timestamp: SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs(),
                                        data: vec,
                                    };
                                    backend.packets_outgoing.push(packet);
                                }
                            }
                        });
                    }
                    ClientServerMessage::HandleRecvFrom(vec) => {
                        ctx.data_mut(|data| {
                            if let Some(backend) =
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            {
                                if let Ok(mut backend) = backend.lock() {
                                    // Increment statistics
                                    backend.statistics.network.incoming_count += 1;

                                    // Append new packet
                                    let packet = PacketInfo {
                                        index: backend.statistics.network.incoming_count,
                                        timestamp: SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs(),
                                        data: vec,
                                    };
                                    backend.packets_incoming.push(packet);
                                }
                            }
                        });
                    }
                    ClientServerMessage::HandleAddTextToScroll(text) => {
                        ctx.data_mut(|data| {
                            if let Some(backend) =
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            {
                                if let Ok(mut backend) = backend.lock() {
                                    let message = ChatMessage {
                                        index: backend.chat_messages.len(),
                                        timestamp: SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs(),
                                        text,
                                    };
                                    backend.chat_messages.push(message);
                                }
                            }
                        });
                    }
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    eprintln!("Channel disconnected");
                    break;
                }
            }
        }

        loop {
            match self.background_update_receiver.try_recv() {
                Ok(update) => match update {
                    BackgroundFetchUpdateMessage::NewsUpdate(wrapper) => {
                        ctx.data_mut(|data| {
                            if let Some(backend) =
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            {
                                backend.lock().unwrap().news = wrapper;
                            }
                        });
                    }
                    BackgroundFetchUpdateMessage::CommunityServersUpdate(wrapper) => {
                        ctx.data_mut(|data| {
                            if let Some(backend) =
                                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
                            {
                                backend.lock().unwrap().community_servers = wrapper;
                            }
                        });
                    }
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    eprintln!("Channel disconnected");
                    break;
                }
            }
        }

        self.ui(ctx);
    }
}
