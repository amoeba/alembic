use std::sync::{Arc, Once};

use libalembic::msg::client_server::ClientServerMessage;

use tokio::sync::{mpsc, Mutex};

static mut dll_tx: Option<Arc<Mutex<mpsc::UnboundedSender<ClientServerMessage>>>> = None;
static mut dll_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<ClientServerMessage>>>> = None;
static channel_init: Once = Once::new();

#[allow(static_mut_refs)]
pub fn ensure_channel() -> (
    &'static Arc<Mutex<tokio::sync::mpsc::UnboundedSender<ClientServerMessage>>>,
    &'static Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<ClientServerMessage>>>,
) {
    unsafe {
        channel_init.call_once(|| {
            let (tx, rx): (
                mpsc::UnboundedSender<ClientServerMessage>,
                mpsc::UnboundedReceiver<ClientServerMessage>,
            ) = mpsc::unbounded_channel();

            dll_tx = Some(Arc::new(Mutex::new(tx)));
            dll_rx = Some(Arc::new(Mutex::new(rx)));
        });

        (dll_tx.as_ref().unwrap(), dll_rx.as_ref().unwrap())
    }
}
