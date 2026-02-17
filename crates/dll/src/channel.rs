use std::sync::{Arc, Once};

use libalembic::msg::client_server::ClientServerMessage;
use tokio::sync::{mpsc, Mutex};

static mut DLL_TX: Option<Arc<Mutex<mpsc::UnboundedSender<ClientServerMessage>>>> = None;
static mut DLL_RX: Option<Arc<Mutex<mpsc::UnboundedReceiver<ClientServerMessage>>>> = None;
static CHANNEL_INIT: Once = Once::new();

#[allow(static_mut_refs)]
pub fn ensure_channel() -> (
    &'static Arc<Mutex<mpsc::UnboundedSender<ClientServerMessage>>>,
    &'static Arc<Mutex<mpsc::UnboundedReceiver<ClientServerMessage>>>,
) {
    unsafe {
        CHANNEL_INIT.call_once(|| {
            let (tx, rx) = mpsc::unbounded_channel();

            DLL_TX = Some(Arc::new(Mutex::new(tx)));
            DLL_RX = Some(Arc::new(Mutex::new(rx)));
        });

        (DLL_TX.as_ref().unwrap(), DLL_RX.as_ref().unwrap())
    }
}
