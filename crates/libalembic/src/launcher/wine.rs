#![cfg(not(all(target_os = "windows", target_env = "msvc", feature = "alembic")))]

use std::{
    num::NonZero,
    process::{Child, Command, Stdio},
};

use crate::{
    client_config::{ClientConfig, WineClientConfig},
    inject_config::InjectConfig,
    launcher::traits::ClientLauncher,
    settings::{Account, ClientConfigType, ServerInfo},
};

pub struct WineLauncherImpl {
    config: WineClientConfig,
    inject_config: Option<InjectConfig>,
    server_info: ServerInfo,
    account_info: Account,
    child_pid: Option<u32>,
    child: Option<Child>,
}

impl WineLauncherImpl {}

impl ClientLauncher for WineLauncherImpl {
    fn new(
        client_config: ClientConfigType,
        inject_config: Option<InjectConfig>,
        server_info: ServerInfo,
        account_info: Account,
    ) -> Self {
        let config = match client_config {
            ClientConfigType::Wine(wine_config) => wine_config,
            _ => panic!("Wine launcher requires a Wine client configuration"),
        };

        Self {
            config,
            inject_config,
            server_info,
            account_info,
            child_pid: None,
            child: None,
        }
    }

    fn launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        println!("Launching new client via Wine...");

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
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "cork.exe not found. Expected in same directory as executable or target/x86_64-pc-windows-gnu/[debug|release]/",
                )
            })?;

        let client_exe = self.config.client_path().display().to_string();

        let wine_exe = self
            .config
            .wrapper_program()
            .expect("Wine config must have wrapper_program set");

        let mut cmd = Command::new(wine_exe);

        // Set Wine environment variables (including WINEPREFIX)
        for (key, value) in self.config.env() {
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
        cmd.arg(cork_path.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid cork path")
        })?)
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
            println!("  DLL: {}", inject_config.dll_path.display());

            cmd.arg("--dll")
                .arg(inject_config.dll_path.display().to_string());

            if let Some(func) = &inject_config.startup_function {
                println!("  Function: {}", func);
                cmd.arg("--function").arg(func);
            }
        }

        // For debugging: inherit stdout/stderr so we can see cork's output
        // Later we can pipe and capture in threads like the CLI does
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let child = cmd.spawn()?;
        let unix_pid = child.id();

        self.child_pid = Some(unix_pid);
        self.child = Some(child);

        Ok(NonZero::new(unix_pid).unwrap())
    }

    fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        println!("find_or_launch not implemented in Wine mode, just calling launch() instead");
        self.launch()
    }

    fn inject(&mut self) -> Result<(), anyhow::Error> {
        println!("Wine DLL injection is handled via cork during launch");
        Ok(())
    }

    fn eject(&mut self) -> Result<(), anyhow::Error> {
        println!("DLL ejection not implemented in Wine mode");
        Ok(())
    }
}
