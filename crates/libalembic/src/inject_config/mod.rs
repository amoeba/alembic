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
}

impl fmt::Display for InjectConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DLL Type: {}", self.dll_type)?;
        writeln!(f, "DLL Path: {}", self.dll_path.display())?;
        Ok(())
    }
}
