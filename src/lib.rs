#![cfg(windows)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

use std::{
    ffi::{c_void, CStr},
    thread,
    time::Duration,
};

use once_cell::sync::Lazy;
use retour::GenericDetour;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{BOOL, HANDLE, HMODULE},
        Storage::FileSystem::{
            CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_WRITE, OPEN_EXISTING,
        },
        System::{
            Console::{AllocConsole, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE},
            LibraryLoader::{GetProcAddress, LoadLibraryA},
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

type fn_LoadLibraryA = extern "system" fn(PCSTR) -> HMODULE;
type fn_RecvFromImpl = extern "stdcall" fn(
    s: isize,
    buf: *mut u8,
    len: isize,
    flags: isize,
    from: *mut u8,
    fromlen: isize,
) -> isize;

fn print_dbg_address(addr: isize, friendly_name: &str) {
    let q = region::query(addr as *const ()).unwrap();

    if q.is_executable() {
        println!("{friendly_name} is executable")
    } else {
        println!("{friendly_name} is NOT executable")
    }
}

static hook_LoadLibraryA: Lazy<GenericDetour<fn_LoadLibraryA>> = Lazy::new(|| {
    unsafe { allocate_console().unwrap() };

    let library_handle = unsafe { LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"LoadLibraryA\0".as_ptr() as _)) };

    print_dbg_address(address.expect("msg") as isize, "LoadLibraryA");
    print_dbg_address(0x007935A4 as isize, "SendToImpl");
    print_dbg_address(0x007935AC as isize, "RecvFromImpl");
    print_dbg_address(0x00675920 as isize, "CLBlockAllocator_OpenDataFile_Impl");

    println!("About to try reprotecting recv");
    let protect;
    unsafe {
        protect = region::protect_with_handle(
            0x00793000 as *const (),
            0x00078000,
            region::Protection::READ_WRITE_EXECUTE,
        );
    }

    match protect {
        Ok(_) => {
            println!("Reprotect was successfull.");
        }
        Err(error) => {
            println!("Reprotect failed with error: {error:?}")
        }
    }

    print_dbg_address(address.expect("msg") as isize, "LoadLibraryA");
    print_dbg_address(0x007935A4 as isize, "SendToImpl");
    print_dbg_address(0x007935AC as isize, "RecvFromImpl");
    print_dbg_address(0x00675920 as isize, "CLBlockAllocator_OpenDataFile_Impl");

    println!("Sleeping...");
    thread::sleep(Duration::from_millis(3000));

    println!("About to try hooking sendto...");

    let hook_result;
    unsafe {
        hook_result = hook_RecvFrom.enable();
    }

    match hook_result {
        Ok(_) => println!("Hook success"),
        Err(error) => println!("Hook failed: {error:?}"),
    }

    println!("...Done hooking sendto");

    println!("Sleeping 3s...");
    thread::sleep(Duration::from_millis(3000));

    let ori: fn_LoadLibraryA = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_LoadLibraryA).unwrap() };
});

static hook_RecvFrom: Lazy<GenericDetour<fn_RecvFromImpl>> = Lazy::new(|| {
    // unsafe { allocate_console().unwrap() };

    let address = 0x007935AC as isize;

    print_dbg_address(0x007935AC as isize, "RecvFromImpl");
    println!("About to try reprotecting recv");
    let protect;
    unsafe {
        protect = region::protect_with_handle(
            0x00793000 as *const (),
            0x00078000,
            region::Protection::READ_WRITE_EXECUTE,
        );
    }

    match protect {
        Ok(_) => {
            println!("Reprotect was successfull.");
        }
        Err(error) => {
            println!("Reprotect failed with error: {error:?}")
        }
    }

    print_dbg_address(0x007935AC as isize, "RecvFromImpl");

    let ori: fn_RecvFromImpl = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_RecvFromImpl).unwrap() };
});

extern "system" fn our_LoadLibraryA(lpFileName: PCSTR) -> HMODULE {
    let file_name = unsafe { CStr::from_ptr(lpFileName.as_ptr() as _) };
    // println!("our_LoadLibraryA lpFileName = {:?}", file_name);
    unsafe { hook_LoadLibraryA.disable().unwrap() };
    let ret_val = hook_LoadLibraryA.call(lpFileName);
    // println!(
    //     "our_LoadLibraryA lpFileName = {:?} ret_val = {:?}",
    //     file_name, ret_val.0
    // );
    unsafe { hook_LoadLibraryA.enable().unwrap() };
    return ret_val;
}

// [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvStdcall) })]
// private static int RecvFromImpl(nint s, byte* buf, int len, int flags, byte* from, int fromlen) {
extern "stdcall" fn our_RecvFromImpl(
    s: isize,
    buf: *mut u8,
    len: isize,
    flags: isize,
    from: *mut u8,
    fromlen: isize,
) -> isize {
    println!("inside our_RecvFromImpl");
    // unsafe { hook_RecvFrom.disable().unwrap() };
    let ret_val = hook_RecvFrom.call(s, buf, len, flags, from, fromlen);
    println!("done calling original fn of our_RecvFromImpl");
    // unsafe { hook_RecvFrom.enable().unwrap() };

    return ret_val;
}

// 0x004122A0
// [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvStdcall) })]
// private static byte Client_IsAlreadyRunning_Impl(IntPtr client) {
//     return 0;
// }
type fn_Client_IsAlreadyRunning_Impl = extern "system" fn(_client: *const c_void) -> u8;
extern "system" fn our_Client_IsAlreadyRunning_Impl(_client: *const c_void) -> u8 {
    println!("our_Client_IsAlreadyRunning_Impl");
    return 0;
}
static hook_Client_IsAlreadyRunning_Impl: Lazy<GenericDetour<fn_Client_IsAlreadyRunning_Impl>> =
    Lazy::new(|| {
        unsafe {
            allocate_console().unwrap();
        }
        println!("hook_Client_IsAlreadyRunning_Impl");
        let address = 0x004122A0 as isize;
        let ori: fn_Client_IsAlreadyRunning_Impl = unsafe { std::mem::transmute(address) };
        return unsafe { GenericDetour::new(ori, our_Client_IsAlreadyRunning_Impl).unwrap() };
    });

// cdecl and static:  .text:0x0045F900 ; void __cdecl UIElementManager::CreateUIElementManager() .text:0045F900 ?CreateUIElementManager@UIElementManager@@SAXXZ
type fn_CreateUIElementManager_Impl = extern "system" fn() -> *mut c_void;
extern "system" fn our_CreateUIElementManager_Impl() -> *mut c_void {
    println!("our_CreateUIElementManager_Impl");
    0 as *mut c_void
}
static hook_CreateUIElementManager_Impl: Lazy<GenericDetour<fn_CreateUIElementManager_Impl>> =
    Lazy::new(|| {
        unsafe {
            allocate_console().unwrap();
        }
        println!("hook_CreateUIElementManager_Impl");
        let address = 0x0045F900 as isize;
        let ori: fn_CreateUIElementManager_Impl = unsafe { std::mem::transmute(address) };
        return unsafe { GenericDetour::new(ori, our_CreateUIElementManager_Impl).unwrap() };
    });

// 0x0045C440
//
// [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvMemberFunction) })]
// private static void UIElementManager_ResetTooltip_Impl(UIElementManager* This) {
//     Hook_UIElementManager_ResetTooltip!.OriginalFunction(This);
// }
type fn_ResetTooltip_Impl = extern "thiscall" fn(This: *mut c_void) -> *mut c_void;
extern "thiscall" fn our_ResetTooltip_Impl(This: *mut c_void) -> *mut c_void {
    println!("our_ResetTooltip_Impl");
    let ret_val = hook_ResetTooltip_Impl.call(This);

    ret_val
}
static hook_ResetTooltip_Impl: Lazy<GenericDetour<fn_ResetTooltip_Impl>> = Lazy::new(|| {
    unsafe {
        allocate_console().unwrap();
    }
    println!("hook_ResetTooltip_Impl");
    let address = 0x0045C440 as isize;
    let ori: fn_ResetTooltip_Impl = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_ResetTooltip_Impl).unwrap() };
});

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("attaching");
            unsafe {
                hook_ResetTooltip_Impl.enable().unwrap();
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
