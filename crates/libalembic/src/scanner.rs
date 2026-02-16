use crate::client_config::{LaunchCommand, WindowsClientConfig, WineClientConfig};
use crate::inject_config::{DllType, InjectConfig};
use crate::settings::{ClientConfigType, SettingsManager};
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Extract WINEPREFIX from a WineClientConfig's launch command.
fn get_wineprefix(wine_config: &WineClientConfig) -> Option<String> {
    wine_config.launch_command.env.get("WINEPREFIX").cloned()
}

/// Trait for client installation scanners
pub trait ClientScanner {
    /// Returns the name of this scanner (e.g., "Wine", "Whisky", "Windows Registry")
    fn name(&self) -> &str;

    /// Scan for client installations and return discovered configs
    fn scan(&self) -> Result<Vec<ClientConfigType>>;

    /// Check if this scanner is available on the current platform
    fn is_available(&self) -> bool;
}

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

/// Convert a Windows-style path back to a Unix path within a Wine prefix
pub fn windows_to_unix_path(prefix: &Path, windows_path: &Path) -> Result<PathBuf> {
    let path_str = windows_path.to_string_lossy();

    // Strip the drive letter (e.g., "C:\") and convert backslashes
    if let Some(rest) = path_str
        .strip_prefix("C:\\")
        .or_else(|| path_str.strip_prefix("c:\\"))
    {
        let unix_relative = rest.replace('\\', "/");
        Ok(prefix.join("drive_c").join(unix_relative))
    } else {
        anyhow::bail!(
            "Path does not start with a drive letter: {}",
            windows_path.display()
        )
    }
}

/// Check if acclient.exe exists anywhere in a Wine prefix
fn prefix_has_acclient(prefix: &Path) -> bool {
    let drive_c = prefix.join("drive_c");
    let search_paths = [
        "Turbine/Asheron's Call",
        "Program Files/Turbine/Asheron's Call",
        "Program Files (x86)/Turbine/Asheron's Call",
    ];

    search_paths
        .iter()
        .any(|search_path| drive_c.join(search_path).join("acclient.exe").exists())
}

/// Discover DLL installations in a Wine prefix.
/// Only returns results if acclient.exe also exists in the prefix,
/// since DLLs are useless without a client to inject into.
pub fn discover_dlls_in_wine_prefix(prefix: &Path) -> Vec<InjectConfig> {
    let mut inject_configs = vec![];

    let drive_c = prefix.join("drive_c");

    // Don't report any DLLs if there's no AC client in this prefix
    if !prefix_has_acclient(prefix) {
        return inject_configs;
    }

    // Check for Alembic.dll in AC installation directories
    let alembic_search_paths = ["Alembic.dll"];
    for search_path in alembic_search_paths {
        let path = drive_c.join(search_path);
        if path.exists() {
            if let Ok(windows_path) = unix_to_windows_path(&path) {
                inject_configs.push(InjectConfig {
                    dll_type: DllType::Alembic,
                    dll_path: windows_path,
                    startup_function: None,
                });
            }
        }
    }

    let decal_search_paths = ["Program Files/Decal 3.0", "Program Files (x86)/Decal 3.0"];
    for search_path in decal_search_paths {
        let inject_dll_path = drive_c.join(search_path).join("Inject.dll");
        if inject_dll_path.exists() {
            if let Ok(dll_path) = unix_to_windows_path(&inject_dll_path) {
                inject_configs.push(InjectConfig {
                    dll_type: DllType::Decal,
                    dll_path,
                    startup_function: Some("DecalStartup".to_string()),
                });
            }
        }
    }

    inject_configs
}

pub struct WineScanner {
    wine_executable_path: PathBuf,
}

impl WineScanner {
    pub fn new(wine_executable_path: PathBuf) -> Self {
        Self {
            wine_executable_path,
        }
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
                let windows_exe_path = unix_to_windows_path(&exe_path)?;

                let launch_command = LaunchCommand::new(&self.wine_executable_path)
                    .env("WINEPREFIX", wine_prefix_path.display().to_string());

                // Discover DLLs in this wine prefix
                let dlls = discover_dlls_in_wine_prefix(wine_prefix_path);

                configs.push(ClientConfigType::Wine(WineClientConfig {
                    name: format!("Wine: {}", wine_prefix_path.display()),
                    client_path: windows_exe_path,
                    launch_command,
                    dlls: dlls.clone(),
                    selected_dll: if !dlls.is_empty() { Some(0) } else { None },
                }));

                break; // Only add once per prefix
            }
        }

        Ok(configs)
    }
}

impl ClientScanner for WineScanner {
    fn name(&self) -> &str {
        "Wine"
    }

    fn scan(&self) -> Result<Vec<ClientConfigType>> {
        let mut all_configs = vec![];
        let mut scanned_prefixes: Vec<PathBuf> = vec![];

        let home = std::env::var("HOME")?;

        // Known wine prefix locations (these are prefixes themselves)
        let known_prefixes = vec![PathBuf::from(&home).join(".wine")];

        // Directories that may contain wine prefixes as subdirectories
        let prefix_containers = vec![PathBuf::from(&home).join(".local/share/wineprefixes")];

        // Scan known prefixes directly
        for prefix in known_prefixes {
            if prefix.exists() && prefix.join("drive_c").is_dir() {
                scanned_prefixes.push(prefix.clone());
                if let Ok(mut configs) = self.scan_prefix(&prefix) {
                    all_configs.append(&mut configs);
                }
            }
        }

        // Scan containers for subdirectories that look like wine prefixes
        for container in prefix_containers {
            if !container.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&container) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join("drive_c").is_dir() {
                        scanned_prefixes.push(path.clone());
                        if let Ok(mut configs) = self.scan_prefix(&path) {
                            all_configs.append(&mut configs);
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

impl WineScanner {
    /// Get the list of prefixes that would be scanned (for reporting)
    pub fn get_scannable_prefixes() -> Vec<PathBuf> {
        let mut prefixes = vec![];

        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return prefixes,
        };

        // Known wine prefix locations
        let known_prefixes = vec![PathBuf::from(&home).join(".wine")];

        // Directories that may contain wine prefixes
        let prefix_containers = vec![PathBuf::from(&home).join(".local/share/wineprefixes")];

        for prefix in known_prefixes {
            if prefix.exists() && prefix.join("drive_c").is_dir() {
                prefixes.push(prefix);
            }
        }

        for container in prefix_containers {
            if !container.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&container) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join("drive_c").is_dir() {
                        prefixes.push(path);
                    }
                }
            }
        }

        prefixes
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
                let expanded = if let Some(suffix) = prefix_value.strip_prefix("~/") {
                    if let Ok(home) = std::env::var("HOME") {
                        PathBuf::from(home).join(suffix)
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
                let windows_exe_path = unix_to_windows_path(&exe_path)?;

                let launch_command = LaunchCommand::new(wine_exe)
                    .env("WINEPREFIX", wine_prefix_path.display().to_string());

                // Discover DLLs in this whisky bottle
                let dlls = discover_dlls_in_wine_prefix(wine_prefix_path);

                configs.push(ClientConfigType::Wine(WineClientConfig {
                    name: format!("Whisky: {}", bottle_name),
                    client_path: windows_exe_path,
                    launch_command,
                    dlls: dlls.clone(),
                    selected_dll: if !dlls.is_empty() { Some(0) } else { None },
                }));

                break;
            }
        }

        Ok(configs)
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
// LUTRIS SCANNER
// ============================================================================

/// Represents a parsed Lutris game configuration
#[derive(Debug, Deserialize)]
struct LutrisGameConfig {
    game: Option<LutrisGameSection>,
    script: Option<LutrisScriptSection>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LutrisGameSection {
    prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LutrisScriptSection {
    installer: Option<Vec<LutrisInstallerStep>>,
}

#[derive(Debug, Deserialize)]
struct LutrisInstallerStep {
    task: Option<LutrisTask>,
}

#[derive(Debug, Deserialize)]
struct LutrisTask {
    wine_path: Option<String>,
}

pub struct LutrisFlatpakScanner;

impl LutrisFlatpakScanner {
    /// Returns the Lutris Flatpak game config directory
    fn get_games_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".var/app/net.lutris.Lutris/data/lutris/games"))
    }

    fn parse_game_config(path: &Path) -> Result<(PathBuf, PathBuf, String)> {
        let content = std::fs::read_to_string(path)?;
        let config: LutrisGameConfig = serde_yml::from_str(&content)?;

        // Get the game name
        let name = config
            .name
            .unwrap_or_else(|| "Unknown Lutris Game".to_string());

        // Get prefix from game section
        let prefix = config
            .game
            .as_ref()
            .and_then(|g| g.prefix.as_ref())
            .ok_or_else(|| anyhow::anyhow!("No prefix found in Lutris config"))?;

        // Get wine_path from script.installer[].task.wine_path
        let wine_path = config
            .script
            .as_ref()
            .and_then(|s| s.installer.as_ref())
            .and_then(|installers| {
                installers
                    .iter()
                    .find_map(|step| step.task.as_ref().and_then(|t| t.wine_path.clone()))
            })
            .ok_or_else(|| anyhow::anyhow!("No wine_path found in Lutris config"))?;

        Ok((PathBuf::from(prefix), PathBuf::from(wine_path), name))
    }

    fn scan_prefix(
        &self,
        wine_prefix_path: &Path,
        game_name: &str,
    ) -> Result<Vec<ClientConfigType>> {
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
                let windows_exe_path = unix_to_windows_path(&exe_path)?;

                let launch_command = LaunchCommand::new("flatpak")
                    .arg("run")
                    .arg("--command=wine")
                    .arg("net.lutris.Lutris")
                    .env("WINEPREFIX", wine_prefix_path.display().to_string());

                // Discover DLLs in this lutris game's wine prefix
                let dlls = discover_dlls_in_wine_prefix(wine_prefix_path);

                configs.push(ClientConfigType::Wine(WineClientConfig {
                    name: format!("Lutris: {}", game_name),
                    client_path: windows_exe_path,
                    launch_command,
                    dlls: dlls.clone(),
                    selected_dll: if !dlls.is_empty() { Some(0) } else { None },
                }));

                break; // Only add once per prefix
            }
        }

        Ok(configs)
    }
}

impl ClientScanner for LutrisFlatpakScanner {
    fn name(&self) -> &str {
        "Lutris (Flatpak)"
    }

    fn scan(&self) -> Result<Vec<ClientConfigType>> {
        let mut all_configs = vec![];

        let games_dir = match Self::get_games_dir() {
            Some(dir) if dir.exists() => dir,
            _ => return Ok(all_configs),
        };

        let entries = match std::fs::read_dir(&games_dir) {
            Ok(e) => e,
            Err(_) => return Ok(all_configs),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yml") {
                match Self::parse_game_config(&path) {
                    Ok((prefix, _wine_path, name)) => {
                        if let Ok(mut configs) = self.scan_prefix(&prefix, &name) {
                            all_configs.append(&mut configs);
                        }
                    }
                    Err(_) => {
                        // Skip configs we can't parse
                    }
                }
            }
        }

        Ok(all_configs)
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "linux")
    }
}

// ============================================================================
// WINDOWS SCANNER
// ============================================================================

pub struct WindowsScanner;

impl Default for WindowsScanner {
    fn default() -> Self {
        Self::new()
    }
}

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
                    dlls: vec![],
                    selected_dll: None,
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

    #[cfg(target_os = "linux")]
    {
        let lutris_scanner = LutrisFlatpakScanner;
        if lutris_scanner.is_available() {
            scanners.push(Box::new(lutris_scanner));
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
                all_configs.append(&mut configs);
            }
            Err(_e) => {
                // Silently skip failed scanners
            }
        }
    }

    Ok(all_configs)
}

/// Discover DLL installations on Windows
#[cfg(target_os = "windows")]
pub fn discover_dlls_on_windows() -> Vec<InjectConfig> {
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
            });
        }
    }

    inject_configs
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn discover_dlls_on_windows() -> Vec<InjectConfig> {
    vec![]
}

/// Scan specifically for Decal DLL installations
pub fn scan_for_decal_dlls() -> Result<Vec<InjectConfig>> {
    let mut all_dlls = vec![];

    #[cfg(target_os = "windows")]
    {
        all_dlls.append(&mut discover_dlls_on_windows());
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On non-Windows, scan wine prefixes from configured clients
        let clients = SettingsManager::get(|s| s.clients.clone());

        for client in clients {
            if let ClientConfigType::Wine(wine_config) = client {
                if let Some(prefix_str) = get_wineprefix(&wine_config) {
                    let prefix = PathBuf::from(prefix_str);
                    if prefix.exists() {
                        let dll_configs = discover_dlls_in_wine_prefix(&prefix);
                        for dll_config in dll_configs {
                            all_dlls.push(dll_config);
                        }
                    }
                }
            }
        }
    }

    Ok(all_dlls)
}

/// Get wine prefixes from configured clients (for DLL scanning reports)
#[cfg(not(target_os = "windows"))]
pub fn get_dll_scannable_prefixes() -> Vec<PathBuf> {
    let clients = SettingsManager::get(|s| s.clients.clone());
    let mut prefixes = vec![];

    for client in clients {
        if let ClientConfigType::Wine(wine_config) = client {
            if let Some(prefix_str) = get_wineprefix(&wine_config) {
                let prefix = PathBuf::from(prefix_str);
                if prefix.exists() {
                    prefixes.push(prefix);
                }
            }
        }
    }

    prefixes
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
                let dll_configs = discover_dlls_in_wine_prefix(&prefix);

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
