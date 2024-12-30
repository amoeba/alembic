use retour::static_detour;
use std::error::Error;
use std::ffi::c_int;
use std::mem;
use std::os::raw::c_void;
use widestring::{U16CString, U16String};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};
use windows::Win32::UI::WindowsAndMessaging::*;

// Chorizite has this as
// private static int RecvFromImpl(nint s, byte* buf, int len, int flags, byte* from, int fromlen) {
static_detour! {
  static RecvFromImplHook: unsafe extern "system" fn(*mut c_void, *mut u8, c_int, c_int, *mut u8, c_int) -> c_int;
}

unsafe fn main() -> Result<(), Box<dyn Error>> {
    let address = 0x007935AC;

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
    println!("RecvFromImpl called with:");
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
            println!("attaching");
            MessageBoxW(None, w!("ATTACH"), w!("Alembic"), MB_OK);

            unsafe {
                match main() {
                    Ok(_) => {
                        MessageBoxW(
                            None,
                            w!("In Attach, main() ran with success"),
                            w!("Alembic"),
                            MB_OK,
                        );
                    }
                    Err(error) => {
                        let lptext =
                            U16String::from_str(format!("Attach failed: {error:?}").as_str());
                        MessageBoxW(None, PCWSTR(lptext.as_ptr()), w!("Alembic"), MB_OK);
                    }
                }
            }
        }
        DLL_PROCESS_DETACH => {
            println!("detaching");
            MessageBoxW(None, w!("DETACH"), w!("Alembic"), MB_OK);
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    };
    return BOOL::from(true);
}
