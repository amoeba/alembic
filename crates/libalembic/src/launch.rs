#![cfg(all(target_os = "windows", target_env = "msvc"))]
#![allow(dead_code)]

use std::{error::Error, ffi::OsString, fs, num::NonZero, os::windows::ffi::OsStrExt};

use crate::{
    inject::InjectionKit,
    settings::{Account, ClientInfo, ServerInfo},
};
use anyhow::bail;
use dll_syringe::process::{OwnedProcess, Process};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, GetLastError},
        System::Threading::{
            CreateProcessW, ResumeThread, CREATE_SUSPENDED, PROCESS_INFORMATION, STARTUPINFOW,
        },
    },
};

#[derive(Debug)]
pub struct Launcher {
    client_info: ClientInfo,
    server_info: ServerInfo,
    account_info: Account,
    dll_path: String,
    pub client: Option<OwnedProcess>,
    injector: Option<InjectionKit>,
}

impl<'a> Launcher {
    pub fn new(
        client_info: ClientInfo,
        server_info: ServerInfo,
        account_info: Account,
        dll_path: String,
    ) -> Self {
        Launcher {
            client_info: client_info,
            server_info: server_info,
            account_info: account_info,
            dll_path: dll_path,
            client: None,
            injector: None,
        }
    }

    pub fn launch(&self) -> Result<PROCESS_INFORMATION, Box<dyn Error>> {
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

    pub fn find(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);
        }

        Ok(())
    }

    pub fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error> {
        println!("Attempting to find process first before attempting to launch.");

        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);

            match self.client.as_ref().unwrap().pid() {
                Ok(val) => return Ok(val.clone()),
                Err(err) => return Err(err),
            }
        }

        println!("Couldn't find existing client to inject into. Launching instead.");

        match self.launch() {
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

    pub fn attach_injected(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn launch_injected(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn inject(&mut self) -> Result<(), anyhow::Error> {
        self.injector = match &self.client {
            Some(client) => Some(InjectionKit::new(client.try_clone().unwrap())),
            None => panic!("Could not create InjectionKit."),
        };

        if !fs::exists(&self.dll_path)? {
            bail!(
                "Can't find DLL to inject at path {}. Bailing.",
                self.dll_path
            );
        }

        match self.injector.as_mut() {
            Some(kit) => {
                kit.inject(&self.dll_path)?;
            }
            None => panic!("Could not get access to underlying injector to inject DLL."),
        }

        Ok(())
    }

    pub fn eject(&mut self) -> Result<(), anyhow::Error> {
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
