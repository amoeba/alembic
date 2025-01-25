use std::{ffi::c_void, panic, slice};

use crate::ensure_channel;
use libalembic::rpc::GuiMessage;
use once_cell::sync::Lazy;
use retour::GenericDetour;
use windows::{
    core::PCSTR,
    Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA},
};

type fn_WinSock_SendTo = extern "system" fn(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    to: *mut u8,
    tolen: *mut i32,
) -> i32;

// wsock32.dll::send_to
pub static Hook_Network_SendTo: Lazy<GenericDetour<fn_WinSock_SendTo>> = Lazy::new(|| {
    let library_handle = unsafe { LoadLibraryA(PCSTR(b"wsock32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"sendto\0".as_ptr() as _)) };
    let ori: fn_WinSock_SendTo = unsafe { std::mem::transmute(address) };

    return unsafe { GenericDetour::new(ori, Hook_Network_SendTo_Impl).unwrap() };
});

extern "system" fn Hook_Network_SendTo_Impl(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    to: *mut u8,
    tolen: *mut i32,
) -> i32 {
    let bytes_sent = Hook_Network_SendTo.call(s, buf, len, flags, to, tolen);

    if bytes_sent > 0 {
        let result = panic::catch_unwind(|| {
            let bytes = unsafe { slice::from_raw_parts(buf, bytes_sent as usize) };
            let bytes_vec = bytes.to_vec();

            let (tx, _rx) = ensure_channel();
            tx.try_lock()
                .unwrap()
                .send(GuiMessage::SendTo(bytes_vec.clone()))

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

pub static Hook_Network_RecvFrom: Lazy<GenericDetour<fn_WinSock_RecvFrom>> = Lazy::new(|| {
    let library_handle = unsafe { LoadLibraryA(PCSTR(b"wsock32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"recvfrom\0".as_ptr() as _)) };
    let ori: fn_WinSock_RecvFrom = unsafe { std::mem::transmute(address) };

    return unsafe { GenericDetour::new(ori, Hook_Network_RecvFrom_Impl).unwrap() };
});

extern "system" fn Hook_Network_RecvFrom_Impl(
    s: *mut c_void,
    buf: *mut u8,
    len: i32,
    flags: i32,
    from: *mut u8,
    fromlen: *mut i32,
) -> i32 {
    let bytes_read = Hook_Network_RecvFrom.call(s, buf, len, flags, from, fromlen);

    if bytes_read > 0 {
        let result = panic::catch_unwind(|| {
            // Convert the buffer to a slice
            let bytes = unsafe { slice::from_raw_parts(buf, bytes_read as usize) };
            let bytes_vec = bytes.to_vec();

            let (tx, _rx) = ensure_channel();
            tx.try_lock()
                .unwrap()
                .send(GuiMessage::RecvFrom(bytes_vec.clone()))
        });

        if let Err(e) = result {
            eprintln!("RecvFrom Error: {:?}", e);
        }
    }

    return bytes_read;
}
