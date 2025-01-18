#[allow(unused)]
pub struct LogEntry {
    pub timestamp: u64,
    pub message: String,
}

#[allow(unused)]
pub struct PacketInfo {
    pub index: usize,
    pub timestamp: u64,
    pub data: Vec<u8>,
}
pub struct Account {
    pub name: String,
}

pub struct Backend {
    pub logs: Vec<LogEntry>,
    pub packets_incoming: Vec<PacketInfo>,
    pub packets_outgoing: Vec<PacketInfo>,
    pub accounts: Vec<Account>,
    pub selected_account: Option<usize>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            logs: vec![],
            packets_incoming: vec![],
            packets_outgoing: vec![],
            accounts: vec![
                Account {
                    name: "Frostfell - F".to_string(),
                },
                Account {
                    name: "Leafcull - L".to_string(),
                },
                Account {
                    name: "WintersEbb - W".to_string(),
                },
            ],
            selected_account: None,
        }
    }
}
