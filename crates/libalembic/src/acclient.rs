use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::size_of;

#[repr(C)]
pub struct TurbineRefCount {
    count: i32,
}

#[repr(C)]
pub struct PSRefBuffer<T: Copy> {
    _ref: TurbineRefCount,
    m_len: i32,
    m_size: u32,
    m_hash: u32,
    m_data: [i32; 128],
    _phantom: PhantomData<T>,
}

#[repr(C)]
pub struct PStringBase<T: Copy> {
    pub m_buffer: *mut PSRefBuffer<T>,
}

impl<T: Copy> PStringBase<T> {
    pub unsafe fn from_ptr(ptr: *const c_void) -> Result<String, &'static str> {
        if ptr.is_null() {
            return Err("Null pointer provided");
        }

        let pstring = &*(ptr as *const PStringBase<T>);

        if pstring.m_buffer.is_null() {
            return Err("Null buffer pointer");
        }

        let buffer = &*pstring.m_buffer;

        if buffer.m_len <= 0 || buffer.m_len > 128 {
            return Err("Invalid buffer length");
        }

        let data_ptr = buffer.m_data.as_ptr() as *const T;
        let slice = std::slice::from_raw_parts(data_ptr, buffer.m_len as usize);

        match std::str::from_utf8(std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            buffer.m_len as usize * size_of::<T>(),
        )) {
            Ok(s) => Ok(s.trim_end_matches('\0').to_string()),
            Err(_) => Err("Invalid UTF-8 sequence"),
        }
    }

    pub unsafe fn to_string(&self) -> Result<String, &'static str> {
        if self.m_buffer.is_null() {
            return Err("Null buffer pointer");
        }

        let buffer = &*self.m_buffer;

        let len = buffer.m_len;
        println!("inside to_string, buffer len is {len}");

        if buffer.m_len <= 0 || buffer.m_len > 128 {
            return Err("Invalid buffer length");
        }

        let data_ptr = buffer.m_data.as_ptr() as *const T;
        let slice = std::slice::from_raw_parts(data_ptr, buffer.m_len as usize);

        match std::str::from_utf8(std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            buffer.m_len as usize * size_of::<T>(),
        )) {
            Ok(s) => Ok(s.trim_end_matches('\0').to_string()),
            Err(_) => Err("Invalid UTF-8 sequence"),
        }
    }
}
