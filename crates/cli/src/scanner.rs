use anyhow::Result;
use std::path::PathBuf;
use std::fs;
use std::process::Command;

/// Represents a discovered wine prefix or client installation
#[derive(Debug, Clone)]
pub struct DiscoveredInstallation {
    pub prefix_path: Option<PathBuf>,
    pub client_path: Option<PathBuf>,
    pub wine_executable: Option<PathBuf>,
    pub display_name: String,
}

/// Scanning method for finding AC clients
#[derive(Debug, Clone)]
pub enum ScanMethod {
    /// Native Windows registry scanning
    Native,
    /// Wine-based scanning (macOS and Linux)
    Wine {
        /// Wine executable to use
        wine_path: PathBuf,
    },
    /// Whisky bottle scanning (macOS only)
    Whisky,
}

impl ScanMethod {
    /// Check if this scan method is available on the current system
    pub fn is_available(&self) -> bool {
        match self {
            ScanMethod::Native => {
                #[cfg(target_os = "windows")]
                {
                    true // Always available on Windows
                }
                #[cfg(not(target_os = "windows"))]
                {
                    false // Not available on non-Windows platforms
                }
            }
            ScanMethod::Wine { wine_path } => {
                // Check if wine executable exists and is executable
                wine_path.exists() && wine_path.is_file()
            }
            ScanMethod::Whisky => {
                #[cfg(target_os = "macos")]
                {
                    // Check if whisky CLI is available by trying to run 'whisky list'
                    Command::new("whisky")
                        .arg("list")
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    false // Whisky is macOS-only
                }
            }
        }
    }

    /// Scan for AC client installations using this method
    pub fn scan(&self) -> Result<Vec<DiscoveredInstallation>> {
        match self {
            ScanMethod::Native => self.scan_native(),
            ScanMethod::Wine { wine_path } => self.scan_wine(wine_path),
            ScanMethod::Whisky => self.scan_whisky(),
        }
    }

    /// Scan using native Windows registry
    #[cfg(target_os = "windows")]
    fn scan_native(&self) -> Result<Vec<DiscoveredInstallation>> {
        let mut installations = Vec::new();

        // TODO: Use Windows registry API to find AC installations
        println!("Windows native scanning stubbed - not yet implemented");

        Ok(installations)
    }

    #[cfg(not(target_os = "windows"))]
    fn scan_native(&self) -> Result<Vec<DiscoveredInstallation>> {
        anyhow::bail!("Native scanning is only available on Windows");
    }

    /// Scan using wine (for macOS and Linux)
    #[cfg(not(target_os = "windows"))]
    fn scan_wine(&self, wine_path: &PathBuf) -> Result<Vec<DiscoveredInstallation>> {
        let mut installations = Vec::new();

        // Get home directory
        let home = match std::env::var("HOME") {
            Ok(h) => PathBuf::from(h),
            Err(_) => {
                eprintln!("Warning: Could not determine HOME directory");
                return Ok(installations);
            }
        };

        // Scan standard wine prefix locations
        let standard_prefixes = vec![
            home.join(".wine"),
            home.join(".local/share/wineprefixes"),
        ];

        for prefix_base in standard_prefixes {
            if prefix_base.exists() && prefix_base.is_dir() {
                // If it's .wine, it's a prefix itself
                if prefix_base.file_name().and_then(|n| n.to_str()) == Some(".wine") {
                    scan_wine_prefix(&prefix_base, wine_path, &mut installations)?;
                } else {
                    // Otherwise, scan subdirectories as potential prefixes
                    if let Ok(entries) = fs::read_dir(&prefix_base) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                scan_wine_prefix(&path, wine_path, &mut installations)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(installations)
    }

    #[cfg(target_os = "windows")]
    fn scan_wine(&self, _wine_path: &PathBuf) -> Result<Vec<DiscoveredInstallation>> {
        anyhow::bail!("Wine scanning is not available on Windows");
    }

    /// Scan for Whisky bottles (macOS only)
    /// Discovers wine prefixes via Whisky CLI and uses shared Wine scanning logic
    #[cfg(target_os = "macos")]
    fn scan_whisky(&self) -> Result<Vec<DiscoveredInstallation>> {
        let mut installations = Vec::new();

        // Use whisky CLI to list bottles
        let output = Command::new("whisky")
            .arg("list")
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to run 'whisky list' command");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse the output to extract bottle names
        // Format is a table with columns: Name | Windows Version | Path
        for line in stdout.lines() {
            // Skip header lines and separators
            if line.starts_with('+') || line.starts_with('|') && line.contains("Name") {
                continue;
            }

            // Parse data lines like: | AC   | Windows 10      | ~/Library/... |
            if line.starts_with('|') {
                let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    let bottle_name = parts[1];

                    // Get wine path and prefix path from whisky shellenv
                    let (wine_exe, bottle_path) = get_whisky_info(bottle_name);

                    // Use shared wine prefix scanning logic
                    if !bottle_path.as_os_str().is_empty() && bottle_path.exists() && bottle_path.is_dir() {
                        let start_count = installations.len();
                        scan_wine_prefix(&bottle_path, &wine_exe, &mut installations)?;

                        // Update display names for newly added installations to indicate these are Whisky bottles
                        for install in installations.iter_mut().skip(start_count) {
                            if install.prefix_path.as_ref() == Some(&bottle_path) {
                                // Replace just the prefix description, preserve the full client path
                                if install.client_path.is_some() {
                                    install.display_name = format!("Whisky Bottle: {} (AC at {})",
                                        bottle_name,
                                        install.client_path.as_ref().unwrap().display());
                                } else {
                                    install.display_name = format!("Whisky Bottle: {} (no AC found)", bottle_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(installations)
    }

    #[cfg(not(target_os = "macos"))]
    fn scan_whisky(&self) -> Result<Vec<DiscoveredInstallation>> {
        anyhow::bail!("Whisky scanning is only available on macOS");
    }

}

/// Scan a wine prefix for AC installations
#[cfg(not(target_os = "windows"))]
fn scan_wine_prefix(
    prefix_path: &PathBuf,
    wine_path: &PathBuf,
    installations: &mut Vec<DiscoveredInstallation>,
) -> Result<()> {
    // Check if this looks like a wine prefix by checking for drive_c
    let drive_c = prefix_path.join("drive_c");
    if !drive_c.exists() || !drive_c.is_dir() {
        return Ok(()); // Not a wine prefix
    }

    let wine_exe = wine_path;

    // Common AC installation paths to check in the filesystem
    let ac_paths = vec![
        "Turbine/Asheron's Call",
        "Program Files/Turbine/Asheron's Call",
        "Program Files (x86)/Turbine/Asheron's Call",
    ];

    let mut found_client = false;

    // Check each potential AC installation path in the filesystem
    for ac_path in ac_paths {
        let client_path = drive_c.join(ac_path);

        if client_path.exists() && client_path.is_dir() {
            installations.push(DiscoveredInstallation {
                prefix_path: Some(prefix_path.clone()),
                client_path: Some(client_path.clone()),
                wine_executable: Some(wine_exe.clone()),
                display_name: format!("Wine Prefix: {} (AC at {})", prefix_path.display(), client_path.display()),
            });

            found_client = true;
            break; // Found one, no need to check other paths
        }
    }

    // If no AC installation found, still add the prefix but without client path
    if !found_client {
        installations.push(DiscoveredInstallation {
            prefix_path: Some(prefix_path.clone()),
            client_path: None,
            wine_executable: Some(wine_exe.clone()),
            display_name: format!("Wine Prefix: {} (no AC found)", prefix_path.display()),
        });
    }

    Ok(())
}

/// Get the wine path and prefix path for a Whisky bottle using 'whisky shellenv'
#[cfg(target_os = "macos")]
fn get_whisky_info(bottle_name: &str) -> (PathBuf, PathBuf) {
    // Run whisky shellenv to get the wine path and prefix
    if let Ok(output) = Command::new("whisky")
        .arg("shellenv")
        .arg(bottle_name)
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            let mut wine_path = None;
            let mut prefix_path = None;

            // Parse the output to find the PATH and WINEPREFIX exports
            for line in stdout.lines() {
                if line.starts_with("export PATH=") {
                    // Extract the path from: export PATH="/path/to/wine/bin:$PATH"
                    if let Some(path_part) = line.strip_prefix("export PATH=") {
                        // Remove quotes and take the first path component (before :$PATH)
                        let path_part = path_part.trim_matches('"');
                        if let Some(wine_bin_path) = path_part.split(':').next() {
                            let wine_exe = PathBuf::from(wine_bin_path).join("wine64");
                            if wine_exe.exists() {
                                wine_path = Some(wine_exe);
                            }
                        }
                    }
                } else if line.starts_with("export WINEPREFIX=") {
                    // Extract the prefix from: export WINEPREFIX="/path/to/prefix"
                    if let Some(prefix_part) = line.strip_prefix("export WINEPREFIX=") {
                        let prefix_part = prefix_part.trim_matches('"');
                        // Expand ~ if present
                        let expanded_prefix = if prefix_part.starts_with("~/") {
                            if let Ok(home) = std::env::var("HOME") {
                                PathBuf::from(home).join(&prefix_part[2..])
                            } else {
                                PathBuf::from(prefix_part)
                            }
                        } else {
                            PathBuf::from(prefix_part)
                        };
                        prefix_path = Some(expanded_prefix);
                    }
                }
            }

            if let (Some(wine), Some(prefix)) = (wine_path, prefix_path) {
                return (wine, prefix);
            }
        }
    }

    // Fallback
    let fallback_wine_paths = vec![
        PathBuf::from("/usr/local/bin/wine64"),
        PathBuf::from("/opt/homebrew/bin/wine64"),
    ];

    let fallback_wine = fallback_wine_paths.into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("wine64"));

    (fallback_wine, PathBuf::new()) // Return empty path if we can't get the prefix
}

/// Scans for AC client installations on the current platform
pub fn scan_installations() -> Result<Vec<DiscoveredInstallation>> {
    let mut all_installations = Vec::new();

    // Get all potential scan methods
    let scan_methods = get_all_scan_methods();

    // Filter to only available methods and scan with each
    for method in scan_methods {
        if method.is_available() {
            match method.scan() {
                Ok(mut installations) => {
                    all_installations.append(&mut installations);
                }
                Err(e) => {
                    eprintln!("Warning: Scan method {:?} failed: {}", method, e);
                }
            }
        }
    }

    Ok(all_installations)
}

/// Get all potential scan methods for the current platform
fn get_all_scan_methods() -> Vec<ScanMethod> {
    let mut methods = Vec::new();

    #[cfg(target_os = "windows")]
    {
        methods.push(ScanMethod::Native);
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Try common wine locations
        let wine_paths = vec![
            PathBuf::from("/opt/homebrew/bin/wine64"),
            PathBuf::from("/usr/local/bin/wine64"),
            PathBuf::from("/usr/bin/wine64"),
            PathBuf::from("/usr/local/bin/wine"),
            PathBuf::from("/usr/bin/wine"),
        ];

        // Add first wine that exists
        for wine_path in wine_paths {
            if wine_path.exists() {
                methods.push(ScanMethod::Wine {
                    wine_path: wine_path.clone(),
                });
                break;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Add Whisky scanning (will check availability when scanning)
        methods.push(ScanMethod::Whisky);
    }

    methods
}
