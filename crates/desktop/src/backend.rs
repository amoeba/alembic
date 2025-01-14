pub struct LogEntry {
    pub timestamp: u64,
    pub message: String,
}

pub struct PacketInfo {
    pub index: usize,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

pub struct Backend {
    pub logs: Vec<LogEntry>,
    pub packets_incoming: Vec<PacketInfo>,
    pub packets_outgoing: Vec<PacketInfo>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            logs: vec![],
            packets_incoming: vec![],
            packets_outgoing: vec![],
        }
    }
}
