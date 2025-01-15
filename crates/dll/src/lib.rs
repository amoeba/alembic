#![cfg(all(target_arch = "x86", target_os = "windows", target_env = "msvc"))]
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

use hooks::{hook_OnChatCommand_Impl, hook_RecvFrom_New, hook_SendTo_New, hook_StartTooltip_Impl};
use libalembic::{
    rpc::{GuiMessage, WorldClient},
    win::allocate_console,
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

static mut dll_tx: Option<Arc<Mutex<mpsc::UnboundedSender<GuiMessage>>>> = None;
static mut dll_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<GuiMessage>>>> = None;
static channel_init: Once = Once::new();
#[allow(static_mut_refs)]
pub fn ensure_channel() -> (
    &'static Arc<Mutex<tokio::sync::mpsc::UnboundedSender<GuiMessage>>>,
    &'static Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<GuiMessage>>>,
) {
    unsafe {
        channel_init.call_once(|| {
            let (tx, rx): (
                mpsc::UnboundedSender<GuiMessage>,
                mpsc::UnboundedReceiver<GuiMessage>,
            ) = mpsc::unbounded_channel();

            dll_tx = Some(Arc::new(Mutex::new(tx)));
            dll_rx = Some(Arc::new(Mutex::new(rx)));
        });

        (dll_tx.as_ref().unwrap(), dll_rx.as_ref().unwrap())
    }
}
fn ensure_client() -> anyhow::Result<()> {
    let (_tx, rx) = ensure_channel();

    println!("inside client_wip, start");

    let runtime = ensure_runtime();

    runtime.spawn(async {
        println!("Hello from inside async runtime");

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
                    GuiMessage::SendTo(value) => {
                        match client.handle_sendto(context::current(), value).await {
                            Ok(resp) => println!("resp is {resp}"),
                            Err(error) => println!("error is {error:?}"),
                        }
                    }
                    GuiMessage::Hello(_) => todo!(),
                    GuiMessage::UpdateString(_) => todo!(),
                    GuiMessage::AppendLog(_) => todo!(),
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    println!("Channel disconnected");
                    break;
                }
            }

            thread::sleep(Duration::from_millis(16));
        }
    });

    println!("inside, client_wip end");

    Ok(())
}

fn on_attach() -> Result<(), anyhow::Error> {
    unsafe {
        match allocate_console() {
            Ok(_) => println!("Call to FreeConsole succeeded"),
            Err(error) => println!("Call to FreeConsole failed: {error:?}"),
        }
    }

    match ensure_client() {
        Ok(_) => println!("Client started without error"),
        Err(error) => println!("Client started with error: {error}"),
    }

    ensure_channel();

    println!("in init_hooks, initializing hooks now");

    unsafe {
        hook_StartTooltip_Impl.enable().unwrap();
    }

    unsafe {
        hook_OnChatCommand_Impl.enable().unwrap();
    }

    unsafe { hook_RecvFrom_New.enable().unwrap() }
    unsafe { hook_SendTo_New.enable().unwrap() }

    // this doesn't work well, don't do this
    //unsafe { init_message_box_detour().unwrap() };

    Ok(())
}

fn on_detach() -> Result<(), anyhow::Error> {
    unsafe {
        match FreeConsole() {
            Ok(_) => println!("Call to FreeConsole succeeded"),
            Err(error) => println!("Call to FreeConsole failed: {error:?}"),
        }
    }

    unsafe {
        hook_StartTooltip_Impl.disable().unwrap();
    }

    unsafe {
        hook_OnChatCommand_Impl.disable().unwrap();
    }

    // unsafe {
    //     hook_RecvFrom.disable().unwrap();
    // }

    Ok(())
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("In DllMain, reason=DLL_PROCESS_ATTACH. initializing hooks now.");
            let _ = on_attach();
        }
        DLL_PROCESS_DETACH => {
            println!("In DllMain, reason=DLL_PROCESS_DETACH. removing hooks now.");
            let _ = on_detach();
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    };
    return BOOL::from(true);
}
