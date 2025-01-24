use std::ffi::c_void;
use std::slice;
use std::str;

#[repr(C)]
pub struct PSRefBufferCharData {
    m_data: [i32; 256],
}

#[repr(C)]
pub struct PStringBase {
    pub m_Charbuffer: *mut PSRefBufferCharData,
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
        if self.m_Charbuffer.is_null() {
            return Err("Null buffer pointer");
        }

        let buffer = &*self.m_Charbuffer;
        let data_ptr = buffer.m_data.as_ptr() as *const u8;

        // Try UTF-8 first
        if let Ok(s) = str::from_utf8(slice::from_raw_parts(data_ptr, 1024)) {
            return Ok(s.trim_end_matches('\0').to_string());
        }

        // If UTF-8 fails, try UTF-16
        let utf16_ptr = data_ptr as *const u16;
        let mut len = 0;
        while *utf16_ptr.add(len) != 0 && len < 512 {
            len += 1;
        }

        let utf16_slice = slice::from_raw_parts(utf16_ptr, len);
        String::from_utf16(utf16_slice).map_err(|_| "Invalid UTF-16 sequence")
    }
}
