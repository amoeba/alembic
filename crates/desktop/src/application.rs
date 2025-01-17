use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use eframe::egui::{self, Align, Align2, Layout, ScrollArea, TextStyle, Ui};
use libalembic::rpc::GuiMessage;
use tokio::sync::{
    mpsc::{error::TryRecvError, Receiver},
    Mutex,
};

use crate::{
    backend::{Backend, LogEntry, PacketInfo},
    components::centered_text,
    try_launch,
};

#[derive(PartialEq)]
enum Tab {
    Main,
    Developer,
}

#[derive(PartialEq)]
enum DeveloperTab {
    Main,
    Network,
    Logs,
}

#[derive(PartialEq)]
enum DeveloperNetworkTab {
    Incoming,
    Outgoing,
}

#[derive(Debug, PartialEq)]
enum ComboOptions {
    First,
    Second,
    Third,
}

pub struct Application {
    backend: Backend,
    current_tab: Tab,
    current_developer_tab: DeveloperTab,
    current_developer_network_tab: DeveloperNetworkTab,
    string: String,
    selected_incoming_packet: Option<usize>,
    selected_outgoing_packet: Option<usize>,
    gui_rx: Arc<Mutex<Receiver<GuiMessage>>>,
    show_about: bool,
}

impl Application {
    pub fn new(gui_rx: Arc<Mutex<Receiver<GuiMessage>>>) -> Self {
        Self {
            backend: Backend::new(),
            current_tab: Tab::Main,
            current_developer_tab: DeveloperTab::Main,
            current_developer_network_tab: DeveloperNetworkTab::Incoming,
            string: "Unset".to_string(),
            selected_incoming_packet: None,
            selected_outgoing_packet: None,
            gui_rx,
            show_about: false,
        }
    }

    fn main(self: &mut Self, ui: &mut Ui) {
        if ui.add(egui::Button::new("Launch")).clicked() {
            println!("Launch clicked.");

            match try_launch() {
                Ok(_) => println!("Launch succeeded."),
                Err(error) => println!("Launch failed with error: {error}"),
            }

            println!("Launch completed.");
        }
        let mut selected = ComboOptions::First;
        egui::ComboBox::from_label("Select one!")
            .selected_text(format!("{:?}", selected))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, ComboOptions::First, "First");
                ui.selectable_value(&mut selected, ComboOptions::Second, "Second");
                ui.selectable_value(&mut selected, ComboOptions::Third, "Third");
            });
    }

    fn developer(self: &mut Self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_developer_tab, DeveloperTab::Main, "Main");
            ui.selectable_value(
                &mut self.current_developer_tab,
                DeveloperTab::Network,
                "Network",
            );
            ui.selectable_value(&mut self.current_developer_tab, DeveloperTab::Logs, "Logs");
        });

        ui.separator();

        match self.current_developer_tab {
            DeveloperTab::Main => self.developer_main(ui),
            DeveloperTab::Network => self.developer_network(ui),
            DeveloperTab::Logs => self.developer_logs(ui),
        }
        // ui.heading("Debugging");
        // ui.horizontal(|ui| {
        //     let string_label = ui.label("String: ");
        //     ui.text_edit_singleline(&mut self.string)
        //         .labelled_by(string_label.id);
        // });
    }

    fn developer_main(self: &mut Self, ui: &mut Ui) {
        ui.heading("Developer Main");
    }

    fn developer_network(self: &mut Self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.current_developer_network_tab,
                DeveloperNetworkTab::Incoming,
                "Incoming",
            );
            ui.selectable_value(
                &mut self.current_developer_network_tab,
                DeveloperNetworkTab::Outgoing,
                "Outgoing",
            );
        });

        ui.separator();

        match self.current_developer_network_tab {
            DeveloperNetworkTab::Incoming => self.developer_network_incoming(ui),
            DeveloperNetworkTab::Outgoing => self.developer_network_outgoing(ui),
        }
    }

    fn developer_logs(&self, ui: &mut Ui) {
        if self.backend.logs.len() <= 0 {
            centered_text(ui, "No logs yet.");
        } else {
            let n_logs = self.backend.logs.len();
            let text_style = TextStyle::Body;
            let total_rows = ui.text_style_height(&text_style);

            ui.vertical(|ui| {
                ScrollArea::vertical().auto_shrink(false).show_rows(
                    ui,
                    total_rows,
                    n_logs,
                    |ui, row_range| {
                        for row in row_range {
                            let text = format!("{}", self.backend.logs[row].message);
                            ui.label(text);
                        }
                    },
                );
            });
        }
    }

    fn developer_network_incoming(&mut self, ui: &mut Ui) {
        if self.backend.packets_incoming.len() <= 0 {
            centered_text(ui, "No incoming packets yet.");
        } else {
            // TODO: Use show_rows() here too
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (index, item) in self.backend.packets_incoming.iter().enumerate() {
                            if ui.button(item.timestamp.to_string()).clicked() {
                                self.selected_incoming_packet = Some(index);
                            }
                        }
                    });
                });

                columns[1].vertical(|ui| {
                    if let Some(index) = self.selected_incoming_packet {
                        ui.label(format!("{:?}", self.backend.packets_incoming[index].data));
                    }
                });
            });
        }
    }

    fn developer_network_outgoing(&mut self, ui: &mut Ui) {
        if self.backend.packets_outgoing.len() <= 0 {
            centered_text(ui, "No outgoing packets yet.");
        } else {
            // TODO: Use show_rows() here too
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (index, item) in self.backend.packets_outgoing.iter().enumerate() {
                            if ui.button(item.timestamp.to_string()).clicked() {
                                self.selected_outgoing_packet = Some(index);
                            }
                        }
                    });
                });

                columns[1].vertical(|ui| {
                    if let Some(index) = self.selected_outgoing_packet {
                        ui.label(format!("{:?}", self.backend.packets_outgoing[index].data));
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
                        self.string = value.to_string();
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
                        self.backend.logs.push(log);
                    }
                    GuiMessage::SendTo(vec) => {
                        println!("Gui got a packet data");
                        let packet = PacketInfo {
                            index: self.backend.packets_incoming.len(),
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            data: vec,
                        };
                        self.backend.packets_incoming.push(packet);
                    }
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    println!("Channel disconnected");
                    break;
                }
            }
        }

        // Handle UI
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Main, "Main");
                ui.selectable_value(&mut self.current_tab, Tab::Developer, "Developer");
            });

            ui.separator();

            match self.current_tab {
                Tab::Main => self.main(ui),
                Tab::Developer => self.developer(ui),
            }
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
