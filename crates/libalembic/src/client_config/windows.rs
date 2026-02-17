use super::traits::{ClientConfig, LaunchCommand};
use crate::inject_config::InjectConfig;
use crate::validation::{ValidationResult, validate_native_path};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsClientConfig {
    pub name: String,
    pub client_path: PathBuf,
    /// DLL configurations for this client
    #[serde(default)]
    pub dlls: Vec<InjectConfig>,
    /// Index of the currently selected DLL for this client
    #[serde(default)]
    pub selected_dll: Option<usize>,
}

impl ClientConfig for WindowsClientConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn client_path(&self) -> &Path {
        &self.client_path
    }

    fn launch_command(&self) -> Option<&LaunchCommand> {
        // Windows launches directly, no wrapper needed
        None
    }

    fn validate(&self, inject_config: Option<&InjectConfig>) -> ValidationResult {
        let mut result = validate_native_path(&self.client_path, "Client executable");

        if let Some(dll) = inject_config {
            result.merge(validate_native_path(&dll.dll_path, "DLL"));
        }

        result
    }
}

impl std::fmt::Display for WindowsClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ClientConfig::fmt_display(self, f)
    }
}
