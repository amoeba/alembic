use super::traits::ClientConfiguration;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsClientConfig {
    /// Display name (e.g., "Asheron's Call - Main Installation")
    pub display_name: String,
    /// Path to AC installation (C:\Turbine\Asheron's Call)
    pub install_path: PathBuf,
}

impl ClientConfiguration for WindowsClientConfig {
    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn install_path(&self) -> &Path {
        &self.install_path
    }
}

impl fmt::Display for WindowsClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.display_name)?;
        writeln!(f, "Install path: {}", self.install_path.display())?;
        write!(f, "Type: Windows")
    }
}
