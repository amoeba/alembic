use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::slice;
use std::str;

#[repr(C)]
pub struct Turbine_RefCount {
    count: i32,
}

#[repr(C)]
pub struct PSRefBuffer {
    _ref: Turbine_RefCount,
    m_len: i32,
    m_size: u32,
    m_hash: u32,
    m_data: [i32; 128],
}

#[repr(C)]
pub struct PStringBase {
    pub m_buffer: *mut PSRefBuffer,
}

impl PStringBase {
    pub unsafe fn from_ptr(ptr: *const c_void) -> Result<String, &'static str> {
        if ptr.is_null() {
            return Err("Null pointer provided");
        }

        let pstring = &*(ptr as *const PStringBase);
        pstring.to_string()
    }

    pub unsafe fn to_string(&self) -> Result<String, &'static str> {
        if self.m_buffer.is_null() {
            return Err("Null buffer pointer");
        }

        let buffer = &*self.m_buffer;
        let data_ptr = buffer.m_data.as_ptr() as *const c_char;

        // Try UTF-8 first
        if let Ok(s) = CStr::from_ptr(data_ptr).to_str() {
            return Ok(s.to_string());
        }

        // If UTF-8 fails, try UTF-16
        let mut len = 0;
        while *data_ptr.add(len * 2) != 0 || *data_ptr.add(len * 2 + 1) != 0 {
            len += 1;
        }

        let utf16_slice = slice::from_raw_parts(data_ptr as *const u16, len);
        String::from_utf16(utf16_slice).map_err(|_| "Invalid UTF-16 sequence")
    }
}
