//! Validation utilities for launch configurations.
//!
//! This module provides functions to validate that paths exist before launching,
//! with special handling for Wine where Windows paths need to be checked via Wine.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Result of validating a launch configuration
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            errors: vec![],
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            errors: vec![msg.into()],
        }
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.is_valid = false;
        self.errors.push(msg.into());
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
    }
}

/// Check if a native (Unix) path exists
pub fn validate_native_path(path: &Path, description: &str) -> ValidationResult {
    if path.exists() {
        ValidationResult::ok()
    } else {
        ValidationResult::error(format!("{} not found: {}", description, path.display()))
    }
}

/// Check if a Windows path exists under Wine.
///
/// This runs `wine cmd /c type "path"` and checks the exit code.
/// Returns success if the file exists, error if it doesn't.
pub fn validate_wine_path(
    wine_executable: &Path,
    windows_path: &Path,
    env: &HashMap<String, String>,
    description: &str,
) -> ValidationResult {
    // Convert backslashes to forward slashes - Wine's cmd.exe handles forward slashes
    // correctly, and this avoids issues with shell escaping and path interpretation
    let path_str = windows_path.to_string_lossy().replace('\\', "/");

    // Use "type" command and check exit code - it returns 0 if file exists, 1 if not.
    // Pass command as separate arguments to avoid shell quoting issues.
    let mut cmd = Command::new(wine_executable);
    cmd.arg("cmd")
        .arg("/c")
        .arg("type")
        .arg(&path_str)
        .envs(env)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                ValidationResult::ok()
            } else {
                ValidationResult::error(format!(
                    "{} not found (Wine path): {}",
                    description,
                    windows_path.display()
                ))
            }
        }
        Err(e) => ValidationResult::error(format!(
            "Failed to run Wine to validate {}: {}",
            description, e
        )),
    }
}

/// Determine if a path looks like a Windows path (contains backslashes or drive letter)
pub fn is_windows_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains('\\') || (path_str.len() >= 2 && path_str.chars().nth(1) == Some(':'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_windows_path() {
        assert!(is_windows_path(Path::new(r"C:\Windows\System32")));
        assert!(is_windows_path(Path::new("C:\\Program Files")));
        assert!(is_windows_path(Path::new("D:")));
        assert!(!is_windows_path(Path::new("/usr/bin/wine")));
        assert!(!is_windows_path(Path::new("./relative/path")));
    }

    #[test]
    fn test_validate_native_path() {
        // Current directory should exist
        let result = validate_native_path(Path::new("."), "Current dir");
        assert!(result.is_valid);

        // Non-existent path
        let result = validate_native_path(Path::new("/nonexistent/path/12345"), "Test path");
        assert!(!result.is_valid);
        assert!(result.errors[0].contains("not found"));
    }
}
