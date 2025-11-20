use super::traits::ClientConfiguration;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsClientConfig {
    pub name: String,
    pub client_path: PathBuf,
}

impl ClientConfiguration for WindowsClientConfig {
    fn display_name(&self) -> &str {
        &self.name
    }

    fn install_path(&self) -> &Path {
        self.client_path.parent().unwrap_or_else(|| Path::new(""))
    }
}

impl fmt::Display for WindowsClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Client path: {}", self.client_path.display())?;
        write!(f, "Type: Windows")
    }
}
