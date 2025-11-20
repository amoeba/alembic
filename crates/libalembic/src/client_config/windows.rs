use super::traits::ClientConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsClientConfig {
    pub name: String,
    pub client_path: PathBuf,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl ClientConfig for WindowsClientConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn client_path(&self) -> &Path {
        &self.client_path
    }

    fn wrapper_program(&self) -> Option<&Path> {
        None
    }

    fn env(&self) -> &HashMap<String, String> {
        &self.env
    }
}

impl std::fmt::Display for WindowsClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ClientConfig::fmt_display(self, f)
    }
}
