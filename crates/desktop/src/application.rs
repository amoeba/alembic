use std::{
    sync::Arc,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    backend::{Backend, LogEntry, PacketInfo},
    widgets::tabs::TabContainer,
};
use eframe::egui::{self, Align, Align2, Layout};
use libalembic::rpc::GuiMessage;
use tokio::sync::mpsc::{error::TryRecvError, Receiver};

// Main tabs

pub struct Application {
    tab_container: TabContainer,
    show_about: bool,
    gui_rx: Arc<tokio::sync::Mutex<Receiver<GuiMessage>>>,
}

impl Application {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        gui_rx: Arc<tokio::sync::Mutex<Receiver<GuiMessage>>>,
    ) -> Self {
        // Inject a new, shared Backend object into the egui_ctx (Context)
        let backend = Arc::new(Mutex::new(Backend::new()));
        cc.egui_ctx
            .data_mut(|data| data.insert_persisted(egui::Id::new("backend"), backend));

        Self {
            tab_container: TabContainer::new(),
            show_about: false,
            gui_rx: gui_rx,
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        // Menu bar
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
                        self.show_about = true;
                    }
                });
            });
        });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.tab_container);
        });

        if self.show_about {
            egui::Window::new("About")
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .show(ctx, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.add(
                            egui::Image::new(egui::include_image!("../assets/logo.png"))
                                .max_width(128.0),
                        );
                        ui.heading("Alembic");
                        ui.add_space(16.0);
                        ui.label("Version 0.1.0");
                        ui.add_space(16.0);
                        ui.label("Copyright © 2025 Bryce Mecum");
                        ui.add_space(16.0);
                        use egui::special_emojis::GITHUB;
                        ui.hyperlink_to(
                            format!("{GITHUB} alembic on GitHub"),
                            "https://github.com/amoeba/alembic",
                        );
                        ui.add_space(16.0);
                        if ui.button("Okay").clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle channel
        loop {
            match self.gui_rx.try_lock().unwrap().try_recv() {
                Ok(msg) => match msg {
                    GuiMessage::Hello(_) => {
                        println!("GUI got Hello");
                    }
                    GuiMessage::UpdateString(value) => {
                        println!("GUI got UpdateString with value {value}");
                    }
                    GuiMessage::AppendLog(value) => {
                        println!("GUI got AppendLog with value {value}");
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
                    GuiMessage::SendTo(vec) => {
                        println!("Gui got a packet data");
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
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    println!("Channel disconnected");
                    break;
                }
            }
        }

        self.ui(ctx);
    }
}
