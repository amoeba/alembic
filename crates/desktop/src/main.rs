#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod backend;
mod fetching;
mod launch;
mod simulator;
mod widgets;

use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
    thread,
    time::Duration,
};

use application::Application;
use backend::News;
use eframe::egui::IconData;
use fetching::{fetch_news, BackgroundFetchRequest, BackgroundFetchUpdateMessage, FetchWrapper};
use futures::{future, StreamExt};
use libalembic::{
    msg::{client_server::ClientServerMessage, server_gui::ServerGuiMessage},
    rpc::{spawn, HelloServer, World},
};
use tarpc::{
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use tokio::sync::{
    mpsc::{channel, error::TryRecvError},
    Mutex,
};

fn main() -> eframe::Result {
    env_logger::init();

    // Channel: Client (i.e., plugin) to Server
    let (client_server_tx, client_server_rx) = channel::<ClientServerMessage>(32);
    let client_server_tx_ref = Arc::new(Mutex::new(client_server_tx));
    let client_server_rx_ref = Arc::new(Mutex::new(client_server_rx));

    // Channel: Server to GUI
    let (server_gui_tx, server_gui_rx) = channel::<ServerGuiMessage>(32);
    let server_gui_tx_ref = Arc::new(Mutex::new(server_gui_tx));
    let server_gui_rx_ref = Arc::new(Mutex::new(server_gui_rx));

    // Channels for background data fetching
    let (background_fetch_sender, fetch_receiver) =
        std::sync::mpsc::channel::<BackgroundFetchRequest>();
    let (update_sender, background_update_receiver) =
        std::sync::mpsc::channel::<BackgroundFetchUpdateMessage>();

    thread::spawn(move || {
        while let Ok(request) = fetch_receiver.recv() {
            match request {
                BackgroundFetchRequest::FetchNews => {
                    update_sender
                        .send(BackgroundFetchUpdateMessage::NewsUpdate(
                            FetchWrapper::Started,
                        ))
                        .expect("Failed to send BackgroundFetchUpdateMessage::NewsUpdate. This is a serious bug and should be reported.");

                    match fetch_news() {
                        Ok(news) => {
                            update_sender
                                .send(BackgroundFetchUpdateMessage::NewsUpdate(FetchWrapper::Success(News {entries: news.news.records }))).expect(
                            "Failed to send BackgroundFetchUpdateMessage::NewsUpdate. This is a serious bug and should be reported.");
                        }
                        Err(err) => {
                            update_sender
                                .send(BackgroundFetchUpdateMessage::NewsUpdate(FetchWrapper::Failed(err.to_string().into()))).expect(
                            "Failed to send BackgroundFetchUpdateMessage::NewsUpdate. This is a serious bug and should be reported.",
                        );
                        }
                    }
                }
            }
        }
    });

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
                    server_gui_tx: Arc::clone(&server_gui_tx_ref),
                    client_server_tx: Arc::clone(&client_server_tx_ref),
                };
                channel.execute(server.serve()).for_each(spawn)
            })
            .buffer_unordered(10)
            .for_each(|_| async {})
            .await;
    });

    // App Icon
    let icon_data: Option<Arc<IconData>> = if cfg!(target_os = "windows") {
        let path = if cfg!(debug_assertions) {
            r"crates\\desktop\\assets\logo.png".to_string()
        } else {
            "logo.png".to_string()
        };

        let image =
            image::open(path).expect("Failed to load app icon. Please report this as a bug:");
        let (icon_rgba, icon_width, icon_height) = {
            let buf = image.into_rgba8();
            let (width, height) = buf.dimensions();

            (buf.into_raw(), width, height)
        };
        Some(Arc::new(IconData {
            rgba: icon_rgba,
            width: icon_width,
            height: icon_height,
        }))
    } else {
        None
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder {
            icon: icon_data,
            ..Default::default()
        }
        .with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    // Pass a cloned paint_rx into the app so we can handle repaints
    let app_paint_rx = Arc::clone(&server_gui_rx_ref);

    eframe::run_native(
        "Alembic",
        options,
        Box::new(|cc| {
            let frame = cc.egui_ctx.clone();

            thread::spawn(move || {
                loop {
                    match app_paint_rx.try_lock().unwrap().try_recv() {
                        Ok(msg) => match msg {
                            ServerGuiMessage::RequestRepaint => {
                                frame.request_repaint();
                            }
                        },
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => {
                            eprintln!("Channel disconnected");
                            break;
                        }
                    }

                    // ? 60FPS
                    thread::sleep(Duration::from_millis(16));
                }
            });

            egui_extras::install_image_loaders(&cc.egui_ctx);

            let app: Application = Application::new(
                cc,
                Arc::clone(&client_server_rx_ref),
                background_fetch_sender,
                background_update_receiver,
            );

            Ok(Box::new(app))
        }),
    )
}
