use anyhow::Result;
use libalembic::client_config::{ClientConfig, WineClientConfig};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;

/// Trait for client installation scanners
pub trait ClientScanner {
    /// Returns the name of this scanner (e.g., "Wine", "Whisky", "Windows Registry")
    fn name(&self) -> &str;

    /// Scan for client installations and return discovered configs
    fn scan(&self) -> Result<Vec<ClientConfig>>;

    /// Check if this scanner is available on the current platform
    fn is_available(&self) -> bool;
}

// ============================================================================
// WINE SCANNER
// ============================================================================

pub struct WineScanner {
    wine_executable: PathBuf,
}

impl WineScanner {
    pub fn new(wine_executable: PathBuf) -> Self {
        Self { wine_executable }
    }

    fn scan_prefix(&self, prefix_path: &Path) -> Result<Vec<ClientConfig>> {
        let mut configs = vec![];

        let drive_c = prefix_path.join("drive_c");
        if !drive_c.exists() || !drive_c.is_dir() {
            return Ok(configs);
        }

        // Check common AC installation paths
        let search_paths = [
            "Turbine/Asheron's Call",
            "Program Files/Turbine/Asheron's Call",
            "Program Files (x86)/Turbine/Asheron's Call",
        ];

        for search_path in search_paths {
            let ac_path = drive_c.join(search_path);
            let exe_path = ac_path.join("acclient.exe");

            if exe_path.exists() {
                // Convert Unix path to Windows path
                let windows_path = self.unix_to_windows_path(&ac_path)?;

                configs.push(ClientConfig::Wine(WineClientConfig {
                    display_name: format!("Wine: {}", prefix_path.display()),
                    install_path: windows_path,
                    wine_executable: self.wine_executable.clone(),
                    prefix_path: prefix_path.to_path_buf(),
                    additional_env: HashMap::new(),
                }));

                break; // Only add once per prefix
            }
        }

        Ok(configs)
    }

    fn unix_to_windows_path(&self, unix_path: &Path) -> Result<PathBuf> {
        let path_str = unix_path.display().to_string();

        if let Some(idx) = path_str.find("/drive_c/") {
            let relative = &path_str[idx + 9..]; // Skip "/drive_c/"
            let windows = format!("C:\\{}", relative.replace("/", "\\"));
            Ok(PathBuf::from(windows))
        } else {
            anyhow::bail!("Path does not contain /drive_c/: {}", path_str)
        }
    }
}

impl ClientScanner for WineScanner {
    fn name(&self) -> &str {
        "Wine"
    }

    fn scan(&self) -> Result<Vec<ClientConfig>> {
        let mut all_configs = vec![];

        // Check standard wine prefix locations
        let home = std::env::var("HOME")?;
        let search_dirs = vec![
            PathBuf::from(&home).join(".wine"),
            PathBuf::from(&home).join(".local/share/wineprefixes"),
        ];

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }

            if dir.ends_with(".wine") {
                // Single prefix
                if let Ok(mut configs) = self.scan_prefix(&dir) {
                    all_configs.append(&mut configs);
                }
            } else {
                // Directory of prefixes
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            if let Ok(mut configs) = self.scan_prefix(&entry.path()) {
                                all_configs.append(&mut configs);
                            }
                        }
                    }
                }
            }
        }

        Ok(all_configs)
    }

    fn is_available(&self) -> bool {
        cfg!(any(target_os = "macos", target_os = "linux"))
    }
}

// ============================================================================
// WHISKY SCANNER
// ============================================================================

pub struct WhiskyScanner;

impl WhiskyScanner {
    fn get_bottle_info(&self, bottle_name: &str) -> Result<(PathBuf, PathBuf)> {
        let output = Command::new("whisky")
            .arg("shellenv")
            .arg(bottle_name)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to get info for bottle: {}", bottle_name);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut wine_exe: Option<PathBuf> = None;
        let mut prefix: Option<PathBuf> = None;

        for line in stdout.lines() {
            if line.starts_with("export PATH=") {
                let path_value = line.strip_prefix("export PATH=\"")
                    .and_then(|s| s.strip_suffix("\""))
                    .unwrap_or("");

                for path_dir in path_value.split(':') {
                    let wine64 = PathBuf::from(path_dir).join("wine64");
                    if wine64.exists() {
                        wine_exe = Some(wine64);
                        break;
                    }
                }
            } else if line.starts_with("export WINEPREFIX=") {
                let prefix_value = line.strip_prefix("export WINEPREFIX=\"")
                    .and_then(|s| s.strip_suffix("\""))
                    .unwrap_or("");

                // Expand ~ if present
                let expanded = if prefix_value.starts_with("~/") {
                    if let Ok(home) = std::env::var("HOME") {
                        PathBuf::from(home).join(&prefix_value[2..])
                    } else {
                        PathBuf::from(prefix_value)
                    }
                } else {
                    PathBuf::from(prefix_value)
                };

                prefix = Some(expanded);
            }
        }

        match (wine_exe, prefix) {
            (Some(exe), Some(pfx)) => Ok((exe, pfx)),
            _ => anyhow::bail!("Could not extract wine info from Whisky"),
        }
    }

    fn scan_prefix(&self, prefix_path: &Path, wine_exe: &Path, bottle_name: &str) -> Result<Vec<ClientConfig>> {
        let mut configs = vec![];

        let drive_c = prefix_path.join("drive_c");
        if !drive_c.exists() {
            return Ok(configs);
        }

        // Check common AC paths
        let search_paths = [
            "Turbine/Asheron's Call",
            "Program Files/Turbine/Asheron's Call",
            "Program Files (x86)/Turbine/Asheron's Call",
        ];

        for search_path in search_paths {
            let ac_path = drive_c.join(search_path);
            let exe_path = ac_path.join("acclient.exe");

            if exe_path.exists() {
                let windows_path = self.unix_to_windows_path(&ac_path)?;

                configs.push(ClientConfig::Wine(WineClientConfig {
                    display_name: format!("Whisky: {}", bottle_name),
                    install_path: windows_path,
                    wine_executable: wine_exe.to_path_buf(),
                    prefix_path: prefix_path.to_path_buf(),
                    additional_env: HashMap::new(),
                }));

                break;
            }
        }

        Ok(configs)
    }

    fn unix_to_windows_path(&self, unix_path: &Path) -> Result<PathBuf> {
        let path_str = unix_path.display().to_string();

        if let Some(idx) = path_str.find("/drive_c/") {
            let relative = &path_str[idx + 9..];
            let windows = format!("C:\\{}", relative.replace("/", "\\"));
            Ok(PathBuf::from(windows))
        } else {
            anyhow::bail!("Path does not contain /drive_c/: {}", path_str)
        }
    }
}

impl ClientScanner for WhiskyScanner {
    fn name(&self) -> &str {
        "Whisky"
    }

    fn scan(&self) -> Result<Vec<ClientConfig>> {
        let mut all_configs = vec![];

        // Get list of bottles
        let output = Command::new("whisky")
            .arg("list")
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to list Whisky bottles");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut bottles = vec![];

        // Parse table output, skip header and separator lines
        for line in stdout.lines() {
            if line.starts_with('+') || (line.starts_with('|') && line.contains("Name")) {
                continue;
            }

            if line.starts_with('|') {
                let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 2 && !parts[1].is_empty() {
                    bottles.push(parts[1].to_string());
                }
            }
        }

        // Scan each bottle
        for bottle in bottles {
            match self.get_bottle_info(&bottle) {
                Ok((wine_exe, prefix)) => {
                    if let Ok(mut configs) = self.scan_prefix(&prefix, &wine_exe, &bottle) {
                        all_configs.append(&mut configs);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get info for bottle '{}': {}", bottle, e);
                }
            }
        }

        Ok(all_configs)
    }

    fn is_available(&self) -> bool {
        if !cfg!(target_os = "macos") {
            return false;
        }

        Command::new("whisky")
            .arg("--version")
            .output()
            .is_ok()
    }
}

// ============================================================================
// WINDOWS SCANNER
// ============================================================================

pub struct WindowsScanner {
    dll_path: PathBuf,
}

impl WindowsScanner {
    pub fn new(dll_path: PathBuf) -> Self {
        Self { dll_path }
    }
}

impl ClientScanner for WindowsScanner {
    fn name(&self) -> &str {
        "Windows Registry"
    }

    fn scan(&self) -> Result<Vec<ClientConfig>> {
        // TODO: Implement Windows registry scanning
        // This would use Windows registry APIs to find AC installations
        // and return WindowsClientConfig entries
        Ok(vec![])
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "windows")
    }
}

// ============================================================================
// SCANNER ORCHESTRATION
// ============================================================================

/// Get all available scanners for the current platform
pub fn get_available_scanners() -> Vec<Box<dyn ClientScanner>> {
    let mut scanners: Vec<Box<dyn ClientScanner>> = vec![];

    #[cfg(target_os = "windows")]
    {
        // TODO: Get actual DLL path from config or default location
        let dll_path = PathBuf::from("Alembic.dll");
        scanners.push(Box::new(WindowsScanner::new(dll_path)));
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // Try to find wine
        if let Ok(wine_path) = find_wine_executable() {
            scanners.push(Box::new(WineScanner::new(wine_path)));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let whisky_scanner = WhiskyScanner;
        if whisky_scanner.is_available() {
            scanners.push(Box::new(whisky_scanner));
        }
    }

    scanners
}

/// Scan using all available scanners and aggregate results
pub fn scan_all() -> Result<Vec<ClientConfig>> {
    let scanners = get_available_scanners();
    let mut all_configs = vec![];

    for scanner in scanners {
        match scanner.scan() {
            Ok(mut configs) => {
                println!("Scanner '{}' found {} client(s)", scanner.name(), configs.len());
                all_configs.append(&mut configs);
            }
            Err(e) => {
                eprintln!("Scanner '{}' failed: {}", scanner.name(), e);
            }
        }
    }

    Ok(all_configs)
}

/// Find wine executable on the system
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn find_wine_executable() -> Result<PathBuf> {
    // Try common locations
    let candidates = [
        "/usr/local/bin/wine64",
        "/opt/homebrew/bin/wine64",
        "/usr/bin/wine64",
        "/usr/bin/wine",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try PATH
    if let Ok(output) = Command::new("which").arg("wine64").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let path = PathBuf::from(path_str.trim());
            if path.exists() {
                return Ok(path);
            }
        }
    }

    anyhow::bail!("Could not find wine executable")
}

#[cfg(target_os = "windows")]
fn find_wine_executable() -> Result<PathBuf> {
    anyhow::bail!("Wine is not available on Windows")
}
