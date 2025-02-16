use std::error::Error;
use std::{fmt::Display, num::NonZero};

use ringbuffer::AllocRingBuffer;
use serde::Deserialize;
use serde_with::serde_as;

use crate::fetching::FetchWrapper;

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
    pub subject: String,
    pub body: String,
    pub author: String,
    pub source_url: String,
    #[serde_as(as = "serde_with::TimestampSeconds")]
    pub created_at: std::time::SystemTime,
}

#[derive(Debug)]
pub struct News {
    pub entries: Vec<NewsEntry>,
}

// Community Servers list structures
#[derive(Debug, Deserialize)]
pub struct CommunityServersServerItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub emu: String,
    pub server_host: String,
    pub server_port: String,
    pub r#type: String,
    pub status: String,
    pub website_url: Option<String>,
    pub discord_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommunityServers {
    #[serde(rename = "ServerItem")]
    pub servers: Vec<CommunityServersServerItem>,
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
    pub news: FetchWrapper<News>,
    pub community_servers: FetchWrapper<CommunityServers>,
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
            news: FetchWrapper::NotStarted,
            community_servers: FetchWrapper::NotStarted,
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
