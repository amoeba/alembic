use std::{future::Future, sync::Arc};

use tarpc::context;
use tokio::sync::{mpsc::Sender, Mutex};

#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
    async fn update_string(value: String) -> String;
    async fn append_log(value: String) -> String;
}

#[derive(Clone)]
pub struct HelloServer {
    pub paint_tx: Arc<Mutex<Sender<PaintMessage>>>,
    pub gui_tx: Arc<Mutex<Sender<GuiMessage>>>,
}

pub enum GuiMessage {
    Hello(String),
    UpdateString(String),
    AppendLog(String),
}

pub enum PaintMessage {
    RequestRepaint,
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
            .send(GuiMessage::UpdateString(value.to_string()))
            .await
        {
            Ok(()) => println!("Request to update string with string {value} sent to GUI."),
            Err(error) => println!("tx error: {error}"),
        }

        match self
            .paint_tx
            .lock()
            .await
            .send(PaintMessage::RequestRepaint)
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
            .send(GuiMessage::AppendLog(value.to_string()))
            .await
        {
            Ok(()) => println!("Request to append logs with string {value} sent to GUI."),
            Err(error) => println!("tx error: {error}"),
        }

        match self
            .paint_tx
            .lock()
            .await
            .send(PaintMessage::RequestRepaint)
            .await
        {
            Ok(()) => println!("Repaint Requested"),
            Err(error) => println!("tx error: {error}"),
        }

        value
    }
}

// This is from tarpc's source and makes the server loop code read a bit better
pub async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
