#![allow(
    dead_code,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types
)]
pub mod client;
pub mod hooks;

use std::{
    ffi::{c_void, CString},
    iter,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Once},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use hooks::{hook_OnChatCommand_Impl, hook_RecvFrom_New, hook_SendTo_New, hook_StartTooltip_Impl};
use tarpc::{client as tarcp_client, context, tokio_serde::formats::Json};

use tokio::runtime::Runtime;
pub(crate) use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{BOOL, HANDLE},
        Storage::FileSystem::{
            CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_WRITE, OPEN_EXISTING,
        },
        System::{
            Console::{
                AllocConsole, FreeConsole, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
            },
            LibraryLoader::{GetModuleHandleW, GetProcAddress},
            SystemServices::{
                DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
            },
        },
    },
};

unsafe fn allocate_console() -> windows::core::Result<()> {
    unsafe {
        // Allocate a new console
        AllocConsole()?;

        // Redirect stdout
        let stdout_handle = CreateFileA(
            PCSTR("CONOUT$\0".as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )?;

        SetStdHandle(STD_OUTPUT_HANDLE, stdout_handle)?;

        // Redirect stderr
        let stderr_handle = CreateFileA(
            PCSTR("CONOUT$\0".as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )?;

        SetStdHandle(STD_ERROR_HANDLE, stderr_handle)?;
    }

    println!("Console allocated and streams redirected successfully!");
    eprintln!("This is an error message test.");

    Ok(())
}

fn print_dbg_address(addr: isize, friendly_name: &str) {
    let q = region::query(addr as *const ()).unwrap();

    if q.is_executable() {
        println!("{friendly_name} is executable")
    } else {
        println!("{friendly_name} is NOT executable")
    }
}

fn get_module_symbol_address(module: &str, symbol: &str) -> Option<usize> {
    let module = module
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    let symbol = CString::new(symbol).unwrap();
    unsafe {
        let handle = GetModuleHandleW(PCWSTR(module.as_ptr() as _)).unwrap();
        match GetProcAddress(handle, PCSTR(symbol.as_ptr() as _)) {
            Some(func) => Some(func as usize),
            None => None,
        }
    }
}

fn print_vec(v: &Vec<u8>) {
    for (i, byte) in v.iter().enumerate() {
        print!("{byte:02X} ");

        if (i + 1) % 16 == 0 {
            println!();
        }
    }
}

struct AsyncRuntime {
    runtime: Arc<Runtime>,
}

impl AsyncRuntime {
    fn new() -> anyhow::Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        Ok(Self { runtime })
    }

    fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let rt = self.runtime.clone();
        rt.spawn(future);
    }

    fn shutdown(self) {
        // Drop the Arc, which will shut down the runtime if it's the last reference
        drop(self.runtime);
    }
}

#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
    async fn update_string(value: String) -> String;
    async fn append_log(value: String) -> String;
}

pub enum GuiMessage {
    Hello(String),
    UpdateString(String),
    AppendLog(String),
}

// Create and manage a Tokio async runtime in this thread
static mut rt: Option<Runtime> = None;
static rt_init: Once = Once::new();
fn ensure_runtime() -> &'static Runtime {
    unsafe {
        rt_init.call_once(|| {
            rt = Some(Runtime::new().expect("Failed to create Tokio runtime"));
        });
        rt.as_ref().unwrap()
    }
}

fn ensure_client() -> anyhow::Result<()> {
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

        println!("Saying hello on a loop");

        let mut max = 100;
        loop {
            // Say hello
            match client
                .hello(context::current(), "Hello from the client".to_string())
                .await
            {
                Ok(resp) => println!("resp is {resp}"),
                Err(error) => println!("error is {error:?}"),
            }

            let current_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            match client
                .append_log(context::current(), current_timestamp.to_string())
                .await
            {
                Ok(resp) => println!("resp is {resp}"),
                Err(error) => println!("error is {error:?}"),
            }

            thread::sleep(Duration::from_secs(3));

            max = max - 1;

            if max < 0 {
                break;
            }
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
