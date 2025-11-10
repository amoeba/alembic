#![allow(dead_code)]

use std::{error::Error, num::NonZero, process::Command};

use crate::{
    settings::{Account, ClientInfo, LaunchConfig, ServerInfo},
    LaunchMode,
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
    launch_mode: LaunchMode,
    client_info: ClientInfo,
    server_info: ServerInfo,
    account_info: Account,
    launch_config: LaunchConfig,
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    client: Option<OwnedProcess>,
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    injector: Option<InjectionKit>,
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    wine_pid: Option<u32>,
}

impl<'a> Launcher {
    pub fn new(
        launch_mode: LaunchMode,
        client_info: ClientInfo,
        server_info: ServerInfo,
        account_info: Account,
        launch_config: LaunchConfig,
    ) -> Self {
        Launcher {
            launch_mode,
            client_info,
            server_info,
            account_info,
            launch_config,
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            client: None,
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            injector: None,
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            wine_pid: None,
        }
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn launch_windows(&self) -> Result<PROCESS_INFORMATION, Box<dyn Error>> {
        let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let cmd_line: Vec<u16> = OsString::from(self.get_cmd_line())
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let current_dir: Vec<u16> = OsString::from(self.get_current_dir())
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let result = CreateProcessW(
                None,
                PWSTR(cmd_line.as_ptr() as *mut _),
                None,
                None,
                false,
                CREATE_SUSPENDED,
                None,
                PWSTR(current_dir.as_ptr() as *mut _),
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

    fn launch_wine(&self) -> Result<u32, Box<dyn Error>> {
        let wine_exe = &self.launch_config.launcher_path;
        let client_path = &self.client_info.path;
        let client_exe = format!("{}\\acclient.exe", client_path);

        let mut cmd = Command::new(wine_exe);

        // Convert Windows path to Unix path within Wine prefix
        // e.g., "C:\AC" -> "$WINEPREFIX/drive_c/AC"
        if let Some(prefix) = &self.launch_config.prefix_path {
            // Strip "C:\" or "C:/" from the beginning and convert backslashes
            let unix_path = client_path
                .trim_start_matches("C:\\")
                .trim_start_matches("C:/")
                .replace("\\", "/");
            let working_dir = format!("{}/drive_c/{}", prefix, unix_path);

            println!("Setting working directory to: {}", working_dir);
            cmd.current_dir(&working_dir);

            cmd.env("WINEPREFIX", prefix);
        }

        for (key, value) in &self.launch_config.environment_variables {
            cmd.env(key, value);
        }

        // Add the game executable and arguments
        cmd.arg(&client_exe)
            .arg("-h").arg(&self.server_info.hostname)
            .arg("-p").arg(&self.server_info.port)
            .arg("-a").arg(&self.account_info.username)
            .arg("-v").arg(&self.account_info.password);

        println!("Launching Wine with command: {:?}", cmd);

        let child = cmd.spawn()?;
        let pid = child.id();

        println!("Wine process launched with PID: {}", pid);

        Ok(pid)
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn find(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);
        }

        Ok(())
    }

    pub fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        match self.launch_mode {
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            LaunchMode::Windows => self.find_or_launch_windows(),
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            LaunchMode::Windows => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Windows launch mode is only supported on Windows"
            )),
            LaunchMode::Wine => self.find_or_launch_wine(),
        }
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn find_or_launch_windows(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        println!("Attempting to find process first before attempting to launch.");

        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);

            match self.client.as_ref().unwrap().pid() {
                Ok(val) => return Ok(val.clone()),
                Err(err) => return Err(err),
            }
        }

        println!("Couldn't find existing client to inject into. Launching instead.");

        match self.launch_windows() {
            Ok(process_info) => {
                self.client = Some(OwnedProcess::from_pid(process_info.dwProcessId).unwrap());

                Ok(NonZero::new(process_info.dwProcessId).unwrap())
            }
            Err(error) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            )),
        }
    }

    fn find_or_launch_wine(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        println!("Launching via Wine (process finding not yet implemented for Wine).");

        match self.launch_wine() {
            Ok(pid) => {
                #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
                {
                    self.wine_pid = Some(pid);
                }
                Ok(NonZero::new(pid).unwrap())
            }
            Err(error) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            )),
        }
    }

    pub fn attach_injected(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn launch_injected(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn inject(&mut self) -> Result<(), anyhow::Error> {
        match self.launch_mode {
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            LaunchMode::Windows => self.inject_windows(),
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            LaunchMode::Windows => {
                println!("Windows DLL injection not available on this platform");
                Ok(())
            }
            LaunchMode::Wine => {
                println!("DLL injection not supported in Wine mode");
                Ok(())
            }
        }
    }

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    fn inject_windows(&mut self) -> Result<(), anyhow::Error> {
        self.injector = match &self.client {
            Some(client) => Some(InjectionKit::new(client.try_clone().unwrap())),
            None => panic!("Could not create InjectionKit."),
        };

        let dll_path = &self.launch_config.launcher_path;
        if !fs::exists(dll_path)? {
            bail!(
                "Can't find DLL to inject at path {}. Bailing.",
                dll_path
            );
        }

        match self.injector.as_mut() {
            Some(kit) => {
                kit.inject(dll_path)?;
            }
            None => panic!("Could not get access to underlying injector to inject DLL."),
        }

        Ok(())
    }

    pub fn eject(&mut self) -> Result<(), anyhow::Error> {
        match self.launch_mode {
            #[cfg(all(target_os = "windows", target_env = "msvc"))]
            LaunchMode::Windows => self.eject_windows(),
            #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
            LaunchMode::Windows => {
                println!("Windows DLL ejection not available on this platform");
                Ok(())
            }
            LaunchMode::Wine => {
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

    // Figure out full path
    fn get_cmd_line(&self) -> String {
        format!(
            "{}\\acclient.exe -h {} -p {} -a {} -v {}",
            self.client_info.path,
            self.server_info.hostname,
            self.server_info.port,
            self.account_info.username,
            self.account_info.password,
        )
    }

    fn get_current_dir(&self) -> String {
        format!("{}", self.client_info.path)
    }
}
