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
    pub packet_count_incoming: usize,
    pub packet_count_outgoing: usize,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            logs: vec![],
            packets_incoming: vec![],
            packets_outgoing: vec![],
            packet_count_incoming: 0,
            packet_count_outgoing: 0,
        }
    }
}
