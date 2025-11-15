use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};

/// Configuration for DLL injection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InjectConfig {
    Windows(WindowsInjectConfig),
    Wine(WineInjectConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsInjectConfig {
    /// Path to the DLL (e.g., C:\Program Files\Alembic\Alembic.dll)
    pub dll_path: PathBuf,
    /// Type of DLL (Alembic or Decal)
    pub dll_type: DllType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineInjectConfig {
    /// Type of DLL (Alembic or Decal)
    pub dll_type: DllType,
    /// Wine prefix path (needed to convert Windows path to Unix path)
    pub wine_prefix: PathBuf,
    /// DLL path in Windows format (e.g., C:\Program Files (x86)\Decal 3.0\Inject.dll)
    pub dll_path: PathBuf,
}

impl InjectConfig {
    pub fn dll_type(&self) -> DllType {
        match self {
            InjectConfig::Windows(config) => config.dll_type,
            InjectConfig::Wine(config) => config.dll_type,
        }
    }

    pub fn dll_path(&self) -> &PathBuf {
        match self {
            InjectConfig::Windows(config) => &config.dll_path,
            InjectConfig::Wine(config) => &config.dll_path,
        }
    }

    /// Get the actual filesystem path to the DLL
    /// For Windows, this is the dll_path
    /// For Wine, this converts the Windows path to the Unix path using the prefix
    pub fn filesystem_path(&self) -> PathBuf {
        match self {
            InjectConfig::Windows(config) => config.dll_path.clone(),
            InjectConfig::Wine(config) => {
                // Convert C:\path\to\file to /prefix/drive_c/path/to/file
                let dll_str = config.dll_path.display().to_string();
                if let Some(relative) = dll_str.strip_prefix("C:\\").or_else(|| dll_str.strip_prefix("C:/")) {
                    let unix_relative = relative.replace("\\", "/");
                    config.wine_prefix.join("drive_c").join(unix_relative)
                } else {
                    config.dll_path.clone()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DllType {
    Alembic,
    Decal,
}

impl fmt::Display for DllType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DllType::Alembic => write!(f, "Alembic"),
            DllType::Decal => write!(f, "Decal"),
        }
    }
}

impl fmt::Display for InjectConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InjectConfig::Windows(config) => {
                writeln!(f, "Type: Windows")?;
                writeln!(f, "DLL Type: {}", config.dll_type)?;
                write!(f, "DLL Path: {}", config.dll_path.display())
            }
            InjectConfig::Wine(config) => {
                writeln!(f, "Type: Wine")?;
                writeln!(f, "DLL Type: {}", config.dll_type)?;
                writeln!(f, "DLL Path: {}", config.dll_path.display())?;
                write!(f, "Wine Prefix: {}", config.wine_prefix.display())
            }
        }
    }
}

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
