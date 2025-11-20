use super::traits::ClientConfiguration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

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

impl ClientConfiguration for WineClientConfig {
    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn install_path(&self) -> &Path {
        &self.install_path
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
