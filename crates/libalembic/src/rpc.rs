use std::{future::Future, sync::Arc};

use tarpc::context;
use tokio::sync::{mpsc::Sender, Mutex};

use crate::msg::{client_server::ClientServerMessage, server_gui::ServerGuiMessage};

#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
    async fn update_string(value: String) -> String;
    async fn append_log(value: String) -> String;
    async fn handle_sendto(value: Vec<u8>) -> usize;
    async fn handle_recvfrom(value: Vec<u8>) -> usize;
    async fn handle_chat(value: String);
}

#[derive(Clone)]
pub struct HelloServer {
    pub paint_tx: Arc<Mutex<Sender<ServerGuiMessage>>>,
    pub gui_tx: Arc<Mutex<Sender<ClientServerMessage>>>,
}

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        println!("rpc hello");
        format!("Hello, {name}!")
    }

    async fn update_string(self, _context: ::tarpc::context::Context, value: String) -> String {
        println!("rpc update_string");

        match self
            .gui_tx
            .lock()
            .await
            .send(ClientServerMessage::UpdateString(value.to_string()))
            .await
        {
            Ok(()) => println!("Request to update string with string {value} sent to GUI."),
            Err(error) => println!("tx error: {error}"),
        }

        match self
            .paint_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => println!("Repaint Requested"),
            Err(error) => println!("tx error: {error}"),
        }

        value
    }

    async fn append_log(self, _context: ::tarpc::context::Context, value: String) -> String {
        println!("rpc append_log");

        match self
            .gui_tx
            .lock()
            .await
            .send(ClientServerMessage::AppendLog(value.to_string()))
            .await
        {
            Ok(()) => println!("Request to append logs with string {value} sent to GUI."),
            Err(error) => println!("tx error: {error}"),
        }

        match self
            .paint_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => println!("Repaint Requested"),
            Err(error) => println!("tx error: {error}"),
        }

        value
    }

    async fn handle_sendto(self, context: tarpc::context::Context, value: Vec<u8>) -> usize {
        let _ = context;
        println!("rpc handle_sendto");
        let len = value.len();

        match self
            .gui_tx
            .lock()
            .await
            .send(ClientServerMessage::SendTo(value))
            .await
        {
            Ok(()) => println!("sendto sent"),
            Err(error) => println!("tx error: {error}"),
        }

        match self
            .paint_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => println!("Repaint Requested"),
            Err(error) => println!("tx error: {error}"),
        }

        len
    }

    async fn handle_recvfrom(self, context: tarpc::context::Context, value: Vec<u8>) -> usize {
        let _ = context;
        println!("rpc handle_recvfrom");
        let len = value.len();

        match self
            .gui_tx
            .lock()
            .await
            .send(ClientServerMessage::RecvFrom(value))
            .await
        {
            Ok(()) => println!("RecvFrom sent"),
            Err(error) => println!("tx error: {error}"),
        }

        match self
            .paint_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => println!("Repaint Requested"),
            Err(error) => println!("tx error: {error}"),
        }

        len
    }

    async fn handle_chat(self, context: ::tarpc::context::Context, value: String) {
        let _ = context;
        println!("rpc handle_chat");

        match self
            .gui_tx
            .lock()
            .await
            .send(ClientServerMessage::AddTextToScroll(value))
            .await
        {
            Ok(()) => println!("AddTextToScroll sent"),
            Err(error) => println!("tx error: {error}"),
        }

        match self
            .paint_tx
            .lock()
            .await
            .send(ServerGuiMessage::RequestRepaint)
            .await
        {
            Ok(()) => println!("Repaint Requested"),
            Err(error) => println!("tx error: {error}"),
        }
    }
}

// This is from tarpc's source and makes the server loop code read a bit better
pub async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
