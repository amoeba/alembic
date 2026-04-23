// Enforce that the DLL can ONLY be built for 32-bit Windows (msvc or gnu)
#![cfg(all(target_arch = "x86", target_os = "windows"))]
#![allow(
    dead_code,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types
)]

// Additional compile-time check with helpful error message
#[cfg(not(all(target_arch = "x86", target_os = "windows")))]
compile_error!(
    "dll can only be built for 32-bit Windows targets (i686-pc-windows-msvc or i686-pc-windows-gnu)"
);

mod channel;
mod client;
mod hooks;
mod logging;
mod runtime;

use std::{ffi::c_void, thread, time::Duration};

use channel::ensure_channel;
use client::{ensure_client, shutdown_client};
use logging::log_message;
use runtime::shutdown_runtime;

pub(crate) use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::{
        Console::FreeConsole,
        SystemServices::{
            DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
        },
    },
};

fn on_attach() -> Result<(), anyhow::Error> {
    ensure_channel();
    ensure_client()?;

    unsafe { crate::hooks::net::Hook_Network_RecvFrom.enable().unwrap() }
    unsafe { crate::hooks::net::Hook_Network_SendTo.enable().unwrap() }
    unsafe {
        crate::hooks::chat::Hook_AddTextToScroll_ushort_ptr_ptr
            .enable()
            .unwrap()
    }

    Ok(())
}

fn on_detach() -> anyhow::Result<()> {
    unsafe { crate::hooks::net::Hook_Network_RecvFrom.disable().unwrap() }
    unsafe { crate::hooks::net::Hook_Network_SendTo.disable().unwrap() }
    unsafe {
        crate::hooks::chat::Hook_AddTextToScroll_ushort_ptr_ptr
            .disable()
            .unwrap()
    }

    shutdown_client()?;

    // Give async tasks time to clean up
    thread::sleep(Duration::from_millis(100));
    shutdown_runtime()?;

    unsafe {
        match FreeConsole() {
            Ok(_) => log_message("Call to FreeConsole succeeded"),
            Err(error) => log_message(&format!("Call to FreeConsole failed: {error:?}")),
        }
    }

    Ok(())
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    unsafe {
        match reason {
            DLL_PROCESS_ATTACH => {
                log_message("DllMain: DLL_PROCESS_ATTACH, initializing hooks");
                match on_attach() {
                    Ok(_) => log_message("on_attach succeeded"),
                    Err(error) => log_message(&format!("on_attach failed: {error}")),
                }
            }
            DLL_PROCESS_DETACH => {
                log_message("DllMain: DLL_PROCESS_DETACH, cleaning up");
                match on_detach() {
                    Ok(_) => log_message("on_detach succeeded"),
                    Err(error) => log_message(&format!("on_detach failed: {error}")),
                }
            }
            DLL_THREAD_ATTACH => {}
            DLL_THREAD_DETACH => {}
            _ => {}
        };
        BOOL::from(true)
    }
}
