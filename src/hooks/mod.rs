use std::ffi::{c_void, CStr};

use client::{PSRefBuffer, PStringBase};
use once_cell::sync::Lazy;
use retour::GenericDetour;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HMODULE,
        System::LibraryLoader::{GetProcAddress, LoadLibraryA},
    },
};

use crate::{allocate_console, client, print_dbg_address};

// LoadLibraryA
type fn_LoadLibraryA = extern "system" fn(PCSTR) -> HMODULE;

static hook_LoadLibraryA: Lazy<GenericDetour<fn_LoadLibraryA>> = Lazy::new(|| {
    unsafe { allocate_console().unwrap() };

    let library_handle = unsafe { LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"LoadLibraryA\0".as_ptr() as _)) };

    let ori: fn_LoadLibraryA = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_LoadLibraryA).unwrap() };
});

extern "system" fn our_LoadLibraryA(lpFileName: PCSTR) -> HMODULE {
    let _file_name = unsafe { CStr::from_ptr(lpFileName.as_ptr() as _) };
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

// RecvFrom
// Address: 0x007935AC
type fn_RecvFromImpl = extern "stdcall" fn(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    from: *mut u8,
    fromlen: i32,
) -> i32;

extern "stdcall" fn our_RecvFromImpl(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    from: *mut u8,
    fromlen: i32,
) -> i32 {
    println!("inside our_RecvFromImpl");
    unsafe { hook_RecvFrom.disable().unwrap() };
    let ret_val = hook_RecvFrom.call(s, buf, len, flags, from, fromlen);
    println!("done calling original fn of our_RecvFromImpl");
    unsafe { hook_RecvFrom.enable().unwrap() };

    return ret_val;
}
pub static hook_RecvFrom: Lazy<GenericDetour<fn_RecvFromImpl>> = Lazy::new(|| {
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
// ResetTooltip
// Address: 0x0045C440
type fn_ResetTooltip_Impl = extern "thiscall" fn(This: *mut c_void) -> *mut c_void;

extern "thiscall" fn our_ResetTooltip_Impl(This: *mut c_void) -> *mut c_void {
    println!("our_ResetTooltip_Impl");
    let ret_val = hook_ResetTooltip_Impl.call(This);

    ret_val
}

static hook_ResetTooltip_Impl: Lazy<GenericDetour<fn_ResetTooltip_Impl>> = Lazy::new(|| {
    println!("hook_ResetTooltip_Impl");
    let address = 0x0045C440 as isize;
    let ori: fn_ResetTooltip_Impl = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_ResetTooltip_Impl).unwrap() };
});

// StartTooltip
// Address: 0x0045DF70
type fn_StartTooltip_Impl = extern "thiscall" fn(
    This: *mut c_void,
    strInfo: *mut c_void,
    el: *mut c_void,
    a: isize,
    b: u32,
    c: isize,
) -> i32;
extern "thiscall" fn our_StartTooltip_Impl(
    This: *mut c_void,
    strInfo: *mut c_void,
    el: *mut c_void,
    a: isize,
    b: u32,
    c: isize,
) -> i32 {
    println!("our_StartTooltip_Impl");
    println!("{strInfo:?}");

    let ret_val = hook_StartTooltip_Impl.call(This, strInfo, el, a, b, c);

    ret_val
}
static hook_StartTooltip_Impl: Lazy<GenericDetour<fn_StartTooltip_Impl>> = Lazy::new(|| {
    println!("hook_StartTooltip_Impl");
    let address = 0x0045DF70 as isize;
    let ori: fn_StartTooltip_Impl = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_StartTooltip_Impl).unwrap() };
});

// OnChatCommand
//
// Address: 0x005821A0
type fn_OnChatCommand_Impl =
    extern "thiscall" fn(This: *mut c_void, text: *mut c_void, chatWindowId: isize) -> isize;

extern "thiscall" fn our_OnChatCommand_Impl(
    This: *mut c_void,
    text: *mut c_void,
    chatWindowId: isize,
) -> isize {
    println!("fn_OnChatCommand_Impl: text pointer is {text:?}");

    let pstring = unsafe { PStringBase::from_mut_ptr(text as *mut PSRefBuffer) };
    println!("pstring to_string is {pstring}");

    let ret_val = hook_OnChatCommand_Impl.call(This, text, chatWindowId);

    ret_val
}

pub static hook_OnChatCommand_Impl: Lazy<GenericDetour<fn_OnChatCommand_Impl>> = Lazy::new(|| {
    println!("hook_OnChatCommand_Impl");
    let address = 0x005821A0 as isize;
    let ori: fn_OnChatCommand_Impl = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_OnChatCommand_Impl).unwrap() };
});
