use std::{
    ffi::{c_void, CStr},
    panic, slice,
};

use once_cell::sync::Lazy;
use retour::GenericDetour;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HMODULE,
        System::LibraryLoader::{GetProcAddress, LoadLibraryA},
    },
};

use crate::{ensure_channel, util::print_vec};

// wsock32.dll::send_to
type fn_WinSock_SendTo = extern "system" fn(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    to: *mut u8,
    tolen: *mut i32,
) -> i32;

pub static hook_SendTo_New: Lazy<GenericDetour<fn_WinSock_SendTo>> = Lazy::new(|| {
    let library_handle = unsafe { LoadLibraryA(PCSTR(b"wsock32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"sendto\0".as_ptr() as _)) };

    println!("hook_SendTo address is {address:?}");

    let ori: fn_WinSock_SendTo = unsafe { std::mem::transmute(address) };

    return unsafe { GenericDetour::new(ori, our_WinSock_SendTo).unwrap() };
});

extern "system" fn our_WinSock_SendTo(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    to: *mut u8,
    tolen: *mut i32,
) -> i32 {
    let bytes_sent = hook_SendTo_New.call(s, buf, len, flags, to, tolen);

    if bytes_sent > 0 {
        let result = panic::catch_unwind(|| {
            let bytes = unsafe { slice::from_raw_parts(buf, bytes_sent as usize) };
            let bytes_vec = bytes.to_vec();

            print_vec(&bytes_vec);

            let (tx, _rx) = ensure_channel();
            tx.try_lock()
                .unwrap()
                .send(crate::rpc::GuiMessage::SendTo(bytes_vec.clone()))

            // TODO: Envision this API
            // Handle the received packet data
            // standalone_loader::backend::handle_s2c_packet_data(bytes_vec);
        });

        if let Err(e) = result {
            eprintln!("SendTo Error: {:?}", e);
        }
    }

    return bytes_sent;
}

// wsock32.dll::recv_from
type fn_WinSock_RecvFrom = extern "system" fn(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    from: *mut u8,
    fromlen: *mut i32,
) -> i32;

pub static hook_RecvFrom_New: Lazy<GenericDetour<fn_WinSock_RecvFrom>> = Lazy::new(|| {
    let library_handle = unsafe { LoadLibraryA(PCSTR(b"wsock32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"recvfrom\0".as_ptr() as _)) };

    println!("hook_RecvFrom address is {address:?}");

    let ori: fn_WinSock_RecvFrom = unsafe { std::mem::transmute(address) };

    return unsafe { GenericDetour::new(ori, our_WinSock_RecvFrom).unwrap() };
});

extern "system" fn our_WinSock_RecvFrom(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    from: *mut u8,
    fromlen: *mut i32,
) -> i32 {
    let bytes_read = hook_RecvFrom_New.call(s, buf, len, flags, from, fromlen);

    if bytes_read > 0 {
        let result = panic::catch_unwind(|| {
            // Convert the buffer to a slice
            let bytes = unsafe { slice::from_raw_parts(buf, bytes_read as usize) };
            let bytes_vec = bytes.to_vec();

            print_vec(&bytes_vec);

            // Handle the received packet data
            // standalone_loader::backend::handle_s2c_packet_data(bytes_vec);
        });

        if let Err(e) = result {
            eprintln!("RecvFrom Error: {:?}", e);
        }
    }

    return bytes_read;
}

// LoadLibraryA
type fn_LoadLibraryA = extern "system" fn(PCSTR) -> HMODULE;

static hook_LoadLibraryA: Lazy<GenericDetour<fn_LoadLibraryA>> = Lazy::new(|| {
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

pub static hook_StartTooltip_Impl: Lazy<GenericDetour<fn_StartTooltip_Impl>> = Lazy::new(|| {
    println!("hook_StartTooltip_Impl");
    let address = 0x0045DF70 as isize;
    let ori: fn_StartTooltip_Impl = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, our_StartTooltip_Impl).unwrap() };
});

// OnChatCommand
//
// Address: 0x005821A0
// Note: Only this breakpoint gets hit
//----- (00581320) --------------------------------------------------------
// bool __thiscall ClientCommunicationSystem::OnChatCommand(ClientCommunicationSystem *this, const PStringBase<unsigned short> *i_strLine, unsigned int i_idCommandSource)
type fn_OnChatCommand_Impl =
    extern "thiscall" fn(This: *mut c_void, text: *mut c_void, chatWindowId: isize) -> isize;

extern "thiscall" fn our_OnChatCommand_Impl(
    This: *mut c_void,
    text: *mut c_void,
    chatWindowId: isize,
) -> isize {
    println!("our_OnChatCommand_Impl");

    let pstring = unsafe {
        crate::client::PStringBase::from_mut_ptr(text as *mut crate::client::PSRefBuffer)
    };
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
