use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::slice;

pub struct PStringBase<T> {
    ptr: *const c_void,
    _phantom: PhantomData<T>,
}

impl<T> PStringBase<T> {
    pub fn new(ptr: *const c_void) -> Result<Self, &'static str> {
        if ptr.is_null() {
            Err("PStringBase pointer was null")
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }
}

impl PStringBase<i8> {
    /// # Safety
    /// The caller must ensure the pointer is valid and points to a null-terminated C string.
    pub unsafe fn to_string(&self) -> Result<String, &'static str> {
        let char_ptr = self.ptr as *const i8;
        CStr::from_ptr(char_ptr)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| "Invalid UTF-8 sequence")
    }
}

impl PStringBase<*const i8> {
    /// # Safety
    /// The caller must ensure the pointer is valid and points to a valid pointer to a null-terminated C string.
    pub unsafe fn to_string(&self) -> Result<String, &'static str> {
        let char_ptr_ptr = self.ptr as *const *const i8;
        let char_ptr = *char_ptr_ptr;
        if char_ptr.is_null() {
            return Err("wrapped char pointer was null");
        }

        CStr::from_ptr(char_ptr)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| "Invalid UTF-8 sequence")
    }
}

impl PStringBase<*const u16> {
    /// # Safety
    /// The caller must ensure the pointer is valid and points to a valid pointer to a null-terminated UTF-16 string.
    pub unsafe fn to_string(&self) -> Result<String, &'static str> {
        let ushort_ptr_ptr = self.ptr as *const *const u16;
        let ushort_ptr = *ushort_ptr_ptr;
        if ushort_ptr.is_null() {
            return Err("wrapped ushort pointer was null");
        }

        let mut len = 0;
        while *ushort_ptr.add(len) != 0 {
            len += 1;
        }

        let utf16_slice = slice::from_raw_parts(ushort_ptr, len);
        String::from_utf16(utf16_slice).map_err(|_| "Invalid UTF-16 sequence")
    }
}
