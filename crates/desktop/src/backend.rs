use std::num::NonZero;

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

#[derive(Clone)]
pub struct Client {
    pub pid: NonZero<u32>,
}

pub struct Backend {
    pub status_message: Option<String>,
    pub client: Option<Client>,
    pub is_injected: bool,
    pub logs: Vec<LogEntry>,
    pub packets_incoming: Vec<PacketInfo>,
    pub packets_outgoing: Vec<PacketInfo>,
    pub chat_messages: Vec<ChatMessage>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            status_message: None,
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
            chat_messages: vec![],
        }
    }
}
