#![cfg(target_os = "windows")]
#![allow(
    dead_code,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types
)]

#[macro_use]
mod hooks;
mod logging;

use std::time::Duration;
use std::{ffi::c_void, thread};

use channel::ensure_channel;
use client::{ensure_client, shutdown_client};
use dll_syringe::payload_procedure;
use logging::log_message;

use runtime::{ensure_runtime, shutdown_runtime};
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};

mod channel;
mod client;
mod runtime;

// Register all hooks - generates attach_hooks() and detach_hooks()
register_hooks!(
    hooks::net::Hook_Network_RecvFrom,
    hooks::net::Hook_Network_SendTo,
    hooks::chat::Hook_AddTextToScroll_ushort_ptr_ptr,
);

fn on_attach() -> Result<(), anyhow::Error> {
    attach_hooks()?;

    ensure_runtime();
    ensure_channel();
    ensure_client()?;

    Ok(())
}

fn on_detach() -> anyhow::Result<()> {
    detach_hooks()?;
    shutdown_client()?;

    // Give tasks time to cleanup
    thread::sleep(Duration::from_millis(100));
    shutdown_runtime()?;

    Ok(())
}

payload_procedure! {
    pub fn dll_startup() {
        match on_attach() {
            Ok(_) => unsafe { log_message("Alembic Startup succeeded.") },
            Err(_) => unsafe { log_message("Alembic Startup failed.") },
        }
    }
}

payload_procedure! {
    pub fn dll_shutdown() {
        match on_detach() {
            Ok(_) => unsafe { log_message("Alembic Shutdown succeeded.") },
            Err(_) => unsafe { log_message("Alembic Shutdown failed.") },
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => BOOL::from(true),
        DLL_PROCESS_DETACH => BOOL::from(true),
        DLL_THREAD_ATTACH => BOOL::from(true),
        DLL_THREAD_DETACH => BOOL::from(true),
        _ => BOOL::from(true),
    }
}
