use std::{fmt::Display, num::NonZero, time::SystemTime};

use ringbuffer::AllocRingBuffer;
use serde::Deserialize;
use serde_with::serde_as;

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

impl PacketInfo {
    fn default() -> PacketInfo {
        Self {
            index: 0,
            timestamp: 0,
            data: vec![],
        }
    }
}

impl Display for PacketInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct NewsEntry {
    pub title: String,
    pub author: String,
    #[serde_as(as = "serde_with::TimestampSeconds")]
    pub created_at: std::time::SystemTime,
    pub body: String,
}

pub struct News {
    pub entries: Vec<NewsEntry>,
}

#[derive(Clone)]
pub struct Client {
    pub pid: NonZero<u32>,
}

#[derive(Clone)]
pub struct NetworkStatistics {
    pub incoming_count: usize,
    pub outgoing_count: usize,
}

#[derive(Clone)]
pub struct Statistics {
    pub network: NetworkStatistics,
}
impl Statistics {
    fn default() -> Self {
        Self {
            network: NetworkStatistics {
                incoming_count: 0,
                outgoing_count: 0,
            },
        }
    }
}
pub struct Backend {
    pub status_message: Option<String>,
    pub news: Option<News>,
    pub client: Option<Client>,
    pub is_injected: bool,
    pub logs: AllocRingBuffer<LogEntry>,
    pub packets_incoming: AllocRingBuffer<PacketInfo>,
    pub packets_outgoing: AllocRingBuffer<PacketInfo>,
    pub chat_messages: AllocRingBuffer<ChatMessage>,
    pub statistics: Statistics,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            status_message: None,
            news: None,
            client: None,
            is_injected: false,
            logs: AllocRingBuffer::<LogEntry>::new(10000),
            packets_incoming: AllocRingBuffer::<PacketInfo>::new(10000),
            packets_outgoing: AllocRingBuffer::<PacketInfo>::new(10000),
            chat_messages: AllocRingBuffer::<ChatMessage>::new(10000),
            statistics: Statistics::default(),
        }
    }
}
