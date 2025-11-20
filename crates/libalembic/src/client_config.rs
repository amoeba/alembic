use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Complete configuration for a client installation
/// Combines location info with how to launch it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientConfig {
    Windows(WindowsClientConfig),
    Wine(WineClientConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsClientConfig {
    /// Display name (e.g., "Asheron's Call - Main Installation")
    pub display_name: String,
    /// Path to AC installation (C:\Turbine\Asheron's Call)
    pub install_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineClientConfig {
    /// Display name (e.g., "Whisky Bottle: AC", "Wine: ~/.wine")
    pub display_name: String,
    /// AC installation in Windows format (C:\Turbine\Asheron's Call)
    pub install_path: PathBuf,
    /// Wine executable path (wine64, wine, etc.)
    pub wine_executable: PathBuf,
    /// Wine prefix directory (WINEPREFIX)
    pub prefix_path: PathBuf,
    /// Additional environment variables
    pub additional_env: HashMap<String, String>,
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

impl fmt::Display for WindowsClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.display_name)?;
        writeln!(f, "Install path: {}", self.install_path.display())?;
        write!(f, "Type: Windows")
    }
}

impl fmt::Display for WineClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.display_name)?;
        writeln!(f, "Install path: {}", self.install_path.display())?;
        writeln!(f, "Type: Wine")?;
        writeln!(f, "Wine executable: {}", self.wine_executable.display())?;
        writeln!(f, "Wine prefix: {}", self.prefix_path.display())?;

        if !self.additional_env.is_empty() {
            writeln!(f)?;
            writeln!(f, "Environment variables:")?;
            for (key, value) in &self.additional_env {
                writeln!(f, "{}={}", key, value)?;
            }
        }

        Ok(())
    }
}
