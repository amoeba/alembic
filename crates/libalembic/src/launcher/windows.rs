#![cfg(all(target_os = "windows", target_env = "msvc"))]

use std::{
    num::NonZero,
    process::{Child, Command, Stdio},
};

use crate::{
    client_config::WindowsClientConfig,
    inject_config::InjectConfig,
    launcher::traits::ClientLauncher,
    settings::{Account, ClientConfigType, ServerInfo},
};

pub struct WindowsLauncherImpl {
    config: WindowsClientConfig,
    inject_config: Option<InjectConfig>,
    server_info: ServerInfo,
    account_info: Account,
    child: Option<Child>,
}

/// Find cork.exe for native Windows.
/// Looks for the i686-pc-windows-msvc build since cork must be 32-bit
/// to inject into 32-bit acclient.exe.
fn find_cork() -> Result<std::path::PathBuf, std::io::Error> {
    let exe_path = std::env::current_exe()?;
    let parent = exe_path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Cannot determine exe directory",
        )
    })?;

    // Strategy 1: Same directory as executable (release/installed)
    let same_dir = parent.join("cork.exe");
    if same_dir.exists() {
        return Ok(same_dir);
    }

    // Strategy 2: Development mode - look in cargo target directory
    // e.g., if exe is at target/debug/desktop.exe, look for target/i686-pc-windows-msvc/debug/cork.exe
    if let Some(target_dir) = parent.parent() {
        if let Some(build_type) = parent.file_name() {
            // Try same build type first (debug/release)
            let same_type_path = target_dir
                .join("i686-pc-windows-msvc")
                .join(build_type)
                .join("cork.exe");
            if same_type_path.exists() {
                return Ok(same_type_path);
            }

            // Fall back to opposite build type
            let other_type = if build_type == "debug" {
                "release"
            } else {
                "debug"
            };
            let other_type_path = target_dir
                .join("i686-pc-windows-msvc")
                .join(other_type)
                .join("cork.exe");
            if other_type_path.exists() {
                return Ok(other_type_path);
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "cork.exe not found. Expected in same directory as executable or target/i686-pc-windows-msvc/[debug|release]/",
    ))
}

impl ClientLauncher for WindowsLauncherImpl {
    fn new(
        client_config: ClientConfigType,
        inject_config: Option<InjectConfig>,
        server_info: ServerInfo,
        account_info: Account,
    ) -> Self {
        let config = match client_config {
            ClientConfigType::Windows(windows_config) => windows_config,
            _ => panic!("Windows launcher requires a Windows client configuration"),
        };

        Self {
            config,
            inject_config,
            server_info,
            account_info,
            child: None,
        }
    }

    fn launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        let cork_path = find_cork()?;

        println!("Using cork: {}", cork_path.display());

        let mut cmd = Command::new(&cork_path);

        cmd.arg("launch")
            .arg("--client")
            .arg(self.config.client_path.display().to_string())
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
            cmd.arg("--dll")
                .arg(inject_config.dll_path.display().to_string());

            if let Some(func) = &inject_config.startup_function {
                cmd.arg("--function").arg(func);
            }
        }

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        // Print launch info
        println!("Launching via cork...");
        println!("  Cork: {}", cork_path.display());
        println!("  Client: {}", self.config.client_path.display());
        println!(
            "  Server: {}:{}",
            self.server_info.hostname, self.server_info.port
        );
        println!("  Account: {}", self.account_info.username);
        if let Some(inject_config) = &self.inject_config {
            println!(
                "  DLL: {} ({})",
                inject_config.dll_path.display(),
                inject_config.dll_type
            );
            if let Some(func) = &inject_config.startup_function {
                println!("  Function: {}", func);
            }
        }

        let child = cmd.spawn()?;
        let pid = child.id();
        self.child = Some(child);

        Ok(NonZero::new(pid).unwrap())
    }

    fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        // Cork handles process creation, so just launch
        self.launch()
    }

    fn inject(&mut self) -> Result<(), anyhow::Error> {
        println!("Windows DLL injection is handled via cork during launch");
        Ok(())
    }

    fn eject(&mut self) -> Result<(), anyhow::Error> {
        println!("DLL ejection not yet implemented for Windows cork mode");
        Ok(())
    }
}
