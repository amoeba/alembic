#![cfg(all(target_os = "windows", target_env = "msvc", feature = "alembic"))]

use std::{
    error::Error,
    ffi::OsString,
    fs,
    num::NonZero,
    os::windows::{
        ffi::OsStrExt,
        io::{AsHandle, AsRawHandle},
    },
};

use anyhow::bail;
use dll_syringe::process::{OwnedProcess, Process};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, GetLastError, HANDLE},
        System::Threading::{
            CreateProcessW, ResumeThread, CREATE_SUSPENDED, PROCESS_INFORMATION, STARTUPINFOW,
        },
    },
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
    client: Option<OwnedProcess>,
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
            client: None,
        }
    }

    fn launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let cmd_line = format!(
            "{} -h {} -p {} -a {} -v {}",
            self.config.client_path.display(),
            self.server_info.hostname,
            self.server_info.port,
            self.account_info.username,
            self.account_info.password,
        );

        // Get the parent directory of acclient.exe as the working directory
        let current_dir = self
            .config
            .client_path
            .parent()
            .map(|p| format!("{}\\", p.display()))
            .unwrap_or_else(|| String::from(".\\"));

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
                inject_config.dll_type,
                dll_path.display()
            );

            let client = self
                .client
                .as_ref()
                .expect("No client process to inject into");

            // Use the injector module to inject and optionally call the startup function
            let handle = HANDLE(client.as_handle().as_raw_handle() as *mut std::ffi::c_void);
            crate::injector::inject_into_process(handle, dll_path.to_str().unwrap(), inject_config.startup_function.as_deref())?;

            println!(
                "Successfully injected {} DLL{}",
                inject_config.dll_type,
                if inject_config.startup_function.is_some() {
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
