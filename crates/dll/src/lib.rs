#![cfg(all(target_os = "windows", target_env = "msvc"))]
#![allow(
    dead_code,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types
)]

mod dll_state;
mod hooks;

use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    ffi::c_void,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Once,
    },
    thread,
    time::Duration,
};

use libalembic::{
    msg::client_server::ClientServerMessage,
    rpc::WorldClient,
    win::{allocate_console, deallocate_console},
};
use tarpc::{client as tarcp_client, context, tokio_serde::formats::Json};
use tokio::{
    runtime::Runtime,
    sync::{
        mpsc::{self, error::TryRecvError},
        Mutex,
    },
};

use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
};

static INIT_RESULT: AtomicBool = AtomicBool::new(false);
static mut INIT_ERROR: Option<String> = None;

static mut DLL_STATE: Option<DllState> = None;
static DLL_STATE_INIT: Once = Once::new();

fn shutdown_client() {
    // WIP
    shutdown_runtime();
}

#[allow(static_mut_refs)]
fn shutdown_runtime() {
    println!("shutdown_client called");

    if !SHUTDOWN_INITIATED.swap(true, Ordering::SeqCst) {
        unsafe {
            if let Some(runtime) = rt.take() {
                // Shutdown the runtime
                println!("shutting down runtime ");

                runtime.shutdown_background();
                println!("done shutting down runtime ");

                // Or use runtime.shutdown_timeout(Duration::from_secs(10))
                // if you want to wait for tasks to complete
            }
        }
    }
}

fn on_attach() -> Result<(), anyhow::Error> {
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
    let dll_state = ensure_dll_state();
    dll_state.shutdown();

    unsafe {
        match FreeConsole() {
            Ok(_) => println!("Call to FreeConsole succeeded"),
            Err(error) => println!("Call to FreeConsole failed: {error:?}"),
        }
    }

    unsafe {
        crate::hooks::chat::Hook_AddTextToScroll_ushort_ptr_ptr
            .disable()
            .unwrap()
    }
    unsafe { crate::hooks::net::Hook_Network_SendTo.disable().unwrap() }
    unsafe { crate::hooks::net::Hook_Network_RecvFrom.disable().unwrap() }

    // WIP
    shutdown_client();

    unsafe { deallocate_console() }?;

    Ok(())
}

#[no_mangle]
unsafe extern "system" fn DllMain(hinst: HANDLE, reason: u32, reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            log_event("DLL_PROCESS_ATTACH");

            // Spawn a thread to do initialization to avoid loader lock issues
            if reserved.is_null() {
                // Static load
                thread::spawn(|| {
                    INIT_RESULT.store(initialize_dll(), Ordering::SeqCst);
                });
            } else {
                // Dynamic load
                INIT_RESULT.store(initialize_dll(), Ordering::SeqCst);
            }

            BOOL::from(true)
        }
        DLL_PROCESS_DETACH => {
            log_event("DLL_PROCESS_DETACH");

            // Only clean up if we successfully initialized
            if INIT_RESULT.load(Ordering::SeqCst) {
                if let Err(e) = on_detach() {
                    log_event(&format!("DLL cleanup failed: {}", e));
                    return BOOL::from(false);
                }
            }

            BOOL::from(true)
        }
        DLL_THREAD_ATTACH => {
            log_event("DLL_THREAD_ATTACH");
            BOOL::from(true)
        }
        DLL_THREAD_DETACH => {
            log_event("DLL_THREAD_DETACH");
            BOOL::from(true)
        }
        _ => BOOL::from(true),
    }
}
