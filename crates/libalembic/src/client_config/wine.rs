use super::traits::{ClientConfig, LaunchCommand};
use crate::inject_config::InjectConfig;
use crate::validation::{
    is_windows_path, validate_native_path, validate_wine_path, ValidationResult,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineClientConfig {
    pub name: String,
    pub client_path: PathBuf,
    /// The launch command specifying program, args, and env vars
    pub launch_command: LaunchCommand,
}

impl ClientConfig for WineClientConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn client_path(&self) -> &Path {
        &self.client_path
    }

    fn launch_command(&self) -> Option<&LaunchCommand> {
        Some(&self.launch_command)
    }

    fn validate(&self, inject_config: Option<&InjectConfig>) -> ValidationResult {
        // For validation, we need to find the wine executable
        // It could be the program itself (e.g., /usr/bin/wine) or
        // passed via --command= arg (e.g., flatpak run --command=wine)
        let wine_exe = self.get_wine_executable();

        let mut result = ValidationResult::ok();

        // Validate client path
        if is_windows_path(&self.client_path) {
            result.merge(validate_wine_path(
                &wine_exe,
                &self.client_path,
                &self.launch_command.env,
                "Client executable",
            ));
        } else {
            result.merge(validate_native_path(&self.client_path, "Client executable"));
        }

        // Validate DLL if present
        if let Some(dll) = inject_config {
            if is_windows_path(&dll.dll_path) {
                result.merge(validate_wine_path(
                    &wine_exe,
                    &dll.dll_path,
                    &self.launch_command.env,
                    "DLL",
                ));
            } else {
                result.merge(validate_native_path(&dll.dll_path, "DLL"));
            }
        }

        result
    }
}

impl WineClientConfig {
    /// Get the wine executable path for validation purposes.
    /// Handles both direct wine paths and flatpak --command=wine patterns.
    fn get_wine_executable(&self) -> PathBuf {
        // Check if any arg contains --command=wine (flatpak pattern)
        for arg in &self.launch_command.args {
            if arg.starts_with("--command=") {
                let cmd = arg.strip_prefix("--command=").unwrap();
                return PathBuf::from(cmd);
            }
        }
        // Otherwise, the program itself is wine
        self.launch_command.program.clone()
    }
}

impl std::fmt::Display for WineClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ClientConfig::fmt_display(self, f)
    }
}
