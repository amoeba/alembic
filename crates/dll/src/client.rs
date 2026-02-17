use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use libalembic::msg::client_server::ClientServerMessage;
use libalembic::rpc::WorldClient;
use tarpc::{client as tarpc_client, context, tokio_serde::formats::Json};
use tokio::sync::mpsc::error::TryRecvError;

use crate::channel::ensure_channel;
use crate::logging::log_message;
use crate::runtime::ensure_runtime;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub fn ensure_client() -> anyhow::Result<()> {
    let (_tx, rx) = ensure_channel();
    let runtime = ensure_runtime();

    SHUTDOWN.store(false, Ordering::SeqCst);

    runtime.spawn(async move {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
        let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
        let client: WorldClient = WorldClient::new(
            tarpc_client::Config::default(),
            transport.await.expect("Failed to connect to server"),
        )
        .spawn();

        loop {
            if SHUTDOWN.load(Ordering::SeqCst) {
                break;
            }

            match rx.try_lock().unwrap().try_recv() {
                Ok(msg) => match msg {
                    ClientServerMessage::HandleSendTo(vec) => {
                        if let Err(e) = client.handle_sendto(context::current(), vec).await {
                            unsafe { log_message(&format!("HandleSendTo error: {}", e)) };
                        }
                    }
                    ClientServerMessage::HandleRecvFrom(vec) => {
                        if let Err(e) = client.handle_recvfrom(context::current(), vec).await {
                            unsafe { log_message(&format!("HandleRecvFrom error: {}", e)) };
                        }
                    }
                    ClientServerMessage::HandleAddTextToScroll(text) => {
                        if let Err(e) = client.handle_chat(context::current(), text).await {
                            unsafe { log_message(&format!("HandleChat error: {}", e)) };
                        }
                    }
                    ClientServerMessage::AppendLog(_) => todo!(),
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(16));
        }

        unsafe { log_message("Client loop shutting down") };
        drop(client);
    });

    Ok(())
}

pub fn shutdown_client() -> anyhow::Result<()> {
    SHUTDOWN.store(true, Ordering::SeqCst);
    Ok(())
}
