use std::ffi::c_void;

use crate::{define_hook, ensure_channel};
use libalembic::{acclient::PStringBase, msg::client_server::ClientServerMessage};

define_hook! {
    name: AddTextToScroll_char_ptr,
    address: 0x004882F0,
    convention: thiscall,
    args: (This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32),
    ret: i32,
    body: |_this, text, _a, _b, _c| {
        println!("Hook_AddTextToScroll_Impl_char_ptr");

        unsafe {
            if let Ok(p) = PStringBase::<i8>::new(text) {
                if let Ok(text_str) = p.to_string() {
                    let (tx, _rx) = ensure_channel();
                    let _ = tx
                        .try_lock()
                        .unwrap()
                        .send(ClientServerMessage::HandleAddTextToScroll(text_str));
                }
            }
        }
    }
}

define_hook! {
    name: AddTextToScroll_char_ptr_ptr,
    address: 0x004C3010,
    convention: thiscall,
    args: (This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32),
    ret: i32,
    body: |_this, text, _a, _b, _c| {
        println!("Hook_AddTextToScroll_Impl_char_ptr_ptr");

        unsafe {
            if let Ok(p) = PStringBase::<*const i8>::new(text) {
                if let Ok(text_str) = p.to_string() {
                    let (tx, _rx) = ensure_channel();
                    let _ = tx
                        .try_lock()
                        .unwrap()
                        .send(ClientServerMessage::HandleAddTextToScroll(text_str));
                }
            }
        }
    }
}

define_hook! {
    name: AddTextToScroll_ushort_ptr_ptr,
    address: 0x005649F0,
    convention: thiscall,
    args: (This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32),
    ret: i32,
    body: |_this, text, _a, _b, _c| {
        unsafe {
            if let Ok(p) = PStringBase::<*const u16>::new(text) {
                if let Ok(text_str) = p.to_string() {
                    let (tx, _rx) = ensure_channel();
                    let _ = tx
                        .try_lock()
                        .unwrap()
                        .send(ClientServerMessage::HandleAddTextToScroll(
                            text_str.trim().to_string(),
                        ));
                }
            }
        }
    }
}
