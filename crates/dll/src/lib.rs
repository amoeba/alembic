#![cfg(all(target_os = "windows", target_env = "msvc"))]
#![allow(
    dead_code,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types
)]

mod hooks;

use std::{
    ffi::c_void,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Once},
    thread,
    time::Duration,
};

use libalembic::{
    msg::client_server::ClientServerMessage, rpc::WorldClient, win::allocate_console,
};
use tarpc::{client as tarcp_client, context, tokio_serde::formats::Json};
use tokio::{
    runtime::Runtime,
    sync::{
        mpsc::{self, error::TryRecvError},
        Mutex,
    },
};

pub(crate) use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::{
        Console::FreeConsole,
        SystemServices::{
            DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
        },
    },
};

// Create and manage a Tokio async runtime in this thread
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
fn ensure_client() -> anyhow::Result<()> {
    let (_tx, rx) = ensure_channel();

    let runtime = ensure_runtime();

    runtime.spawn(async {
        let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
        let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
        let client: WorldClient = WorldClient::new(
            tarcp_client::Config::default(),
            transport.await.expect("oops"),
        )
        .spawn();

        loop {
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
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(16));
        }
    });

    Ok(())
}

fn on_attach() -> Result<(), anyhow::Error> {
    if cfg!(debug_assertions) {
        unsafe {
            allocate_console()?;
        }
    }

    ensure_client()?;
    ensure_channel();

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
    unsafe {
        match FreeConsole() {
            Ok(_) => println!("Call to FreeConsole succeeded"),
            Err(error) => println!("Call to FreeConsole failed: {error:?}"),
        }
    }

    unsafe { crate::hooks::net::Hook_Network_RecvFrom.disable().unwrap() }
    unsafe { crate::hooks::net::Hook_Network_SendTo.disable().unwrap() }
    unsafe {
        crate::hooks::chat::Hook_AddTextToScroll_ushort_ptr_ptr
            .disable()
            .unwrap()
    }

    Ok(())
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("In DllMain, reason=DLL_PROCESS_ATTACH. initializing hooks now.");
            match on_attach() {
                Ok(_) => println!("on_attach succeeded"),
                Err(error) => println!("on_attach failed with error: {error}"),
            }
        }
        DLL_PROCESS_DETACH => {
            println!("In DllMain, reason=DLL_PROCESS_DETACH. removing hooks now.");
            match on_detach() {
                Ok(_) => println!("on_detach succeeded"),
                Err(error) => println!("on_detach failed with error: {error}"),
            }
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    };
    return BOOL::from(true);
}
