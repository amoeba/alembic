#[allow(unused)]
pub struct LogEntry {
    pub timestamp: u64,
    pub message: String,
}

#[allow(unused)]
pub struct ChatMessage {
    pub index: usize,
    pub timestamp: u64,
    pub text: String,
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
#[derive(Clone)]
pub struct Client {
    pub pid: usize,
}

pub struct Backend {
    pub client: Option<Client>,
    pub is_injected: bool,
    pub logs: Vec<LogEntry>,
    pub packets_incoming: Vec<PacketInfo>,
    pub packets_outgoing: Vec<PacketInfo>,
    pub accounts: Vec<Account>,
    pub selected_account: Option<usize>,
    pub chat_messages: Vec<ChatMessage>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            client: None,
            is_injected: false,
            logs: vec![],
            packets_incoming: vec![
                PacketInfo {
                    index: 0,
                    timestamp: 1234,
                    data: vec![1, 2, 3, 4],
                },
                PacketInfo {
                    index: 0,
                    timestamp: 1234,
                    data: vec![1, 2, 3, 4],
                },
                PacketInfo {
                    index: 0,
                    timestamp: 1234,
                    data: vec![1, 2, 3, 4],
                },
            ],
            packets_outgoing: vec![
                PacketInfo {
                    index: 0,
                    timestamp: 1234,
                    data: vec![1, 2, 3, 4],
                },
                PacketInfo {
                    index: 0,
                    timestamp: 1234,
                    data: vec![1, 2, 3, 4],
                },
                PacketInfo {
                    index: 0,
                    timestamp: 1234,
                    data: vec![1, 2, 3, 4],
                },
            ],
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
            chat_messages: vec![],
        }
    }
}
