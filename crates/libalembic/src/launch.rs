#![allow(dead_code)]

use std::{
    error::Error,
    num::NonZero,
    process::{Child, Command, Stdio},
};

use crate::{
    client_config::{ClientConfig, InjectConfig, WineClientConfig, WindowsClientConfig},
    settings::{Account, ServerInfo},
};

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use std::fs;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use anyhow::bail;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use std::{ffi::OsString, os::windows::ffi::OsStrExt};

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use crate::inject::InjectionKit;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use dll_syringe::process::{OwnedProcess, Process};

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, GetLastError},
        System::Threading::{
            CreateProcessW, ResumeThread, CREATE_SUSPENDED, PROCESS_INFORMATION, STARTUPINFOW,
        },
    },
};

pub struct Launcher {
    client_config: ClientConfig,
    inject_config: Option<InjectConfig>,
    server_info: ServerInfo,
    account_info: Account,
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    client: Option<OwnedProcess>,
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    injector: Option<InjectionKit>,
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    child_pid: Option<u32>,
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    child: Option<Child>,
}

impl<'a> Launcher {
    pub fn new(
        client_config: ClientConfig,
        inject_config: Option<InjectConfig>,
        server_info: ServerInfo,
        account_info: Account,
    ) -> Self {
        Launcher {
            client_config,
            inject_config,
            server_info,
            account_info,
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            client: None,
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            injector: None,
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            child_pid: None,
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            child: None,
        }
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn launch_windows(
        &self,
        config: &WindowsClientConfig,
    ) -> Result<PROCESS_INFORMATION, Box<dyn Error>> {
        let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let cmd_line = format!(
            "{}\\acclient.exe -h {} -p {} -a {} -v {}",
            config.install_path.display(),
            self.server_info.hostname,
            self.server_info.port,
            self.account_info.username,
            self.account_info.password,
        );

        let current_dir = format!("{}\\", config.install_path.display());

        let cmd_line_wide: Vec<u16> = OsString::from(&cmd_line)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let current_dir_wide: Vec<u16> = OsString::from(&current_dir)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let result = CreateProcessW(
                None,
                PWSTR(cmd_line_wide.as_ptr() as *mut _),
                None,
                None,
                false,
                CREATE_SUSPENDED,
                None,
                PWSTR(current_dir_wide.as_ptr() as *mut _),
                &startup_info,
                &mut process_info,
            );

            match result {
                Ok(_) => {
                    println!("Process created with ID: {}", process_info.dwProcessId);
                    let resume_result = ResumeThread(process_info.hThread);
                    if resume_result == u32::MAX {
                        println!("Failed to resume thread. Last error: {:?}", GetLastError());
                    }

                    let _ = CloseHandle(process_info.hThread);
                    let _ = CloseHandle(process_info.hProcess);

                    Ok(process_info)
                }
                Err(error) => {
                    eprintln!("CreateProcessW failure: {error:?}");
                    Err(error.into())
                }
            }
        }
    }

    fn launch_wine(&self, config: &WineClientConfig) -> Result<Child, Box<dyn Error>> {
        let client_exe = format!("{}\\acclient.exe", config.install_path.display());

        let mut cmd = Command::new(&config.wine_executable);

        // Set WINEPREFIX
        cmd.env("WINEPREFIX", &config.prefix_path);

        // Set additional environment variables
        for (key, value) in &config.additional_env {
            cmd.env(key, value);
        }

        // Convert Windows path to Unix path for working directory
        // e.g., "C:\AC" -> "$WINEPREFIX/drive_c/AC"
        let windows_path_str = config.install_path.display().to_string();
        let unix_path = windows_path_str
            .trim_start_matches("C:\\")
            .trim_start_matches("C:/")
            .replace("\\", "/");
        let working_dir = config.prefix_path.join("drive_c").join(&unix_path);

        cmd.current_dir(&working_dir);

        println!("Launching client directly (cork will be called separately)");
        println!("Launching: {} -h {} -p {} -a {}",
            client_exe,
            self.server_info.hostname,
            self.server_info.port,
            self.account_info.username
        );

        // Launch client directly
        cmd.arg(&client_exe)
            .arg("-h")
            .arg(&self.server_info.hostname)
            .arg("-p")
            .arg(&self.server_info.port)
            .arg("-a")
            .arg(&self.account_info.username)
            .arg("-v")
            .arg(&self.account_info.password);

        // Pipe stdout/stderr so we can capture and display in TUI
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn()?;

        Ok(child)
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn find(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);
        }

        Ok(())
    }

    pub fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        match &self.client_config {
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            ClientConfig::Windows(_) => self.find_or_launch_windows(),
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            ClientConfig::Windows(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Windows launch mode is only supported on Windows",
            )),
            ClientConfig::Wine(_) => self.find_or_launch_wine(),
        }
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn find_or_launch_windows(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);

            match self.client.as_ref().unwrap().pid() {
                Ok(val) => return Ok(val.clone()),
                Err(err) => return Err(err),
            }
        }

        println!("Couldn't find existing client to inject into. Launching instead.");

        if let ClientConfig::Windows(config) = &self.client_config {
            match self.launch_windows(config) {
                Ok(process_info) => {
                    self.client = Some(OwnedProcess::from_pid(process_info.dwProcessId).unwrap());

                    Ok(NonZero::new(process_info.dwProcessId).unwrap())
                }
                Err(error) => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    error.to_string(),
                )),
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Expected Windows client config",
            ))
        }
    }

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    fn get_windows_pid_from_wine(&self, config: &WineClientConfig) -> Result<u32, Box<dyn Error>> {
        use std::thread;
        use std::time::Duration;

        println!("Waiting for acclient.exe to start in wine...");
        thread::sleep(Duration::from_millis(2000));

        let mut cmd = Command::new(&config.wine_executable);
        cmd.env("WINEPREFIX", &config.prefix_path);

        for (key, value) in &config.additional_env {
            cmd.env(key, value);
        }

        cmd.arg("winedbg");
        cmd.arg("--command");
        cmd.arg("info proc");

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        println!("winedbg output:\n{}", stdout);

        // Parse the output to find acclient.exe
        for line in stdout.lines() {
            if line.contains("acclient.exe") {
                println!("Found acclient.exe line: {}", line);
                // Try to extract PID from the line
                // Format is typically: 00000028 13       'acclient.exe'
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pid_str) = parts.first() {
                    // Parse hex PID
                    if let Ok(pid) = u32::from_str_radix(pid_str, 16) {
                        println!("Extracted Windows PID: 0x{} (decimal: {})", pid_str, pid);
                        return Ok(pid);
                    }
                }
            }
        }

        Err("Could not find Windows PID for acclient.exe".into())
    }

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    fn call_cork_with_pid(&self, config: &WineClientConfig, windows_pid: u32) -> Result<(), Box<dyn Error>> {
        // Try both cork.exe and cork (for cross-platform compatibility)
        let cork_path = std::env::current_exe()
            .ok()
            .and_then(|p| {
                let parent = p.parent()?;
                let exe_path = parent.join("cork.exe");
                if exe_path.exists() {
                    Some(exe_path)
                } else {
                    let unix_path = parent.join("cork");
                    if unix_path.exists() {
                        Some(unix_path)
                    } else {
                        None
                    }
                }
            });

        if let Some(cork_path) = cork_path {
            println!("Calling cork with Windows PID {}", windows_pid);

                let mut cmd = Command::new(&config.wine_executable);
                cmd.env("WINEPREFIX", &config.prefix_path);

                for (key, value) in &config.additional_env {
                    cmd.env(key, value);
                }

                cmd.arg(cork_path.to_str().ok_or("Invalid cork path")?)
                    .arg("--client")
                    .arg("dummy")
                    .arg("--hostname")
                    .arg(&self.server_info.hostname)
                    .arg("--port")
                    .arg(&self.server_info.port)
                    .arg("--account")
                    .arg(&self.account_info.username)
                    .arg("--password")
                    .arg(&self.account_info.password)
                    .arg("--pid")
                    .arg(windows_pid.to_string());

            let output = cmd.output()?;
            println!("Cork output:\n{}", String::from_utf8_lossy(&output.stdout));
            println!("Cork stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        } else {
            println!("cork binary not found in target/debug directory");
        }

        Ok(())
    }

    fn find_or_launch_wine(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        if let ClientConfig::Wine(config) = &self.client_config {
            match self.launch_wine(config) {
                Ok(child) => {
                    let unix_pid = child.id();
                    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
                    {
                        self.child_pid = Some(unix_pid);
                        self.child = Some(child);

                        // Get Windows PID and call cork
                        match self.get_windows_pid_from_wine(config) {
                            Ok(windows_pid) => {
                                println!("Unix PID: {}, Windows PID: {}", unix_pid, windows_pid);
                                if let Err(e) = self.call_cork_with_pid(config, windows_pid) {
                                    println!("Error calling cork: {}", e);
                                }
                            }
                            Err(e) => {
                                println!("Error getting Windows PID: {}", e);
                            }
                        }
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
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Expected Wine client config",
            ))
        }
    }

    pub fn attach_injected(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn launch_injected(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn inject(&mut self) -> Result<(), anyhow::Error> {
        match &self.client_config {
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            ClientConfig::Windows(_) => self.inject_windows(),
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            ClientConfig::Windows(_) => {
                println!("Windows DLL injection not available on this platform");
                Ok(())
            }
            ClientConfig::Wine(config) => self.inject_wine(config),
        }
    }

    /// Wait for and find the acclient.exe process PID
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    fn wait_for_acclient(&self) -> Result<u32, anyhow::Error> {
        use std::process::Command;
        use std::thread;
        use std::time::Duration;

        println!("Waiting for acclient.exe process to start...");

        // Try up to 10 times with 500ms delays (5 seconds total)
        for attempt in 1..=10 {
            // Use pgrep to find acclient.exe process
            let output = Command::new("pgrep")
                .arg("-f")
                .arg("acclient.exe")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(first_pid) = stdout.lines().next() {
                        if let Ok(pid) = first_pid.trim().parse::<u32>() {
                            println!("Found acclient.exe with PID: {}", pid);
                            return Ok(pid);
                        }
                    }
                }
            }

            if attempt < 10 {
                println!("Attempt {}/10: acclient.exe not found yet, waiting...", attempt);
                thread::sleep(Duration::from_millis(500));
            }
        }

        anyhow::bail!("Timeout waiting for acclient.exe process to start")
    }

    fn inject_wine(&self, _config: &WineClientConfig) -> Result<(), anyhow::Error> {
        // DLL injection for Wine is not yet implemented
        // Cork now handles launching the client directly with connection parameters
        if self.inject_config.is_some() {
            println!("Note: DLL injection is not yet implemented for Wine clients.");
            println!("The client was launched via cork without injection.");
        }

        Ok(())
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn inject_windows(&mut self) -> Result<(), anyhow::Error> {
        self.injector = match &self.client {
            Some(client) => Some(InjectionKit::new(client.try_clone().unwrap())),
            None => panic!("Could not create InjectionKit."),
        };

        if let Some(inject_config) = &self.inject_config {
            // Get the filesystem path (for Windows configs, this is just dll_path)
            let dll_path = inject_config.filesystem_path();

            if !fs::exists(&dll_path)? {
                bail!(
                    "Can't find DLL to inject at path {}. Bailing.",
                    dll_path.display()
                );
            }

            println!("Injecting {} DLL from: {}", inject_config.dll_type(), dll_path.display());

            match self.injector.as_mut() {
                Some(kit) => {
                    kit.inject(dll_path.to_str().unwrap())?;
                }
                None => panic!("Could not get access to underlying injector to inject DLL."),
            }
        } else {
            println!("No DLL injection configured.");
        }

        Ok(())
    }

    pub fn eject(&mut self) -> Result<(), anyhow::Error> {
        match &self.client_config {
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            ClientConfig::Windows(_) => self.eject_windows(),
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            ClientConfig::Windows(_) => {
                println!("Windows DLL ejection not available on this platform");
                Ok(())
            }
            ClientConfig::Wine(_) => {
                println!("DLL ejection not applicable in Wine mode");
                Ok(())
            }
        }
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn eject_windows(&mut self) -> Result<(), anyhow::Error> {
        match self.injector.as_mut() {
            Some(injector) => {
                injector.eject()?;
            }
            None => bail!("Eject called with no active injector."),
        }

        Ok(())
    }

    pub fn attach_or_launch_injected(&mut self) -> Result<(), Box<dyn Error>> {
        self.find_or_launch()?;
        self.inject()?;

        Ok(())
    }

    /// Take ownership of the wine child process for stdout/stderr monitoring
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    pub fn take_child(&mut self) -> Option<Child> {
        self.child.take()
    }
}
