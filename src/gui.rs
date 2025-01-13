mod inject;
mod launch;
mod rpc;

use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use eframe::egui::{self, ScrollArea, TextStyle, Ui};
use futures::{future, StreamExt};
use tarpc::{
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use tokio::sync::{
    mpsc::{channel, error::TryRecvError, Receiver},
    Mutex,
};

use launch::Launcher;
use rpc::{spawn, GuiMessage, HelloServer, PaintMessage, World};

fn main() -> eframe::Result {
    env_logger::init();

    // Channel: GUI
    let (gui_tx, gui_rx) = channel::<GuiMessage>(32);
    let gui_rx_ref = Arc::new(Mutex::new(gui_rx));
    let gui_tx_ref = Arc::new(Mutex::new(gui_tx));

    // Channel: Painting
    let (paint_tx, paint_rx) = channel::<PaintMessage>(32);
    let paint_rx_ref = Arc::new(Mutex::new(paint_rx));
    let paint_tx_ref = Arc::new(Mutex::new(paint_tx));

    // tarpc
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.spawn(async move {
        let addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

        let listener = tarpc::serde_transport::tcp::listen(&addr, Json::default)
            .await
            .expect("whoops!");
        listener
            // Ignore accept errors.
            .filter_map(|r| future::ready(r.ok()))
            .map(server::BaseChannel::with_defaults)
            .map(|channel| {
                let server = HelloServer {
                    paint_tx: Arc::clone(&paint_tx_ref),
                    gui_tx: Arc::clone(&gui_tx_ref),
                };
                channel.execute(server.serve()).for_each(spawn)
            })
            .buffer_unordered(10)
            .for_each(|_| async {})
            .await;
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    let app = Application::new(Arc::clone(&gui_rx_ref));

    // Pass a cloned paint_rx into the app so we can handle repaints
    let app_paint_rx = Arc::clone(&paint_rx_ref);

    eframe::run_native(
        "Alembic",
        options,
        Box::new(|cc| {
            let frame = cc.egui_ctx.clone();

            thread::spawn(move || {
                loop {
                    match app_paint_rx.try_lock().unwrap().try_recv() {
                        Ok(msg) => match msg {
                            PaintMessage::RequestRepaint => {
                                println!("Repaint request received!");
                                frame.request_repaint();
                            }
                        },
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => {
                            println!("Channel disconnected");
                            break;
                        }
                    }

                    // ? 60FPS
                    thread::sleep(Duration::from_millis(16));
                }
            });

            Ok(Box::new(app))
        }),
    )
}

fn try_launch() -> anyhow::Result<()> {
    let mut launcher = Launcher::new();
    launcher.find_or_launch()?;
    launcher.inject()?;

    Ok(())
}

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

struct PacketInfo {
    index: usize,
    timestamp: u64,
    data: Vec<u8>,
}

struct Application {
    current_tab: Tab,
    current_developer_tab: DeveloperTab,
    current_developer_network_tab: DeveloperNetworkTab,
    string: String,
    logs: Vec<String>,
    packets: Vec<PacketInfo>,
    incoming_packet_count: usize,
    outgoing_packet_count: usize,
    selected_incoming_packet: Option<usize>,
    selected_outgoing_packet: Option<usize>,
    gui_rx: Arc<Mutex<Receiver<GuiMessage>>>,
}

impl Application {
    pub fn new(gui_rx: Arc<Mutex<Receiver<GuiMessage>>>) -> Self {
        Self {
            current_tab: Tab::Main,
            current_developer_tab: DeveloperTab::Main,
            current_developer_network_tab: DeveloperNetworkTab::Incoming,
            string: "Unset".to_string(),
            logs: vec![],
            packets: vec![],
            incoming_packet_count: 0,
            outgoing_packet_count: 0,
            selected_incoming_packet: None,
            selected_outgoing_packet: None,
            gui_rx,
        }
    }

    fn main(self: &mut Self, ui: &mut Ui) {
        if ui.add(egui::Button::new("Test")).clicked() {
            println!("clicked");

            match try_launch() {
                Ok(_) => println!("Launch success"),
                Err(error) => println!("Launch failure: {error}"),
            }
        }
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
        let n_logs = self.logs.len();
        let text_style = TextStyle::Body;
        let total_rows = ui.text_style_height(&text_style);

        ui.heading("Logs");
        ui.vertical(|ui| {
            ScrollArea::vertical().auto_shrink(false).show_rows(
                ui,
                total_rows,
                n_logs,
                |ui, row_range| {
                    for row in row_range {
                        let text = format!("{}", self.logs[row]);
                        ui.label(text);
                    }
                },
            );
        });
    }

    fn developer_network_incoming(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            columns[0].vertical(|ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for (index, item) in self.packets.iter().enumerate() {
                        if ui.button(item.timestamp.to_string()).clicked() {
                            self.selected_incoming_packet = Some(index);
                        }
                    }
                });
            });

            columns[1].vertical(|ui| {
                if let Some(index) = self.selected_incoming_packet {
                    ui.label(format!("{:?}", self.packets[index].data));
                } else {
                    ui.label("Select an item from the left panel");
                }
            });
        });

        // let text_style = TextStyle::Body;
        // let row_height = ui.text_style_height(&text_style);
        // let total_rows = self.packets.len();
        // ui.add(SidePanel::left("network_incoming_left")).show(|ui| {
        //     ScrollArea::vertical().auto_shrink(false).show_rows(
        //         ui,
        //         row_height,
        //         total_rows,
        //         |ui, row_range| {
        //             for row in row_range {
        //                 let text = format!("{:?}", self.packets[row]);
        //                 ui.label(text);
        //             }
        //         },
        //     );
        // });
    }

    fn developer_network_outgoing(&self, ui: &mut Ui) {
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        let total_rows = self.packets.len();

        ui.heading("Outgoing Packets");
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
                        self.logs.push(value);
                    }
                    GuiMessage::SendTo(vec) => {
                        println!("Gui got a packet data");
                        let packet = PacketInfo {
                            index: 0,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            data: vec,
                        };
                        self.packets.push(packet);
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
                    if ui.add(egui::Button::new("Testing")).clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui: &mut egui::Ui| {
                    if ui.add(egui::Button::new("About")).clicked() {
                        ui.close_menu();
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
    }
}
