use super::traits::ClientConfig;
use crate::inject_config::InjectConfig;
use crate::validation::{is_windows_path, validate_native_path, validate_wine_path, ValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineClientConfig {
    pub name: String,
    pub client_path: PathBuf,
    pub wrapper_program: Option<PathBuf>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl ClientConfig for WineClientConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn client_path(&self) -> &Path {
        &self.client_path
    }

    fn wrapper_program(&self) -> Option<&Path> {
        self.wrapper_program.as_deref()
    }

    fn env(&self) -> &HashMap<String, String> {
        &self.env
    }

    fn validate(&self, inject_config: Option<&InjectConfig>) -> ValidationResult {
        let wine_exe = self
            .wrapper_program
            .as_ref()
            .map(|p| p.as_path())
            .unwrap_or(Path::new("wine"));

        let mut result = ValidationResult::ok();

        // Validate client path
        if is_windows_path(&self.client_path) {
            result.merge(validate_wine_path(
                wine_exe,
                &self.client_path,
                &self.env,
                "Client executable",
            ));
        } else {
            result.merge(validate_native_path(&self.client_path, "Client executable"));
        }

        // Validate DLL if present
        if let Some(dll) = inject_config {
            if is_windows_path(&dll.dll_path) {
                result.merge(validate_wine_path(wine_exe, &dll.dll_path, &self.env, "DLL"));
            } else {
                result.merge(validate_native_path(&dll.dll_path, "DLL"));
            }
        }

        result
    }
}

impl std::fmt::Display for WineClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ClientConfig::fmt_display(self, f)
    }
}
