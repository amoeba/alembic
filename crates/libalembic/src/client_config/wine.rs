use super::traits::ClientConfiguration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineClientConfig {
    pub name: String,
    pub client_path: PathBuf,
    pub wine_executable_path: PathBuf,
    pub wine_env: HashMap<String, String>,
}

impl ClientConfiguration for WineClientConfig {
    fn display_name(&self) -> &str {
        &self.name
    }

    fn install_path(&self) -> &Path {
        // Return the parent directory of acclient.exe as the install path
        self.client_path.parent().unwrap_or_else(|| Path::new(""))
    }
}

impl fmt::Display for WineClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Client path: {}", self.client_path.display())?;
        writeln!(f, "Type: Wine")?;
        writeln!(f, "Wine executable: {}", self.wine_executable_path.display())?;

        if !self.wine_env.is_empty() {
            writeln!(f)?;
            writeln!(f, "Wine environment variables:")?;
            for (key, value) in &self.wine_env {
                writeln!(f, "  {}={}", key, value)?;
            }
        }

        Ok(())
    }
}
