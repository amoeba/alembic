use std::{error::Error, ffi::OsString, os::windows::ffi::OsStrExt, thread, time::Duration};

use dll_syringe::{process::OwnedProcess, Syringe};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, GetLastError},
        System::Threading::{
            CreateProcessW, ResumeThread, CREATE_SUSPENDED, PROCESS_INFORMATION, STARTUPINFOW,
        },
    },
};

use crate::inject::InjectionKit;

pub struct Launcher {
    client: Option<OwnedProcess>,
}

impl Launcher {
    pub fn new() -> Self {
        Launcher { client: None }
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

    fn find_or_launch(&self) -> Result<OwnedProcess, Box<dyn Error>> {
        if let Some(target) = OwnedProcess::find_first_by_name("acclient") {
            return Ok(target);
        }

        println!("Couldn't find existing client to inject into. Launching instead.");

        match self.launch() {
            Ok(process_info) => {
                OwnedProcess::from_pid(process_info.dwProcessId).map_err(|e| e.into())
            }
            Err(error) => {
                println!("Error on launch: {error:?}");
                Err(error.into())
            }
        }
    }

    pub fn attach_or_launch_injected(&self) -> Result<(), Box<dyn Error>> {
        let target = self.find_or_launch()?;

        // debugging
        thread::sleep(Duration::from_secs(5));

        let mut kit = InjectionKit::new(Syringe::for_process(target));
        kit.inject("target\\i686-pc-windows-msvc\\debug\\alembic.dll");

        Ok(())
    }
}
