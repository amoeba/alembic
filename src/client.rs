use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_int;

#[repr(C)]
pub struct TurbineRefCount {
    pub ref_count: c_int,
}

#[repr(C)]
pub struct PSRefBuffer<T: Copy> {
    pub _ref: TurbineRefCount,
    pub m_len: i32,
    pub m_size: u32,
    pub m_hash: u32,
    pub m_data: [i32; 128], // Fixed-size array of i32 (treated as bytes or u16s)
    pub _phantom: PhantomData<T>,
}

impl<T: Copy> PSRefBuffer<T> {
    pub fn new() -> Self {
        PSRefBuffer {
            _ref: TurbineRefCount { ref_count: 0 },
            m_len: 0,
            m_size: 0,
            m_hash: 0,
            m_data: [0; 128],
            _phantom: PhantomData,
        }
    }
}

#[repr(C)]
pub struct PStringBase<T: Copy> {
    pub m_buffer: *mut PSRefBuffer<T>,
    pub _phantom: PhantomData<T>,
}

impl<T: Copy> PStringBase<T> {
    /// Creates a new `PStringBase<T>` from a mutable pointer to `PSRefBuffer<T>`.
    ///
    /// # Safety
    /// The caller must ensure that the pointer is valid and points to a `PSRefBuffer<T>`.
    pub unsafe fn from_mut_ptr(ptr: *mut PSRefBuffer<T>) -> Self {
        PStringBase {
            m_buffer: ptr,
            _phantom: PhantomData,
        }
    }

    /// Converts the `PStringBase<T>` to a string.
    ///
    /// If `T` is `u16`, the `m_data` array is treated as UTF-16.
    /// Otherwise, it is treated as a sequence of bytes (ASCII or UTF-8).
    pub fn to_string(&self) -> String {
        if self.m_buffer.is_null() {
            return "null".to_string();
        }

        let buffer = unsafe { &*self.m_buffer };
        if buffer.m_len == 0 {
            return "null".to_string();
        }

        // Treat m_data as u16 (UTF-16)
        let u16_slice = unsafe {
            std::slice::from_raw_parts(buffer.m_data.as_ptr() as *const u16, buffer.m_len as usize)
        };
        String::from_utf16_lossy(u16_slice)
    }
}

impl<T: Copy> fmt::Display for PStringBase<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
