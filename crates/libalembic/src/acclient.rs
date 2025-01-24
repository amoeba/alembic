use std::ffi::{c_void, CStr};
use std::slice;
use std::str;

pub unsafe fn string_from_char_ptr(ptr: *const c_void) -> Result<String, &'static str> {
    if ptr.is_null() {
        return Err("Null pointer provided");
    }

    let char_ptr = ptr as *const i8;
    CStr::from_ptr(char_ptr)
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| "Invalid UTF-8 sequence")
}

pub unsafe fn string_from_char_ptr_ptr(ptr: *const c_void) -> Result<String, &'static str> {
    if ptr.is_null() {
        return Err("Null pointer provided");
    }

    let char_ptr_ptr = ptr as *const *const i8;
    let char_ptr = *char_ptr_ptr;
    if char_ptr.is_null() {
        return Err("Null char pointer");
    }

    CStr::from_ptr(char_ptr)
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| "Invalid UTF-8 sequence")
}

pub unsafe fn string_from_ushort_ptr_ptr(ptr: *const c_void) -> Result<String, &'static str> {
    if ptr.is_null() {
        return Err("Null pointer provided");
    }

    let ushort_ptr_ptr = ptr as *const *const u16;
    let ushort_ptr = *ushort_ptr_ptr;
    if ushort_ptr.is_null() {
        return Err("Null ushort pointer");
    }

    let mut len = 0;
    while *ushort_ptr.add(len) != 0 {
        len += 1;
    }

    let utf16_slice = slice::from_raw_parts(ushort_ptr, len);
    String::from_utf16(utf16_slice).map_err(|_| "Invalid UTF-16 sequence")
}
