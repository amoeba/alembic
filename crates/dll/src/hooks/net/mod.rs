use std::{ffi::c_void, slice};

use crate::{define_dll_hook, ensure_channel};
use libalembic::msg::client_server::ClientServerMessage;

define_dll_hook! {
    name: Network_SendTo,
    dll: b"wsock32.dll\0",
    proc: b"sendto\0",
    convention: system,
    args: (s: *mut c_void, buf: *mut u8, len: i32, flags: i32, to: *mut u8, tolen: *mut i32),
    ret: i32,
    body: |bytes_sent, _s, buf, _len, _flags, _to, _tolen| {
        if bytes_sent > 0 {
            let bytes = unsafe { slice::from_raw_parts(buf, bytes_sent as usize) };
            let bytes_vec = bytes.to_vec();

            let (tx, _rx) = ensure_channel();
            let _ = tx
                .try_lock()
                .unwrap()
                .send(ClientServerMessage::HandleSendTo(bytes_vec));
        }
    }
}

define_dll_hook! {
    name: Network_RecvFrom,
    dll: b"wsock32.dll\0",
    proc: b"recvfrom\0",
    convention: system,
    args: (s: *mut c_void, buf: *mut u8, len: i32, flags: i32, from: *mut u8, fromlen: *mut i32),
    ret: i32,
    body: |bytes_read, _s, buf, _len, _flags, _from, _fromlen| {
        if bytes_read > 0 {
            let bytes = unsafe { slice::from_raw_parts(buf, bytes_read as usize) };
            let bytes_vec = bytes.to_vec();

            let (tx, _rx) = ensure_channel();
            let _ = tx
                .try_lock()
                .unwrap()
                .send(ClientServerMessage::HandleRecvFrom(bytes_vec));
        }
    }
}
