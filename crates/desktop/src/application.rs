use std::{
    sync::Arc,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    backend::{Backend, ChatMessage, LogEntry, PacketInfo},
    widgets::{about::About, settings::Settings, tabs::TabContainer, wizard::Wizard},
};

use eframe::{
    egui::{self, Align, Align2, Layout},
    App,
};
use libalembic::{msg::client_server::ClientServerMessage, settings::AlembicSettings};
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
    show_about: bool,
    client_server_rx: Arc<tokio::sync::Mutex<Receiver<ClientServerMessage>>>,
}

impl Application {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        client_server_rx: Arc<tokio::sync::Mutex<Receiver<ClientServerMessage>>>,
    ) -> Self {
        // Inject a new, shared Backend object into the egui_ctx (Context)
        let backend: Arc<Mutex<Backend>> = Arc::new(Mutex::new(Backend::new()));
        cc.egui_ctx
            .data_mut(|data| data.insert_persisted(egui::Id::new("backend"), backend));

        // Inject a new, shared Settings object into the egui_ctx (Context)
        let settings: Arc<Mutex<AlembicSettings>> = Arc::new(Mutex::new(AlembicSettings::new()));
        match settings.lock().unwrap().load() {
            Ok(_) => {}
            Err(error) => eprintln!("Error loading settings: {error}"),
        }
        cc.egui_ctx
            .data_mut(|data| data.insert_persisted(egui::Id::new("settings"), settings));

        // Set up view state
        cc.egui_ctx.memory_mut(|mem| {
            mem.data
                .insert_persisted(egui::Id::new("app_page"), AppPage::Wizard);
            mem.data
                .insert_persisted(egui::Id::new("wizard_page"), WizardPage::Start);
        });

        Self {
            tab_container: TabContainer::new(),
            wizard: Wizard::new(),
            about: About::new(),
            settings: Settings::new(),
            show_about: false,
            client_server_rx: client_server_rx,
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
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle channel
        loop {
            match self.client_server_rx.try_lock().unwrap().try_recv() {
                Ok(msg) => match msg {
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
                                    let packet = PacketInfo {
                                        index: backend.packets_outgoing.len(),
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
                                    let packet = PacketInfo {
                                        index: backend.packets_incoming.len(),
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

        self.ui(ctx);
    }
}
