use std::{
    io,
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

use app::App;

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
    mpsc::channel,
    Mutex,
};

pub mod app;
pub mod tabs;

fn main() -> io::Result<()> {
    // Channel: ClientServer
    let (client_server_tx, client_server_rx) = channel::<ClientServerMessage>(32);
    let client_server_tx_ref = Arc::new(Mutex::new(client_server_tx));
    let client_server_rx_ref = Arc::new(Mutex::new(client_server_rx));

    // Channel: Painting
    let (server_gui_tx, server_gui_rx) = channel::<ServerGuiMessage>(32);
    let server_gui_tx_ref = Arc::new(Mutex::new(server_gui_tx));
    let server_gui_rx_ref = Arc::new(Mutex::new(server_gui_rx));

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

    let mut terminal = ratatui::init();
    let app_result = App::new(Arc::clone(&client_server_rx_ref)).run(&mut terminal);
    ratatui::restore();

    app_result
}
