use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::inject_config::InjectConfig;
use crate::validation::ValidationResult;

/// Get the parent directory of a path, handling Windows-style paths on Unix.
/// On non-Windows systems, std::path::Path doesn't understand backslashes as
/// separators, so we need to handle them manually.
pub fn windows_path_parent(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();

    // If the path contains backslashes, treat it as a Windows path
    if path_str.contains('\\') {
        if let Some(idx) = path_str.rfind('\\') {
            return PathBuf::from(&path_str[..idx]);
        }
    }

    // Fall back to standard parent() for Unix paths
    path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::new())
}

pub trait ClientConfig {
    fn name(&self) -> &str;
    fn client_path(&self) -> &Path;
    fn wrapper_program(&self) -> Option<&Path>;
    fn env(&self) -> &HashMap<String, String>;

    /// Validate that all paths in this config and the optional inject config exist.
    fn validate(&self, inject_config: Option<&InjectConfig>) -> ValidationResult;

    fn install_path(&self) -> PathBuf {
        windows_path_parent(self.client_path())
    }

    fn fmt_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name())?;
        writeln!(f, "Client path: {}", self.client_path().display())?;

        if let Some(wrapper) = self.wrapper_program() {
            writeln!(f, "Type: Wine")?;
            writeln!(f, "Wrapper program: {}", wrapper.display())?;
        } else {
            writeln!(f, "Type: Windows")?;
        }

        if !self.env().is_empty() {
            writeln!(f)?;
            writeln!(f, "Environment variables:")?;
            for (key, value) in self.env() {
                writeln!(f, "  {}={}", key, value)?;
            }
        }

        Ok(())
    }
}
