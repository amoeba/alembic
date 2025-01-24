use std::ffi::c_void;

use libalembic::acclient::{
    string_from_char_ptr, string_from_char_ptr_ptr, string_from_ushort_ptr_ptr,
};
use once_cell::sync::Lazy;
use retour::GenericDetour;

// ushort_ptr_ptr
type fn_AddTextToScroll_Impl_ushort_ptr_ptr =
    extern "thiscall" fn(This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32) -> i32;

extern "thiscall" fn Hook_AddTextToScroll_Impl_ushort_ptr_ptr(
    This: *mut c_void,
    text: *mut c_void,
    a: u32,
    b: u8,
    c: u32,
) -> i32 {
    println!("Hook_AddTextToScroll_Impl_ushort_ptr_ptr");

    unsafe {
        match string_from_ushort_ptr_ptr(text) {
            Ok(val) => println!("OK: {val}"),
            Err(err) => println!("Err: {err}"),
        }
    };

    Hook_AddTextToScroll_ushort_ptr_ptr.call(This, text, a, b, c)
}

pub static Hook_AddTextToScroll_ushort_ptr_ptr: Lazy<
    GenericDetour<fn_AddTextToScroll_Impl_ushort_ptr_ptr>,
> = Lazy::new(|| {
    println!("AddTextToScroll_Impl_ushort_ptr_ptr");

    // ClientSystem__AddTextToScroll = 0x004882F0, <- Broadcasts
    // ClientSystem__AddTextToScroll_ = 0x004C3010,
    // ClientSystem__AddTextToScroll__ = 0x005649F0,
    //    crashes on first invocation

    let address: isize = 0x005649F0 as isize;
    let ori: fn_AddTextToScroll_Impl_ushort_ptr_ptr = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl_ushort_ptr_ptr).unwrap() };
});

// char_ptr_ptr
type fn_AddTextToScroll_Impl_char_ptr_ptr =
    extern "thiscall" fn(This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32) -> i32;

extern "thiscall" fn Hook_AddTextToScroll_Impl_char_ptr_ptr(
    This: *mut c_void,
    text: *mut c_void,
    a: u32,
    b: u8,
    c: u32,
) -> i32 {
    println!("Hook_AddTextToScroll_Impl_B");

    unsafe {
        match string_from_char_ptr_ptr(text) {
            Ok(val) => println!("OK: {val}"),
            Err(err) => println!("Err: {err}"),
        }
    };

    Hook_AddTextToScroll_char_ptr_ptr.call(This, text, a, b, c)
}

pub static Hook_AddTextToScroll_char_ptr_ptr: Lazy<
    GenericDetour<fn_AddTextToScroll_Impl_char_ptr_ptr>,
> = Lazy::new(|| {
    println!("AddTextToScroll_Impl_B");

    // ClientSystem__AddTextToScroll = 0x004882F0, <- Broadcasts
    // ClientSystem__AddTextToScroll_ = 0x004C3010,
    // ClientSystem__AddTextToScroll__ = 0x005649F0,
    //    crashes on first invocation

    let address: isize = 0x004C3010 as isize;
    let ori: fn_AddTextToScroll_Impl_char_ptr_ptr = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl_char_ptr_ptr).unwrap() };
});

// char_ptr_ptr
type fn_AddTextToScroll_Impl_char_ptr =
    extern "thiscall" fn(This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32) -> i32;

extern "thiscall" fn Hook_AddTextToScroll_Impl_char_ptr(
    This: *mut c_void,
    text: *mut c_void,
    a: u32,
    b: u8,
    c: u32,
) -> i32 {
    println!("Hook_AddTextToScroll_Impl_char_ptr");

    unsafe {
        match string_from_char_ptr(text) {
            Ok(val) => println!("OK: {val}"),
            Err(err) => println!("Err: {err}"),
        }
    };

    Hook_AddTextToScroll_char_ptr.call(This, text, a, b, c)
}

pub static Hook_AddTextToScroll_char_ptr: Lazy<GenericDetour<fn_AddTextToScroll_Impl_char_ptr>> =
    Lazy::new(|| {
        println!("AddTextToScroll_Impl_char_ptr");

        // ClientSystem__AddTextToScroll = 0x004882F0, <- Broadcasts
        // ClientSystem__AddTextToScroll_ = 0x004C3010,
        // ClientSystem__AddTextToScroll__ = 0x005649F0,
        //    crashes on first invocation

        let address: isize = 0x004882F0 as isize;
        let ori: fn_AddTextToScroll_Impl_char_ptr = unsafe { std::mem::transmute(address) };
        return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl_char_ptr).unwrap() };
    });
