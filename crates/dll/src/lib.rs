#![cfg(all(target_os = "windows", target_env = "msvc"))]
#![allow(
    dead_code,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types
)]

mod hooks;
mod logging;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use std::{ffi::c_void, sync::Once, thread};

use dll_syringe::payload_procedure;
use libalembic::msg::client_server::ClientServerMessage;
use libalembic::rpc::WorldClient;
use logging::log_message;

use tarpc::tokio_util::sync::CancellationToken;
use tarpc::{client as tarcp_client, context, tokio_serde::formats::Json};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::{mpsc, Mutex};
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};

/// Runtime
static mut rt: Option<Runtime> = None;
static rt_init: Once = Once::new();
#[allow(static_mut_refs)]
fn ensure_runtime() -> &'static Runtime {
    unsafe {
        rt_init.call_once(|| {
            rt = Some(Runtime::new().expect("Failed to create Tokio runtime"));
        });
        rt.as_ref().unwrap()
    }
}

fn shutdown_runtime() -> anyhow::Result<()> {
    unsafe {
        rt.take()
            .unwrap()
            .shutdown_timeout(Duration::from_millis(100));
    }

    Ok(())
}

/// Hooks <-> TARPC Channel
static mut dll_tx: Option<Arc<Mutex<mpsc::UnboundedSender<ClientServerMessage>>>> = None;
static mut dll_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<ClientServerMessage>>>> = None;
static channel_init: Once = Once::new();
#[allow(static_mut_refs)]
pub fn ensure_channel() -> (
    &'static Arc<Mutex<tokio::sync::mpsc::UnboundedSender<ClientServerMessage>>>,
    &'static Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<ClientServerMessage>>>,
) {
    unsafe {
        channel_init.call_once(|| {
            let (tx, rx): (
                mpsc::UnboundedSender<ClientServerMessage>,
                mpsc::UnboundedReceiver<ClientServerMessage>,
            ) = mpsc::unbounded_channel();

            dll_tx = Some(Arc::new(Mutex::new(tx)));
            dll_rx = Some(Arc::new(Mutex::new(rx)));
        });

        (dll_tx.as_ref().unwrap(), dll_rx.as_ref().unwrap())
    }
}

/// Shutdown coordination
static mut SHUTDOWN_TOKEN: Option<CancellationToken> = None;
static token_init: Once = Once::new();

fn ensure_shutdown_token() -> &'static CancellationToken {
    unsafe {
        token_init.call_once(|| {
            SHUTDOWN_TOKEN = Some(CancellationToken::new());
        });
        SHUTDOWN_TOKEN.as_ref().unwrap()
    }
}

/// TARPC
fn ensure_client() -> anyhow::Result<()> {
    let (_tx, rx) = ensure_channel();
    let runtime = ensure_runtime();
    let token = ensure_shutdown_token().clone();

    runtime.spawn(async move {
        let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
        let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
        let client: WorldClient = WorldClient::new(
            tarcp_client::Config::default(),
            transport.await.expect("oops"),
        )
        .spawn();

        loop {
            // Check if shutdown was requested
            if token.is_cancelled() {
                break;
            }

            match rx.try_lock().unwrap().try_recv() {
                Ok(msg) => match msg {
                    ClientServerMessage::HandleSendTo(vec) => {
                        match client.handle_sendto(context::current(), vec).await {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                    }
                    ClientServerMessage::HandleRecvFrom(vec) => {
                        match client.handle_recvfrom(context::current(), vec).await {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                    }
                    ClientServerMessage::HandleAddTextToScroll(text) => {
                        match client.handle_chat(context::current(), text).await {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                    }
                    ClientServerMessage::AppendLog(_) => todo!(),
                    ClientServerMessage::ClientInjected() => todo!(),
                    ClientServerMessage::ClientEjected() => todo!(),
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(16));
        }

        // Cleanup connection
        drop(client);
    });

    Ok(())
}

fn shutdown_client() -> anyhow::Result<()> {
    if let Some(token) = unsafe { SHUTDOWN_TOKEN.as_ref() } {
        token.cancel();
    }

    Ok(())
}

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

fn attach_hooks() -> anyhow::Result<()> {
    unsafe { crate::hooks::net::Hook_Network_RecvFrom.enable()? }
    unsafe { crate::hooks::net::Hook_Network_SendTo.enable()? }
    unsafe { crate::hooks::chat::Hook_AddTextToScroll_ushort_ptr_ptr.enable()? }

    Ok(())
}

fn detach_hooks() -> anyhow::Result<()> {
    unsafe { crate::hooks::net::Hook_Network_RecvFrom.disable()? }
    unsafe { crate::hooks::net::Hook_Network_SendTo.disable()? }
    unsafe { crate::hooks::chat::Hook_AddTextToScroll_ushort_ptr_ptr.disable()? }

    Ok(())
}

payload_procedure! {
    pub fn dll_startup() {
        match on_attach() {
            Ok(_) => unsafe { log_message("on_attach call succeeded") },
            Err(_) => unsafe { log_message("on_attach call failed") },
        }

        unsafe { log_message("startup done"); }

    }
}

payload_procedure! {
    pub fn dll_shutdown() {
        match on_detach() {
            Ok(_) => unsafe { log_message("on_detach call succeeded") },
            Err(_) => unsafe { log_message("on_detach call failed") },
        }

        unsafe { log_message("shutdown done"); }
    }
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => BOOL::from(true),
        DLL_PROCESS_DETACH => BOOL::from(true),
        DLL_THREAD_ATTACH => BOOL::from(true),
        DLL_THREAD_DETACH => BOOL::from(true),
        _ => BOOL::from(true),
    }
}
