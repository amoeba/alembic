use std::{ffi::OsString, os::windows::ffi::OsStrExt, sync::mpsc::channel, thread, time::Duration};

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

fn main() {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    // launch acclient
    let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
    startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

    unsafe {
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
            }
            Err(error) => {
                eprintln!("CreateProcessW failure: {error:?}");
            }
        }
    }
    thread::sleep(Duration::from_millis(1000));
    println!("Process is launched. Starting injection process...");

    let target = OwnedProcess::from_pid(process_info.dwProcessId).unwrap();
    let syringe = Syringe::for_process(target);

    println!("About to inject");
    let injected_payload = syringe
        .inject("target\\i686-pc-windows-msvc\\debug\\alembic.dll")
        .unwrap();

    let remote_add =
        unsafe { syringe.get_payload_procedure::<fn(i32, i32) -> i32>(injected_payload, "add") }
            .unwrap()
            .unwrap();
    let result = remote_add.call(&2, &4).unwrap();
    println!("{}", result); // prints 6

    // Block until Ctrl+C
    rx.recv().expect("Could not receive from channel.");
    println!("ctrl+c received...");

    syringe.eject(injected_payload).unwrap();

    println!("Exiting.");
}
