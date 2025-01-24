use std::ffi::c_void;

use libalembic::acclient::{PSRefBuffer, PStringBase};
use once_cell::sync::Lazy;
use retour::GenericDetour;

type fn_AddTextToScroll_Impl_A =
    extern "thiscall" fn(This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32) -> i32;

extern "thiscall" fn Hook_AddTextToScroll_Impl_A(
    This: *mut c_void,
    text: *mut c_void,
    a: u32,
    b: u8,
    c: u32,
) -> i32 {
    println!("Hook_AddTextToScroll_Impl_A");

    let wide_pstring = PStringBase::<u16> {
        m_buffer: text as *mut PSRefBuffer<u16>,
    };

    unsafe {
        match wide_pstring.to_string() {
            Ok(val) => println!("OK: {val}"),
            Err(err) => println!("Err: {err}"),
        }
    };

    Hook_AddTextToScroll_A.call(This, text, a, b, c)
}

pub static Hook_AddTextToScroll_A: Lazy<GenericDetour<fn_AddTextToScroll_Impl_A>> =
    Lazy::new(|| {
        println!("AddTextToScroll_Impl_A");

        // ClientSystem__AddTextToScroll = 0x004882F0, <- Broadcasts
        // ClientSystem__AddTextToScroll_ = 0x004C3010,
        // ClientSystem__AddTextToScroll__ = 0x005649F0,
        //    crashes on first invocation

        let address: isize = 0x005649F0 as isize;
        let ori: fn_AddTextToScroll_Impl_A = unsafe { std::mem::transmute(address) };
        return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl_A).unwrap() };
    });

type fn_AddTextToScroll_Impl_B =
    extern "thiscall" fn(This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32) -> i32;

extern "thiscall" fn Hook_AddTextToScroll_Impl_B(
    This: *mut c_void,
    text: *mut c_void,
    a: u32,
    b: u8,
    c: u32,
) -> i32 {
    println!("Hook_AddTextToScroll_Impl_B");

    let wide_pstring = PStringBase::<u8> {
        m_buffer: text as *mut PSRefBuffer<u8>,
    };

    unsafe {
        match wide_pstring.to_string() {
            Ok(val) => println!("OK: {val}"),
            Err(err) => println!("Err: {err}"),
        }
    };

    Hook_AddTextToScroll_A.call(This, text, a, b, c)
}

pub static Hook_AddTextToScroll_B: Lazy<GenericDetour<fn_AddTextToScroll_Impl_B>> =
    Lazy::new(|| {
        println!("AddTextToScroll_Impl_B");

        // ClientSystem__AddTextToScroll = 0x004882F0, <- Broadcasts
        // ClientSystem__AddTextToScroll_ = 0x004C3010,
        // ClientSystem__AddTextToScroll__ = 0x005649F0,
        //    crashes on first invocation

        let address: isize = 0x004C3010 as isize;
        let ori: fn_AddTextToScroll_Impl_B = unsafe { std::mem::transmute(address) };
        return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl_B).unwrap() };
    });
