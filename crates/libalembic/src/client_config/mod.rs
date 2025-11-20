use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientType {
    Windows,
    Wine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub name: String,
    pub client_type: ClientType,
    pub client_path: PathBuf,
    pub wrapper_program: Option<PathBuf>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl ClientConfig {
    pub fn is_wine(&self) -> bool {
        self.client_type == ClientType::Wine
    }

    pub fn is_windows(&self) -> bool {
        self.client_type == ClientType::Windows
    }

    pub fn install_path(&self) -> &Path {
        self.client_path.parent().unwrap_or_else(|| Path::new(""))
    }
}

impl fmt::Display for ClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Client path: {}", self.client_path.display())?;
        writeln!(f, "Type: {:?}", self.client_type)?;

        if let Some(wrapper) = &self.wrapper_program {
            writeln!(f, "Wrapper program: {}", wrapper.display())?;
        }

        if !self.env.is_empty() {
            writeln!(f)?;
            writeln!(f, "Environment variables:")?;
            for (key, value) in &self.env {
                writeln!(f, "  {}={}", key, value)?;
            }
        }

        Ok(())
    }
}
