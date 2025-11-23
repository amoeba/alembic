use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::inject_config::InjectConfig;
use crate::validation::ValidationResult;

/// A command specification for launching processes, modeled after std::process::Command.
/// Contains the program to run, ordered arguments, and environment variables.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchCommand {
    /// The program/executable to run (e.g., "flatpak", "/usr/bin/wine")
    pub program: PathBuf,
    /// Ordered arguments to pass before the dynamic arguments (cork, client, etc.)
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables to set on the process
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl LaunchCommand {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

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
    path.parent().map(|p| p.to_path_buf()).unwrap_or_default()
}

pub trait ClientConfig {
    fn name(&self) -> &str;
    fn client_path(&self) -> &Path;
    fn launch_command(&self) -> Option<&LaunchCommand>;

    /// Validate that all paths in this config and the optional inject config exist.
    fn validate(&self, inject_config: Option<&InjectConfig>) -> ValidationResult;

    fn install_path(&self) -> PathBuf {
        windows_path_parent(self.client_path())
    }

    fn fmt_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name())?;
        writeln!(f, "Client path: {}", self.client_path().display())?;

        if let Some(cmd) = self.launch_command() {
            writeln!(f, "Type: Wine")?;
            writeln!(f, "Program: {}", cmd.program.display())?;
            if !cmd.args.is_empty() {
                writeln!(f, "Args: {}", cmd.args.join(" "))?;
            }
            if !cmd.env.is_empty() {
                writeln!(f)?;
                writeln!(f, "Environment variables:")?;
                for (key, value) in &cmd.env {
                    writeln!(f, "  {}={}", key, value)?;
                }
            }
        } else {
            writeln!(f, "Type: Windows")?;
        }

        Ok(())
    }
}
