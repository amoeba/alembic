use std::{future::Future, sync::Arc};

use tokio::sync::{mpsc::Sender, Mutex};

use crate::msg::{client_server::ClientServerMessage, server_gui::ServerGuiMessage};

#[tarpc::service]
pub trait World {
    async fn append_log(value: String) -> String;
    async fn handle_sendto(value: Vec<u8>) -> usize;
    async fn handle_recvfrom(value: Vec<u8>) -> usize;
    async fn handle_chat(value: String);
}

#[derive(Clone)]
pub struct HelloServer {
    pub server_gui_tx: Arc<Mutex<Sender<ServerGuiMessage>>>,
    pub client_server_tx: Arc<Mutex<Sender<ClientServerMessage>>>,
}

impl World for HelloServer {
    async fn append_log(self, _context: ::tarpc::context::Context, value: String) -> String {
        match self
            .client_server_tx
            .lock()
            .await
            .send(ClientServerMessage::AppendLog(value.to_string()))
            .await
        {
            Ok(()) => {}
            Err(error) => eprintln!("tx error: {error}"),
        }

        match self
            .server_gui_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => {}
            Err(error) => eprintln!("tx error: {error}"),
        }

        value
    }

    async fn handle_sendto(self, context: tarpc::context::Context, value: Vec<u8>) -> usize {
        let _ = context;
        let len = value.len();

        match self
            .client_server_tx
            .lock()
            .await
            .send(ClientServerMessage::HandleSendTo(value))
            .await
        {
            Ok(()) => {}
            Err(error) => eprintln!("tx error: {error}"),
        }

        match self
            .server_gui_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => {}
            Err(error) => eprintln!("tx error: {error}"),
        }

        len
    }

    async fn handle_recvfrom(self, context: tarpc::context::Context, value: Vec<u8>) -> usize {
        let _ = context;
        let len = value.len();

        match self
            .client_server_tx
            .lock()
            .await
            .send(ClientServerMessage::HandleRecvFrom(value))
            .await
        {
            Ok(()) => {}
            Err(error) => eprintln!("tx error: {error}"),
        }

        match self
            .server_gui_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => {}
            Err(error) => eprintln!("tx error: {error}"),
        }

        len
    }

    async fn handle_chat(self, context: ::tarpc::context::Context, value: String) {
        let _ = context;

        match self
            .client_server_tx
            .lock()
            .await
            .send(ClientServerMessage::HandleAddTextToScroll(value))
            .await
        {
            Ok(()) => {}
            Err(error) => eprintln!("tx error: {error}"),
        }

        match self
            .server_gui_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => {}
            Err(error) => eprintln!("tx error: {error}"),
        }
    }
}

// This is from tarpc's source and makes the server loop code read a bit better
pub async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
