#![cfg(all(target_os = "windows", target_env = "msvc"))]
#![allow(dead_code)]

use std::{error::Error, ffi::OsString, fs, os::windows::ffi::OsStrExt};

use crate::inject::InjectionKit;
use anyhow::bail;
use dll_syringe::process::OwnedProcess;
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
    client: Option<OwnedProcess>,
    injector: Option<InjectionKit>,
}

impl<'a> Launcher {
    pub fn new() -> Self {
        Launcher {
            client: None,
            injector: None,
        }
    }

    fn launch(&self) -> Result<PROCESS_INFORMATION, Box<dyn Error>> {
        let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        let cmd_line: Vec<u16> = OsString::from(
            "C:\\Games\\AC\\acclient.exe -h play.coldeve.ac -p 9000 -a treestats -v treestats",
        )
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

        let current_dir: Vec<u16> = OsString::from("C:\\Games\\AC")
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

    fn find(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);
        }

        Ok(())
    }

    pub fn find_or_launch(&mut self) -> Result<(), anyhow::Error> {
        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            self.client = Some(target);
            return Ok(());
        }

        println!("Couldn't find existing client to inject into. Launching instead.");

        match self.launch() {
            Ok(process_info) => {
                self.client = Some(OwnedProcess::from_pid(process_info.dwProcessId).unwrap())
            }
            Err(error) => {
                bail!("Error on launch: {error:?}");
            }
        }

        Ok(())
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

        let dll_path = "target\\i686-pc-windows-msvc\\debug\\alembic.dll";

        if !fs::exists(dll_path)? {
            bail!("Can't find DLL to inject at path {dll_path}. Bailing.");
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
}
