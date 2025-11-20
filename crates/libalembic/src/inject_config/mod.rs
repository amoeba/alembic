mod dll_type;

pub use dll_type::DllType;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Configuration for DLL injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectConfig {
    /// Path to the DLL (e.g., C:\Program Files\Alembic\Alembic.dll)
    pub dll_path: PathBuf,
    /// Type of DLL (Alembic or Decal)
    pub dll_type: DllType,
    /// Optional startup function to call after injection (e.g., "DecalStartup")
    pub startup_function: Option<String>,
    /// Wine prefix path (needed for Wine to convert Windows path to Unix path)
    /// None on native Windows, Some on Wine/macOS/Linux
    pub wine_prefix: Option<PathBuf>,
}

impl InjectConfig {
    /// Get the actual filesystem path to the DLL
    /// For Windows, this is the dll_path
    /// For Wine, this converts the Windows path to the Unix path using the prefix
    pub fn filesystem_path(&self) -> PathBuf {
        if let Some(wine_prefix) = &self.wine_prefix {
            // Wine: Convert C:\path\to\file to /prefix/drive_c/path/to/file
            let dll_str = self.dll_path.display().to_string();
            if let Some(relative) = dll_str
                .strip_prefix("C:\\")
                .or_else(|| dll_str.strip_prefix("C:/"))
            {
                let unix_relative = relative.replace("\\", "/");
                wine_prefix.join("drive_c").join(unix_relative)
            } else {
                self.dll_path.clone()
            }
        } else {
            // Windows: Use dll_path directly
            self.dll_path.clone()
        }
    }
}

impl fmt::Display for InjectConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Type: {}", if self.wine_prefix.is_some() { "Wine" } else { "Windows" })?;
        writeln!(f, "DLL Type: {}", self.dll_type)?;
        writeln!(f, "DLL Path: {}", self.dll_path.display())?;
        if let Some(prefix) = &self.wine_prefix {
            write!(f, "Wine Prefix: {}", prefix.display())?;
        }
        Ok(())
    }
}
