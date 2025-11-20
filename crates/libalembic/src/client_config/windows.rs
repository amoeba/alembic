use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsClientConfig {
    /// Display name (e.g., "Asheron's Call - Main Installation")
    pub display_name: String,
    /// Path to AC installation (C:\Turbine\Asheron's Call)
    pub install_path: PathBuf,
}

impl fmt::Display for WindowsClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.display_name)?;
        writeln!(f, "Install path: {}", self.install_path.display())?;
        write!(f, "Type: Windows")
    }
}
