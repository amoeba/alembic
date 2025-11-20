use std::{
    error::Error,
    num::NonZero,
    process::{Child, Command, Stdio},
};

use crate::{
    client_config::{ClientConfig, DllType, InjectConfig, WineClientConfig},
    launcher::traits::ClientLauncher,
    settings::{Account, ServerInfo},
};

/// Wine-specific launcher implementation
pub struct WineLauncherImpl {
    config: WineClientConfig,
    inject_config: Option<InjectConfig>,
    server_info: ServerInfo,
    account_info: Account,
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    child_pid: Option<u32>,
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    child: Option<Child>,
}

impl WineLauncherImpl {
    pub fn attach_or_launch_injected(&mut self) -> Result<(), Box<dyn Error>> {
        self.find_or_launch()?;
        Ok(())
    }

    /// Launch client using cork (which handles injection if configured)
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    fn launch_with_cork(&self) -> Result<Child, Box<dyn Error>> {
        use std::path::PathBuf;

        // Find cork.exe - try multiple locations for dev vs release
        let cork_path = std::env::current_exe()
            .ok()
            .and_then(|exe_path| {
                let parent = exe_path.parent()?;

                // Strategy 1: Same directory as executable (release/installed)
                let same_dir = parent.join("cork.exe");
                if same_dir.exists() {
                    return Some(same_dir);
                }

                // Strategy 2: Development mode - look in cargo target directory
                // e.g., if exe is at target/debug/desktop, look for target/x86_64-pc-windows-gnu/debug/cork.exe
                // Try matching build type first (debug/release), then try the other
                if let Some(target_dir) = parent.parent() {
                    let build_type = parent.file_name()?; // "debug" or "release"

                    // Try same build type first
                    let same_type_path = target_dir
                        .join("x86_64-pc-windows-gnu")
                        .join(build_type)
                        .join("cork.exe");
                    if same_type_path.exists() {
                        return Some(same_type_path);
                    }

                    // Fall back to opposite build type
                    let other_type = if build_type == "debug" { "release" } else { "debug" };
                    let other_type_path = target_dir
                        .join("x86_64-pc-windows-gnu")
                        .join(other_type)
                        .join("cork.exe");
                    if other_type_path.exists() {
                        return Some(other_type_path);
                    }
                }

                None
            })
            .ok_or("cork.exe not found. Expected in same directory as executable or target/x86_64-pc-windows-gnu/[debug|release]/")?;

        let client_exe = format!("{}\\acclient.exe", self.config.install_path.display());

        let mut cmd = Command::new(&self.config.wine_executable);
        cmd.env("WINEPREFIX", &self.config.prefix_path);

        // Set additional environment variables
        for (key, value) in &self.config.additional_env {
            cmd.env(key, value);
        }

        println!("Launching client via Wine using cork");
        println!("  Client: {}", client_exe);
        println!(
            "  Server: {}:{}",
            self.server_info.hostname, self.server_info.port
        );
        println!("  Account: {}", self.account_info.username);

        // Build cork command
        cmd.arg(cork_path.to_str().ok_or("Invalid cork path")?)
            .arg("launch")
            .arg("--client")
            .arg(&client_exe)
            .arg("--hostname")
            .arg(&self.server_info.hostname)
            .arg("--port")
            .arg(&self.server_info.port)
            .arg("--account")
            .arg(&self.account_info.username)
            .arg("--password")
            .arg(&self.account_info.password);

        // Add DLL injection parameters if configured
        if let Some(inject_config) = &self.inject_config {
            println!("  DLL: {}", inject_config.dll_path().display());

            cmd.arg("--dll")
                .arg(inject_config.dll_path().display().to_string());

            // Determine if we need to call a function after injection
            let dll_function = match inject_config.dll_type() {
                DllType::Decal => Some("DecalStartup"),
                DllType::Alembic => None,
            };

            // TEMPORARILY DISABLED: Calling the function causes issues under Wine/MinGW
            // if let Some(func) = dll_function {
            //     println!("  Function: {}", func);
            //     cmd.arg("--function").arg(func);
            // }
        }

        // For debugging: inherit stdout/stderr so we can see cork's output
        // Later we can pipe and capture in threads like the CLI does
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let child = cmd.spawn()?;
        Ok(child)
    }

    /// Fallback: launch client directly without cork (no injection support)
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn launch_with_cork(&self) -> Result<Child, Box<dyn Error>> {
        Err("Cork launching not supported on Windows MSVC".into())
    }

    /// Take ownership of the wine child process for stdout/stderr monitoring
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    pub fn take_child(&mut self) -> Option<Child> {
        self.child.take()
    }
}

impl ClientLauncher for WineLauncherImpl {
    fn new(
        client_config: ClientConfig,
        inject_config: Option<InjectConfig>,
        server_info: ServerInfo,
        account_info: Account,
    ) -> Self {
        let config = match client_config {
            ClientConfig::Wine(config) => config,
            ClientConfig::Windows(_) => {
                panic!("Windows launcher is only supported on Windows MSVC platform")
            }
        };

        Self {
            config,
            inject_config,
            server_info,
            account_info,
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            child_pid: None,
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            child: None,
        }
    }

    fn launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        println!("Launching new client via Wine...");

        match self.launch_with_cork() {
            Ok(child) => {
                let unix_pid = child.id();
                #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
                {
                    self.child_pid = Some(unix_pid);
                    self.child = Some(child);
                }
                #[cfg(all(target_os = "windows", target_env = "msvc"))]
                {
                    let _ = child; // Consume child on Windows
                }
                Ok(NonZero::new(unix_pid).unwrap())
            }
            Err(error) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            )),
        }
    }

    fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        // For Wine, we don't typically find existing processes - just launch
        // (finding Wine processes is complex and not commonly needed)
        self.launch()
    }

    fn inject(&mut self) -> Result<(), anyhow::Error> {
        // For Wine, injection happens during find_or_launch via cork
        if self.inject_config.is_some() {
            println!("Wine DLL injection is handled via cork during launch");
        }
        Ok(())
    }

    fn eject(&mut self) -> Result<(), anyhow::Error> {
        println!("DLL ejection not applicable in Wine mode");
        Ok(())
    }
}
