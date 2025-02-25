// Keep track of injection state and resources
pub struct DllState {
    runtime: Runtime,
    shutdown_requested: Arc<AtomicBool>,
    tx: Arc<Mutex<mpsc::UnboundedSender<ClientServerMessage>>>,
    rx: Arc<Mutex<mpsc::UnboundedReceiver<ClientServerMessage>>>,
}

impl DllState {
    pub fn new() -> anyhow::Result<Self> {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime.");
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::unbounded_channel();
        
        Ok(Self {
            runtime,
            shutdown_requested,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        })
    }

    pub fn shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);

        // Not sure if truly need to sleep here, but just to be safe
        thread::sleep(Duration::from_millis(100));
        self.runtime.shutdown_timeout(Duration::from_secs(1));
    }
}