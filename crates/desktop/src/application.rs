use std::{
    sync::Arc,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use eframe::egui::{self, Response, ScrollArea, TextStyle, Ui, Widget};
use libalembic::rpc::GuiMessage;
use tokio::sync::mpsc::{error::TryRecvError, Receiver};

use crate::{
    backend::{Backend, LogEntry, PacketInfo},
    launch::try_launch,
    widgets::components::centered_text,
};

// Main tabs
enum TabContent {
    Main(MainTab),
    Developer(DeveloperTab),
}

struct MainTab {}

impl Widget for &mut MainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.group(|ui| {
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

struct DeveloperTab {
    tabs: Vec<DeveloperTabContent>,
    selected_tab: usize,
}

impl Widget for &mut DeveloperTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        DeveloperTabContent::Main(_) => "Main",
                        DeveloperTabContent::Network(_) => "Network",
                        DeveloperTabContent::Logs(_) => "Logs",
                    };

                    if ui
                        .selectable_label(self.selected_tab == index, label)
                        .clicked()
                    {
                        self.selected_tab = index;
                    }
                }
            });

            ui.separator();

            // Tab contents
            if let Some(tab) = self.tabs.get_mut(self.selected_tab) {
                match tab {
                    DeveloperTabContent::Main(tab) => {
                        ui.add(tab);
                    }
                    DeveloperTabContent::Network(tab) => {
                        ui.add(tab);
                    }
                    DeveloperTabContent::Logs(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}
enum DeveloperTabContent {
    Main(DeveloperMainTab),
    Network(DeveloperNetworkTab),
    Logs(DeveloperLogsTab),
}
struct DeveloperMainTab {}

impl Widget for &mut DeveloperMainTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.group(|ui| ui.label("Developer Main")).response
    }
}
struct DeveloperNetworkTab {
    selected_tab: usize,
    tabs: Vec<DeveloperNetworkTabContent>,
}

impl Widget for &mut DeveloperNetworkTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        DeveloperNetworkTabContent::Incoming(_) => "Incoming",
                        DeveloperNetworkTabContent::Outgoing(_) => "Outgoing",
                    };

                    if ui
                        .selectable_label(self.selected_tab == index, label)
                        .clicked()
                    {
                        self.selected_tab = index;
                    }
                }
            });

            ui.separator();

            // Tab contents
            if let Some(tab) = self.tabs.get_mut(self.selected_tab) {
                match tab {
                    DeveloperNetworkTabContent::Incoming(tab) => {
                        ui.add(tab);
                    }
                    DeveloperNetworkTabContent::Outgoing(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}

struct DeveloperLogsTab {}

impl Widget for &mut DeveloperLogsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        // WIP: Here's how we access the backend for now
        // let state_id = ui.id();
        // let backend: Arc<Mutex<Backend>> = ui.data(|data| data.get_temp(state_id).unwrap());

        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            ui.group(|ui| {
                if backend.lock().unwrap().logs.len() <= 0 {
                    centered_text(ui, "No logs yet.");
                } else {
                    let n_logs = backend.lock().unwrap().logs.len();
                    let text_style = TextStyle::Body;
                    let total_rows = ui.text_style_height(&text_style);

                    ui.vertical(|ui| {
                        ScrollArea::vertical().auto_shrink(false).show_rows(
                            ui,
                            total_rows,
                            n_logs,
                            |ui, row_range| {
                                for row in row_range {
                                    let text =
                                        format!("{}", backend.lock().unwrap().logs[row].message);
                                    ui.label(text);
                                }
                            },
                        );
                    });
                }
            })
            .response
        } else {
            ui.group(|ui| centered_text(ui, "ERROR")).response
        }
    }
}

enum DeveloperNetworkTabContent {
    Incoming(DeveloperNetworkIncomingTab),
    Outgoing(DeveloperNetworkOutgoingTab),
}

struct DeveloperNetworkIncomingTab {}

impl Widget for &mut DeveloperNetworkIncomingTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.group(|ui| ui.label("NetworkIncoming")).response
    }
}

struct DeveloperNetworkOutgoingTab {}

impl Widget for &mut DeveloperNetworkOutgoingTab {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.group(|ui| ui.label("NetworkOutgoing")).response
    }
}

struct TabContainer {
    tabs: Vec<TabContent>,
    selected_tab: usize,
}

impl TabContainer {
    fn new() -> Self {
        Self {
            tabs: vec![
                TabContent::Main(MainTab {}),
                TabContent::Developer(DeveloperTab {
                    selected_tab: 0,
                    tabs: vec![
                        DeveloperTabContent::Main(DeveloperMainTab {}),
                        DeveloperTabContent::Network(DeveloperNetworkTab {
                            selected_tab: 0,
                            tabs: vec![
                                DeveloperNetworkTabContent::Incoming(
                                    DeveloperNetworkIncomingTab {},
                                ),
                                DeveloperNetworkTabContent::Outgoing(
                                    DeveloperNetworkOutgoingTab {},
                                ),
                            ],
                        }),
                        DeveloperTabContent::Logs(DeveloperLogsTab {}),
                    ],
                }),
            ],
            selected_tab: 0,
        }
    }
}

impl Widget for &mut TabContainer {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            // Tabs
            ui.horizontal(|ui| {
                for (index, tab) in self.tabs.iter().enumerate() {
                    let label = match tab {
                        TabContent::Main(_) => "Settings",
                        TabContent::Developer(_) => "Developer",
                    };

                    if ui
                        .selectable_label(self.selected_tab == index, label)
                        .clicked()
                    {
                        self.selected_tab = index;
                    }
                }
            });

            ui.separator();

            // Tab contents
            if let Some(tab) = self.tabs.get_mut(self.selected_tab) {
                match tab {
                    TabContent::Main(tab) => {
                        ui.add(tab);
                    }
                    TabContent::Developer(tab) => {
                        ui.add(tab);
                    }
                }
            }
        })
        .response
    }
}

pub struct Application {
    tab_container: TabContainer,
    gui_rx: Arc<tokio::sync::Mutex<Receiver<GuiMessage>>>,
}

impl Application {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        gui_rx: Arc<tokio::sync::Mutex<Receiver<GuiMessage>>>,
    ) -> Self {
        // TODO: VERY WIP
        let backend = Arc::new(Mutex::new(Backend::new()));
        cc.egui_ctx
            .data_mut(|data| data.insert_persisted(egui::Id::new("backend"), backend));

        Self {
            tab_container: TabContainer::new(),
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
                        // self.show_about = true;
                    }
                });
            });
        });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.tab_container);
        });
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
                        // TODO: Bring this back
                        println!("Gui got a packet data");
                        // let packet = PacketInfo {
                        //     index: backend.packets_incoming.len(),
                        //     timestamp: SystemTime::now()
                        //         .duration_since(UNIX_EPOCH)
                        //         .unwrap()
                        //         .as_secs(),
                        //     data: vec,
                        // };
                        // backend.packets_incoming.push(packet);
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
