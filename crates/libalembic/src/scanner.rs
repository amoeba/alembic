use crate::client_config::{WineClientConfig, WindowsClientConfig};
use crate::inject_config::{DllType, InjectConfig};
use crate::settings::ClientConfigType;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Trait for client installation scanners
pub trait ClientScanner {
    /// Returns the name of this scanner (e.g., "Wine", "Whisky", "Windows Registry")
    fn name(&self) -> &str;

    /// Scan for client installations and return discovered configs
    fn scan(&self) -> Result<Vec<ClientConfigType>>;

    /// Check if this scanner is available on the current platform
    fn is_available(&self) -> bool;
}

// ============================================================================
// DLL DETECTION HELPERS
// ============================================================================

/// Convert a Unix path within a Wine prefix to a Windows-style path
fn unix_to_windows_path(unix_path: &Path) -> Result<PathBuf> {
    let path_str = unix_path.display().to_string();

    if let Some(idx) = path_str.find("/drive_c/") {
        let relative = &path_str[idx + 9..]; // Skip "/drive_c/"
        let windows = format!("C:\\{}", relative.replace("/", "\\"));
        Ok(PathBuf::from(windows))
    } else {
        anyhow::bail!("Path does not contain /drive_c/: {}", path_str)
    }
}

/// Scan a Wine prefix for Alembic and Decal DLL installations
fn find_dlls_in_prefix(wine_prefix_path: &Path) -> Vec<InjectConfig> {
    let mut inject_configs = vec![];
    let drive_c = wine_prefix_path.join("drive_c");

    if !drive_c.exists() {
        return inject_configs;
    }

    // Check for Alembic.dll in AC installation directories
    let alembic_search_paths = [
        "Turbine/Asheron's Call",
        "Program Files/Turbine/Asheron's Call",
        "Program Files (x86)/Turbine/Asheron's Call",
        "AC",
        "Games/AC",
    ];

    for search_path in alembic_search_paths {
        let dll_dir = drive_c.join(search_path);
        let alembic_path = dll_dir.join("Alembic.dll");
        if alembic_path.exists() {
            // Convert Unix path to Windows path
            if let Ok(windows_path) = unix_to_windows_path(&alembic_path) {
                inject_configs.push(InjectConfig {
                    dll_type: DllType::Alembic,
                    dll_path: windows_path,
                    startup_function: None,
                    wine_prefix: Some(wine_prefix_path.to_path_buf()),
                });
            }
        }
    }

    // Check for Decal's Inject.dll in common Decal installation directories
    let decal_search_paths = [
        "Decal",
        "Decal 3.0",
        "Program Files/Decal",
        "Program Files/Decal 3.0",
        "Program Files (x86)/Decal",
        "Program Files (x86)/Decal 3.0",
    ];

    for search_path in decal_search_paths {
        let dll_dir = drive_c.join(search_path);
        let decal_path = dll_dir.join("Inject.dll");
        if decal_path.exists() {
            // Convert Unix path to Windows path
            if let Ok(windows_path) = unix_to_windows_path(&decal_path) {
                inject_configs.push(InjectConfig {
                    dll_type: DllType::Decal,
                    dll_path: windows_path,
                    startup_function: Some("DecalStartup".to_string()),
                    wine_prefix: Some(wine_prefix_path.to_path_buf()),
                });
            }
        }
    }

    inject_configs
}

// ============================================================================
// WINE SCANNER
// ============================================================================

pub struct WineScanner {
    wine_executable_path: PathBuf,
}

impl WineScanner {
    pub fn new(wine_executable_path: PathBuf) -> Self {
        Self { wine_executable_path }
    }

    fn scan_prefix(&self, wine_prefix_path: &Path) -> Result<Vec<ClientConfigType>> {
        let mut configs = vec![];

        let drive_c = wine_prefix_path.join("drive_c");
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
                // Convert Unix path to Windows path for acclient.exe
                let windows_exe_path = self.unix_to_windows_path(&exe_path)?;

                let mut env = HashMap::new();
                env.insert("WINEPREFIX".to_string(), wine_prefix_path.display().to_string());

                configs.push(ClientConfigType::Wine(WineClientConfig {
                    name: format!("Wine: {}", wine_prefix_path.display()),
                    client_path: windows_exe_path,
                    wrapper_program: Some(self.wine_executable_path.clone()),
                    env,
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

    fn scan(&self) -> Result<Vec<ClientConfigType>> {
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
                let path_value = line
                    .strip_prefix("export PATH=\"")
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
                let prefix_value = line
                    .strip_prefix("export WINEPREFIX=\"")
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

    fn scan_prefix(
        &self,
        wine_prefix_path: &Path,
        wine_exe: &Path,
        bottle_name: &str,
    ) -> Result<Vec<ClientConfigType>> {
        let mut configs = vec![];

        let drive_c = wine_prefix_path.join("drive_c");
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
                let windows_exe_path = self.unix_to_windows_path(&exe_path)?;

                let mut env = HashMap::new();
                env.insert("WINEPREFIX".to_string(), wine_prefix_path.display().to_string());

                configs.push(ClientConfigType::Wine(WineClientConfig {
                    name: format!("Whisky: {}", bottle_name),
                    client_path: windows_exe_path,
                    wrapper_program: Some(wine_exe.to_path_buf()),
                    env,
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

    fn scan(&self) -> Result<Vec<ClientConfigType>> {
        let mut all_configs = vec![];

        // Get list of bottles
        let output = Command::new("whisky").arg("list").output()?;

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

        Command::new("whisky").arg("--version").output().is_ok()
    }
}

// ============================================================================
// WINDOWS SCANNER
// ============================================================================

pub struct WindowsScanner;

impl WindowsScanner {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl ClientScanner for WindowsScanner {
    fn name(&self) -> &str {
        "Windows File System"
    }

    fn scan(&self) -> Result<Vec<ClientConfigType>> {
        let mut configs = vec![];

        // Common AC installation paths on Windows
        let search_paths = [
            r"C:\Turbine\Asheron's Call",
            r"C:\Program Files\Turbine\Asheron's Call",
            r"C:\Program Files (x86)\Turbine\Asheron's Call",
            r"C:\AC",
            r"C:\Games\AC",
        ];

        for search_path in search_paths {
            let path = PathBuf::from(search_path);
            let client_exe = path.join("acclient.exe");

            if client_exe.exists() {
                let name = format!("Asheron's Call - {}", search_path);
                configs.push(ClientConfigType::Windows(WindowsClientConfig {
                    name,
                    client_path: client_exe,
                    env: HashMap::new(),
                }));
            }
        }

        Ok(configs)
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
        scanners.push(Box::new(WindowsScanner::new()));
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
pub fn scan_all() -> Result<Vec<ClientConfigType>> {
    let scanners = get_available_scanners();
    let mut all_configs = vec![];

    for scanner in scanners {
        match scanner.scan() {
            Ok(mut configs) => {
                println!(
                    "Scanner '{}' found {} client(s)",
                    scanner.name(),
                    configs.len()
                );
                all_configs.append(&mut configs);
            }
            Err(e) => {
                eprintln!("Scanner '{}' failed: {}", scanner.name(), e);
            }
        }
    }

    Ok(all_configs)
}

/// Scan for DLLs on Windows
#[cfg(target_os = "windows")]
fn scan_windows_for_dlls() -> Vec<InjectConfig> {
    let mut inject_configs = vec![];

    // Search for Alembic.dll in AC installation directories
    let alembic_search_paths = [
        r"C:\Turbine\Asheron's Call",
        r"C:\Program Files\Turbine\Asheron's Call",
        r"C:\Program Files (x86)\Turbine\Asheron's Call",
        r"C:\AC",
        r"C:\Games\AC",
    ];

    for search_path in alembic_search_paths {
        let alembic_path = PathBuf::from(search_path).join("Alembic.dll");
        if alembic_path.exists() {
            inject_configs.push(InjectConfig {
                dll_path: alembic_path,
                dll_type: DllType::Alembic,
                startup_function: None,
                wine_prefix: None,
            });
        }
    }

    // Search for Decal's Inject.dll
    let decal_search_paths = [
        r"C:\Decal",
        r"C:\Decal 3.0",
        r"C:\Program Files\Decal",
        r"C:\Program Files\Decal 3.0",
        r"C:\Program Files (x86)\Decal",
        r"C:\Program Files (x86)\Decal 3.0",
    ];

    for search_path in decal_search_paths {
        let decal_path = PathBuf::from(search_path).join("Inject.dll");
        if decal_path.exists() {
            inject_configs.push(InjectConfig {
                dll_path: decal_path,
                dll_type: DllType::Decal,
                startup_function: Some("DecalStartup".to_string()),
                wine_prefix: None,
            });
        }
    }

    inject_configs
}

/// Scan specifically for Decal DLL installations
pub fn scan_for_decal_dlls() -> Result<Vec<InjectConfig>> {
    let mut all_dlls = vec![];

    #[cfg(target_os = "windows")]
    {
        all_dlls.append(&mut scan_windows_for_dlls());
    }

    #[cfg(target_os = "macos")]
    {
        let whisky_scanner = WhiskyScanner;
        if whisky_scanner.is_available() {
            if let Ok(mut dlls) = scan_whisky_for_decal_dlls(&whisky_scanner) {
                all_dlls.append(&mut dlls);
            }
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // Try wine prefixes
        if let Ok(wine_path) = find_wine_executable() {
            let scanner = WineScanner::new(wine_path);
            if let Ok(mut dlls) = scan_wine_for_decal_dlls(&scanner) {
                all_dlls.append(&mut dlls);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // TODO: Implement Windows Decal scanning
    }

    Ok(all_dlls)
}

#[cfg(target_os = "macos")]
fn scan_whisky_for_decal_dlls(scanner: &WhiskyScanner) -> Result<Vec<InjectConfig>> {
    let mut all_dlls = vec![];

    // Get list of bottles
    let output = Command::new("whisky").arg("list").output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to list Whisky bottles");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut bottles = vec![];

    // Parse table output
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

    // Scan each bottle for Decal
    for bottle in bottles {
        match scanner.get_bottle_info(&bottle) {
            Ok((_wine_exe, prefix)) => {
                // Find all DLLs in this prefix
                let dll_configs = find_dlls_in_prefix(&prefix);

                for dll_config in dll_configs {
                    if dll_config.dll_type == DllType::Decal {
                        all_dlls.push(dll_config);
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to get info for bottle '{}': {}", bottle, e);
            }
        }
    }

    Ok(all_dlls)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn scan_wine_for_decal_dlls(_scanner: &WineScanner) -> Result<Vec<InjectConfig>> {
    let mut all_dlls = vec![];

    let home = std::env::var("HOME")?;
    let search_dirs = vec![
        PathBuf::from(&home).join(".wine"),
        PathBuf::from(&home).join(".local/share/wineprefixes"),
    ];

    for dir in search_dirs {
        if !dir.exists() {
            continue;
        }

        let prefixes_to_scan = if dir.ends_with(".wine") {
            vec![dir]
        } else {
            // Directory of prefixes
            std::fs::read_dir(&dir)?
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect()
        };

        for prefix in prefixes_to_scan {
            let dll_configs = find_dlls_in_prefix(&prefix);

            for dll_config in dll_configs {
                if dll_config.dll_type == DllType::Decal {
                    all_dlls.push(dll_config);
                }
            }
        }
    }

    Ok(all_dlls)
}

/// Find the AC installation directory in a Wine prefix
#[allow(dead_code)]
fn find_ac_in_prefix(wine_prefix_path: &Path) -> Option<PathBuf> {
    let drive_c = wine_prefix_path.join("drive_c");
    if !drive_c.exists() {
        return None;
    }

    let search_paths = [
        "Turbine/Asheron's Call",
        "Program Files/Turbine/Asheron's Call",
        "Program Files (x86)/Turbine/Asheron's Call",
        "AC",
        "Games/AC",
    ];

    for search_path in search_paths {
        let ac_path = drive_c.join(search_path);
        let exe_path = ac_path.join("acclient.exe");

        if exe_path.exists() {
            return Some(ac_path);
        }
    }

    None
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
