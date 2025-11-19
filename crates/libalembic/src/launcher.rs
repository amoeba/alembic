#![allow(dead_code)]

use std::{
    error::Error,
    num::NonZero,
    process::{Child, Command, Stdio},
};

use crate::{
    client_config::{ClientConfig, DllType, InjectConfig},
    settings::{Account, ServerInfo},
};

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use crate::client_config::WindowsClientConfig;

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
use crate::client_config::WineClientConfig;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use std::fs;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use anyhow::bail;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use std::{
    ffi::OsString,
    os::windows::{
        ffi::OsStrExt,
        io::{AsHandle, AsRawHandle},
    },
};

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

/// Trait for platform-specific client launcher implementations
pub trait ClientLauncher: std::any::Any {
    /// Create a new launcher
    fn new(
        client_config: ClientConfig,
        inject_config: Option<InjectConfig>,
        server_info: ServerInfo,
        account_info: Account,
    ) -> Self;

    /// Launch a new client process
    fn launch(&mut self) -> Result<NonZero<u32>, std::io::Error>;

    /// Find or launch the client process (tries to find existing first)
    fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error>;

    /// Inject a DLL into the running client
    fn inject(&mut self) -> Result<(), anyhow::Error>;

    /// Eject the injected DLL
    fn eject(&mut self) -> Result<(), anyhow::Error>;
}

/// Platform-specific launcher type alias
/// On Windows: WindowsLauncherImpl
/// On other platforms: WineLauncherImpl
#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub type Launcher = WindowsLauncherImpl;

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub type Launcher = WineLauncherImpl;

/// Windows-specific launcher implementation
#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub struct WindowsLauncherImpl {
    config: WindowsClientConfig,
    inject_config: Option<InjectConfig>,
    server_info: ServerInfo,
    account_info: Account,
    client: Option<OwnedProcess>,
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
impl WindowsLauncherImpl {
    pub fn attach_or_launch_injected(&mut self) -> Result<(), Box<dyn Error>> {
        self.find_or_launch()?;
        self.inject()?;
        Ok(())
    }
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
impl ClientLauncher for WindowsLauncherImpl {
    fn new(
        client_config: ClientConfig,
        inject_config: Option<InjectConfig>,
        server_info: ServerInfo,
        account_info: Account,
    ) -> Self {
        let config = match client_config {
            ClientConfig::Windows(config) => config,
            ClientConfig::Wine(_) => {
                panic!("Wine launcher is not supported on Windows MSVC platform")
            }
        };

        Self {
            config,
            inject_config,
            server_info,
            account_info,
            client: None,
        }
    }

    fn launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        println!("Launching new client process...");

        let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let cmd_line = format!(
            "{}\\acclient.exe -h {} -p {} -a {} -v {}",
            self.config.install_path.display(),
            self.server_info.hostname,
            self.server_info.port,
            self.account_info.username,
            self.account_info.password,
        );

        let current_dir = format!("{}\\", self.config.install_path.display());

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

                    self.client = Some(OwnedProcess::from_pid(process_info.dwProcessId).unwrap());
                    let pid = NonZero::new(process_info.dwProcessId).unwrap();

                    // Inject if we have an inject config
                    if self.inject_config.is_some() {
                        if let Err(e) = self.inject() {
                            eprintln!("Warning: Failed to inject DLL: {}", e);
                        }
                    }

                    Ok(pid)
                }
                Err(error) => {
                    eprintln!("CreateProcessW failure: {error:?}");
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        error.to_string(),
                    ))
                }
            }
        }
    }

    fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        // Try to find existing acclient process
        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);
            let pid = match self.client.as_ref().unwrap().pid() {
                Ok(val) => val,
                Err(err) => return Err(err),
            };

            // Inject if we have an inject config
            if self.inject_config.is_some() {
                if let Err(e) = self.inject() {
                    eprintln!("Warning: Failed to inject DLL: {}", e);
                }
            }

            return Ok(pid);
        }

        println!("Couldn't find existing client to inject into. Launching instead.");

        // Launch new process (which will handle injection internally)
        self.launch()
    }

    fn inject(&mut self) -> Result<(), anyhow::Error> {
        if let Some(inject_config) = &self.inject_config {
            let dll_path = inject_config.filesystem_path();

            if !fs::exists(&dll_path)? {
                bail!(
                    "Can't find DLL to inject at path {}. Bailing.",
                    dll_path.display()
                );
            }

            println!(
                "Injecting {} DLL from: {}",
                inject_config.dll_type(),
                dll_path.display()
            );

            let client = self
                .client
                .as_ref()
                .expect("No client process to inject into");

            // Determine if we need to call a function after injection
            let dll_function = match inject_config.dll_type() {
                DllType::Decal => Some("DecalStartup"),
                DllType::Alembic => None,
            };

            // Use the injector module to inject and optionally call the startup function
            use windows::Win32::Foundation::HANDLE;
            let handle = HANDLE(client.as_handle().as_raw_handle() as *mut std::ffi::c_void);
            crate::injector::inject_into_process(handle, dll_path.to_str().unwrap(), dll_function)?;

            println!(
                "Successfully injected {} DLL{}",
                inject_config.dll_type(),
                if dll_function.is_some() {
                    " and called startup function"
                } else {
                    ""
                }
            );
        } else {
            println!("No DLL injection configured.");
        }

        Ok(())
    }

    fn eject(&mut self) -> Result<(), anyhow::Error> {
        // TODO: Implement DLL ejection
        println!("DLL ejection not yet implemented for Windows");
        Ok(())
    }
}

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
        self.inject()?;
        Ok(())
    }

    fn launch_wine(&self) -> Result<Child, Box<dyn Error>> {
        let client_exe = format!("{}\\acclient.exe", self.config.install_path.display());

        let mut cmd = Command::new(&self.config.wine_executable);

        // Set WINEPREFIX
        cmd.env("WINEPREFIX", &self.config.prefix_path);

        // Set additional environment variables
        for (key, value) in &self.config.additional_env {
            cmd.env(key, value);
        }

        // Convert Windows path to Unix path for working directory
        let windows_path_str = self.config.install_path.display().to_string();
        let unix_path = windows_path_str
            .trim_start_matches("C:\\")
            .trim_start_matches("C:/")
            .replace("\\", "/");
        let working_dir = self.config.prefix_path.join("drive_c").join(&unix_path);

        cmd.current_dir(&working_dir);

        println!("Launching client via Wine");
        println!(
            "Launching: {} -h {} -p {} -a {}",
            client_exe,
            self.server_info.hostname,
            self.server_info.port,
            self.account_info.username
        );

        // Launch client
        cmd.arg(&client_exe)
            .arg("-h")
            .arg(&self.server_info.hostname)
            .arg("-p")
            .arg(&self.server_info.port)
            .arg("-a")
            .arg(&self.account_info.username)
            .arg("-v")
            .arg(&self.account_info.password);

        // Pipe stdout/stderr
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn()?;

        Ok(child)
    }

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    fn get_windows_pid_from_wine(&self) -> Result<u32, Box<dyn Error>> {
        use std::thread;
        use std::time::Duration;

        println!("Waiting for acclient.exe to start in wine...");
        thread::sleep(Duration::from_millis(2000));

        let mut cmd = Command::new(&self.config.wine_executable);
        cmd.env("WINEPREFIX", &self.config.prefix_path);

        for (key, value) in &self.config.additional_env {
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
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pid_str) = parts.first() {
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
    fn call_cork_with_injection(&self, windows_pid: u32) -> Result<(), Box<dyn Error>> {
        let cork_path = std::env::current_exe().ok().and_then(|p| {
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
            if let Some(inject_config) = &self.inject_config {
                println!(
                    "Calling cork with Windows PID {} for DLL injection",
                    windows_pid
                );

                let mut cmd = Command::new(&self.config.wine_executable);
                cmd.env("WINEPREFIX", &self.config.prefix_path);

                for (key, value) in &self.config.additional_env {
                    cmd.env(key, value);
                }

                // Determine if we need to call a function after injection
                let dll_function = match inject_config.dll_type() {
                    DllType::Decal => Some("DecalStartup"),
                    DllType::Alembic => None,
                };

                cmd.arg(cork_path.to_str().ok_or("Invalid cork path")?)
                    .arg("inject")
                    .arg("--pid")
                    .arg(windows_pid.to_string())
                    .arg("--dll")
                    .arg(inject_config.dll_path().display().to_string());

                if let Some(func) = dll_function {
                    cmd.arg("--function").arg(func);
                }

                let output = cmd.output()?;
                println!("Cork output:\n{}", String::from_utf8_lossy(&output.stdout));
                if !output.stderr.is_empty() {
                    println!("Cork stderr:\n{}", String::from_utf8_lossy(&output.stderr));
                }
            } else {
                println!("No DLL injection configured for Wine launch");
            }
        } else {
            println!("cork binary not found - DLL injection skipped");
        }

        Ok(())
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

        match self.launch_wine() {
            Ok(child) => {
                let unix_pid = child.id();
                #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
                {
                    self.child_pid = Some(unix_pid);
                    self.child = Some(child);

                    // Get Windows PID and call cork for injection if InjectConfig is present
                    if self.inject_config.is_some() {
                        match self.get_windows_pid_from_wine() {
                            Ok(windows_pid) => {
                                println!("Unix PID: {}, Windows PID: {}", unix_pid, windows_pid);
                                if let Err(e) = self.call_cork_with_injection(windows_pid) {
                                    eprintln!("Warning: Failed to inject DLL: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Error getting Windows PID for injection: {}", e);
                            }
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
