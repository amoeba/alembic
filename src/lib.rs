#![cfg(windows)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

use std::{ffi::CStr, os::raw::c_void};

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

    let ori: fn_LoadLibraryA = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_LoadLibraryA).unwrap() };
});

extern "system" fn our_LoadLibraryA(lpFileName: PCSTR) -> HMODULE {
    let file_name = unsafe { CStr::from_ptr(lpFileName.as_ptr() as _) };
    println!("our_LoadLibraryA lpFileName = {:?}", file_name);
    unsafe { hook_LoadLibraryA.disable().unwrap() };
    let ret_val = hook_LoadLibraryA.call(lpFileName);
    println!(
        "our_LoadLibraryA lpFileName = {:?} ret_val = {:?}",
        file_name, ret_val.0
    );
    unsafe { hook_LoadLibraryA.enable().unwrap() };
    return ret_val;
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("attaching");
            unsafe {
                hook_LoadLibraryA.enable().unwrap();
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
