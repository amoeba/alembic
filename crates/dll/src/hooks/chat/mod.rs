use std::ffi::c_void;

use libalembic::acclient::PStringBase;
use once_cell::sync::Lazy;
use retour::GenericDetour;

// char_ptr
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
        match PStringBase::<i8>::new(text).and_then(|p| p.to_string()) {
            Ok(str) => println!("str is {str}"),
            Err(err) => println!("err is {err}"),
        }
    }

    Hook_AddTextToScroll_char_ptr.call(This, text, a, b, c)
}

pub static Hook_AddTextToScroll_char_ptr: Lazy<GenericDetour<fn_AddTextToScroll_Impl_char_ptr>> =
    Lazy::new(|| {
        let address: isize = 0x004882F0 as isize;
        let ori: fn_AddTextToScroll_Impl_char_ptr = unsafe { std::mem::transmute(address) };
        return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl_char_ptr).unwrap() };
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
    println!("Hook_AddTextToScroll_Impl_char_ptr_ptr");

    unsafe {
        match PStringBase::<*const i8>::new(text).and_then(|p| p.to_string()) {
            Ok(str) => println!("str is {str}"),
            Err(err) => println!("err is {err}"),
        }
    }

    Hook_AddTextToScroll_char_ptr_ptr.call(This, text, a, b, c)
}

pub static Hook_AddTextToScroll_char_ptr_ptr: Lazy<
    GenericDetour<fn_AddTextToScroll_Impl_char_ptr_ptr>,
> = Lazy::new(|| {
    let address: isize = 0x004C3010 as isize;
    let ori: fn_AddTextToScroll_Impl_char_ptr_ptr = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl_char_ptr_ptr).unwrap() };
});

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
        match PStringBase::<*const u16>::new(text).and_then(|p| p.to_string()) {
            Ok(str) => println!("str is {str}"),
            Err(err) => println!("err is {err}"),
        }
    }

    Hook_AddTextToScroll_ushort_ptr_ptr.call(This, text, a, b, c)
}

pub static Hook_AddTextToScroll_ushort_ptr_ptr: Lazy<
    GenericDetour<fn_AddTextToScroll_Impl_ushort_ptr_ptr>,
> = Lazy::new(|| {
    let address: isize = 0x005649F0 as isize;
    let ori: fn_AddTextToScroll_Impl_ushort_ptr_ptr = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, Hook_AddTextToScroll_Impl_ushort_ptr_ptr).unwrap() };
});
