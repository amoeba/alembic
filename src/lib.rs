use retour::static_detour;
use std::error::Error;
use std::ffi::c_int;
use std::mem;
use std::os::raw::c_void;
use windows::core::PCSTR;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::STD_ERROR_HANDLE;

use windows::Win32::System::Console::{AllocConsole, SetStdHandle, STD_OUTPUT_HANDLE};

use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};

// Chorizite has this as
// private static int RecvFromImpl(nint s, byte* buf, int len, int flags, byte* from, int fromlen) {
static_detour! {
  static RecvFromImplHook: unsafe extern "system" fn(*mut c_void, *mut u8, c_int, c_int, *mut u8, c_int) -> c_int;
}

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

pub fn is_executable_address(address: *const ()) -> bool {
    region::query(address as *const _)
        .unwrap()
        .protection()
        .contains(region::Protection::EXECUTE)
}

unsafe fn main() -> Result<(), Box<dyn Error>> {
    allocate_console()?;

    let address: i32 = 0x007935AC;

    if is_executable_address(address as *const ()) {
        println!("hook target address 0x{address:x} IS executable");
    } else {
        println!("hook target address 0x{address:x} NOT executable");
    }

    RecvFromImplHook
        .initialize(mem::transmute(address), my_recv_from_impl_hook)?
        .enable()?;
    Ok(())
}

fn my_recv_from_impl_hook(
    s: *mut c_void,
    buf: *mut u8,
    len: c_int,
    flags: c_int,
    from: *mut u8,
    fromlen: c_int,
) -> c_int {
    println!("my_recv_from_impl_hook called with args:");
    println!("  Socket: {:?}", s);
    println!("  Buffer length: {}", len);
    println!("  Flags: {}", flags);
    println!("  From length: {}", fromlen);

    // Call the original function
    let result = unsafe { RecvFromImplHook.call(s, buf, len, flags, from, fromlen) };

    // If the call was successful, print the received data
    // if result > 0 {
    //     let received_data = std::slice::from_raw_parts(buf, result as usize);
    //     println!("Received data: {:?}", received_data);
    // }

    println!("RecvFromImpl returned: {}", result);

    result
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("In DllMain, attaching because reason is DLL_PROCESS_ATTACH");

            unsafe {
                match main() {
                    Ok(_) => {
                        println!("main ran successfully")
                    }
                    Err(error) => {
                        eprintln!("main ran with error {error:?}");
                    }
                }
            }
        }
        DLL_PROCESS_DETACH => {
            println!("In Dllmain, detaching because reason is DLL_PROCESS_DETACH");
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    };
    return BOOL::from(true);
}
