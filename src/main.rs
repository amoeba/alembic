pub mod inject;
pub mod launch;

use std::error::Error;
use std::{ffi::OsString, os::windows::ffi::OsStrExt, sync::mpsc::channel};

use dll_syringe::{process::OwnedProcess, Syringe};
use launch::attach_or_launch_injected;
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

    // TODO: Pull config from somewhere
    // TODO: Make following code use that config

    match attach_or_launch_injected() {
        Ok(payload) => {
            rx.recv().expect("Could not receive from channel.");
            println!("ctrl+c received...");
            // syringe.eject(payload);
        }
        Err(error) => {
            println!("Error in attach_or_launch_injected: {error}. Exiting now.");
            return;
        }
    }

    // Block until Ctrl+C

    println!("Exiting.");
}
