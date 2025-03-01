use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Once;
use std::thread;
use std::time::Duration;

use libalembic::msg::client_server::ClientServerMessage;
use libalembic::rpc::WorldClient;

use tarpc::tokio_util::sync::CancellationToken;
use tarpc::{client as tarcp_client, context, tokio_serde::formats::Json};
use tokio::sync::mpsc::error::TryRecvError;

use crate::channel::ensure_channel;
use crate::runtime::ensure_runtime;

static mut SHUTDOWN_TOKEN: Option<CancellationToken> = None;
static token_init: Once = Once::new();

pub fn ensure_client() -> anyhow::Result<()> {
    let (_tx, rx) = ensure_channel();
    let runtime = ensure_runtime();
    let token = ensure_shutdown_token().clone();

    runtime.spawn(async move {
        let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
        let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
        let client: WorldClient = WorldClient::new(
            tarcp_client::Config::default(),
            transport.await.expect("oops"),
        )
        .spawn();

        loop {
            // Check if shutdown was requested
            if token.is_cancelled() {
                break;
            }

            match rx.try_lock().unwrap().try_recv() {
                Ok(msg) => match msg {
                    ClientServerMessage::HandleSendTo(vec) => {
                        match client.handle_sendto(context::current(), vec).await {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                    }
                    ClientServerMessage::HandleRecvFrom(vec) => {
                        match client.handle_recvfrom(context::current(), vec).await {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                    }
                    ClientServerMessage::HandleAddTextToScroll(text) => {
                        match client.handle_chat(context::current(), text).await {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                    }
                    ClientServerMessage::AppendLog(_) => todo!(),
                    ClientServerMessage::ClientInjected() => todo!(),
                    ClientServerMessage::ClientEjected() => todo!(),
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(16));
        }

        // Cleanup connection
        drop(client);
    });

    Ok(())
}

pub fn shutdown_client() -> anyhow::Result<()> {
    if let Some(token) = unsafe { SHUTDOWN_TOKEN.as_ref() } {
        token.cancel();
    }

    Ok(())
}

pub fn ensure_shutdown_token() -> &'static CancellationToken {
    unsafe {
        token_init.call_once(|| {
            SHUTDOWN_TOKEN = Some(CancellationToken::new());
        });
        SHUTDOWN_TOKEN.as_ref().unwrap()
    }
}
