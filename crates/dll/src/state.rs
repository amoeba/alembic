use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use libalembic::msg::client_server::ClientServerMessage;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, Mutex},
};

pub struct DllState {
    init_result: Arc<AtomicBool>,
    init_error: Option<String>,
    shutdown_requested: Arc<AtomicBool>,
    is_initialized: Arc<AtomicBool>,

    runtime: Runtime,

    tx: Arc<Mutex<mpsc::UnboundedSender<ClientServerMessage>>>,
    rx: Arc<Mutex<mpsc::UnboundedReceiver<ClientServerMessage>>>,
}

impl DllState {
    pub fn new() -> anyhow::Result<Self> {
        let init_result = Arc::new(AtomicBool::new(false));
        let init_error: Option<String> = None;
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let is_initialized = Arc::new(AtomicBool::new(false));

        let runtime = Runtime::new().expect("Failed to create Tokio runtime.");
        let (tx, rx) = mpsc::unbounded_channel();

        Ok(Self {
            init_result,
            init_error,
            shutdown_requested,
            runtime,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            is_initialized,
        })
    }

    pub fn startup(&self) -> anyhow::Result<()> {
        if self.is_initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Initialize runtime resources here
        self.runtime.block_on(async {
            // Initialize any async resources, RPC servers, etc.
            Ok(())
        })?;

        self.is_initialized.store(true, Ordering::SeqCst);
        self.init_result.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn shutdown(&self) {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return;
        }

        self.shutdown_requested.store(true, Ordering::SeqCst);

        // Cleanup any resources in the runtime
        self.runtime.block_on(async {
            // Shutdown RPC servers, close connections, etc.
        });

        thread::sleep(Duration::from_millis(100));
        self.runtime.shutdown_timeout(Duration::from_secs(1));

        self.is_initialized.store(false, Ordering::SeqCst);
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized.load(Ordering::SeqCst)
    }
}
