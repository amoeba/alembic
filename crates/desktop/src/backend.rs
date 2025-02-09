use std::{fmt::Display, num::NonZero};

use ringbuffer::{AllocRingBuffer, RingBuffer};

#[allow(unused)]
pub struct LogEntry {
    pub timestamp: u64,
    pub message: String,
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.timestamp, self.message)
    }
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
    pub logs: AllocRingBuffer<LogEntry>,
    pub packets_incoming: AllocRingBuffer<PacketInfo>,
    pub packets_outgoing: AllocRingBuffer<PacketInfo>,
    pub chat_messages: AllocRingBuffer<ChatMessage>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            status_message: None,
            client: None,
            is_injected: false,
            logs: AllocRingBuffer::<LogEntry>::new(1000),
            packets_incoming: AllocRingBuffer::<PacketInfo>::new(100),
            packets_outgoing: AllocRingBuffer::<PacketInfo>::new(100),
            chat_messages: AllocRingBuffer::<ChatMessage>::new(100),
        }
    }
}
