use once_cell::sync::Lazy;
use retour::GenericDetour;
use std::ffi::c_int;
use std::os::raw::c_void;
use std::ptr::null_mut;
use widestring::U16String;
use windows::core::w;
use windows::core::PCSTR;
use windows::core::PCWSTR;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::STD_ERROR_HANDLE;
use windows::Win32::System::Memory::VirtualProtect;
use windows::Win32::System::Memory::VirtualProtectEx;
use windows::Win32::System::Memory::PAGE_EXECUTE_READWRITE;
use windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS;
use windows::Win32::UI::WindowsAndMessaging::MessageBoxW;
use windows::Win32::UI::WindowsAndMessaging::MB_OK;

use windows::Win32::System::Console::{AllocConsole, SetStdHandle, STD_OUTPUT_HANDLE};

use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};

// Chorizite has this as
// private static int RecvFromImpl(nint s, byte* buf, int len, int flags, byte* from, int fromlen) {
// static_detour! {
//   static RecvFromImplHook: unsafe extern "system" fn(*mut c_void, *mut u8, c_int, c_int, *mut u8, c_int) -> c_int;
// }

type FnRecvFromImplHook =
    unsafe extern "system" fn(*mut c_void, *mut u8, c_int, c_int, *mut u8, c_int) -> c_int;

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

extern "system" fn my_recv_from_impl_hook(
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
    unsafe { HOOK_RECV.disable().unwrap() };
    let ret_val = unsafe { HOOK_RECV.call(s, buf, len, flags, from, fromlen) };
    unsafe { HOOK_RECV.enable().unwrap() };
    ret_val
}
static HOOK_RECV: Lazy<GenericDetour<FnRecvFromImplHook>> = Lazy::new(|| {
    unsafe { allocate_console().unwrap() };
    let address: i32 = 0x007935AC;

    let ori: FnRecvFromImplHook = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, my_recv_from_impl_hook).unwrap() };
});

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("attaching");
            unsafe {
                // let address: i32 = 0x007935AC;
                let address: *mut c_void = 0x007935AC as *mut c_void;
                let mut old_protect: u32 = 0;

                // pub unsafe fn VirtualProtectEx<P0>(
                //     hprocess: P0,
                //     lpaddress: *const c_void,
                //     dwsize: usize,
                //     flnewprotect: PAGE_PROTECTION_FLAGS,
                //     lpfloldprotect: *mut PAGE_PROTECTION_FLAGS,
                // ) -> Result<()>
                // where
                //     P0: Param<HANDLE>,

                // Chorizite code:
                // VirtualProtectEx(Process.GetCurrentProcess().Handle, address, (UIntPtr)4, 0x40, out int b);
                // *(int*)address = newValue;
                // VirtualProtectEx(Process.GetCurrentProcess().Handle, address, (UIntPtr)4, b, out b);

                println!("Trying VirtualProtectEx");
                let old_protect: *mut PAGE_PROTECTION_FLAGS = std::ptr::null_mut();
                match VirtualProtectEx(
                    _hinst,
                    address,
                    4 as usize,
                    PAGE_EXECUTE_READWRITE,
                    old_protect,
                ) {
                    Ok(_) => todo!(),
                    Err(error) => {
                        let lptext =
                            U16String::from_str(format!("Attach failed: {error:?}").as_str());
                        MessageBoxW(None, PCWSTR(lptext.as_ptr()), w!("Alembic"), MB_OK);
                    }
                }
                println!("Done VirtualProtectEx");

                match HOOK_RECV.enable() {
                    Ok(_) => {
                        println!("Hook enable success");
                    }
                    Err(error) => {
                        println!("Hook enable error: {error:?}")
                    }
                }
            }
        }
        DLL_PROCESS_DETACH => {
            println!("detaching");
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    };
    return BOOL::from(true);
}
