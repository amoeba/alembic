mod windows;
mod wine;

pub use windows::WindowsClientConfig;
pub use wine::WineClientConfig;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientConfig {
    Windows(WindowsClientConfig),
    Wine(WineClientConfig),
}

impl ClientConfig {
    pub fn display_name(&self) -> &str {
        match self {
            ClientConfig::Windows(c) => &c.display_name,
            ClientConfig::Wine(c) => &c.display_name,
        }
    }

    pub fn install_path(&self) -> &Path {
        match self {
            ClientConfig::Windows(c) => &c.install_path,
            ClientConfig::Wine(c) => &c.install_path,
        }
    }

    pub fn is_wine(&self) -> bool {
        matches!(self, ClientConfig::Wine(_))
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, ClientConfig::Windows(_))
    }
}

impl fmt::Display for ClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientConfig::Windows(config) => write!(f, "{}", config),
            ClientConfig::Wine(config) => write!(f, "{}", config),
        }
    }
}
