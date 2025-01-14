use std::fmt;
use std::os::raw::c_int;

#[repr(C)]
pub struct TurbineRefCount {
    pub ref_count: c_int,
}

#[repr(C)]
pub struct PSRefBuffer {
    pub _ref: TurbineRefCount,
    pub m_len: i32,
}

impl PSRefBuffer {
    pub fn new() -> Self {
        PSRefBuffer {
            _ref: TurbineRefCount { ref_count: 0 },
            m_len: 0,
        }
    }
}

#[repr(C)]
pub struct PStringBase {
    pub m_buffer: *mut PSRefBuffer,
}

impl PStringBase {
    pub unsafe fn from_mut_ptr(ptr: *mut PSRefBuffer) -> Self {
        PStringBase { m_buffer: ptr }
    }

    pub fn to_string(&self) -> String {
        if self.m_buffer.is_null() {
            return "null".to_string();
        }

        let buffer = unsafe { &*self.m_buffer };
        if buffer.m_len == 0 {
            return "zerolen".to_string();
        } else {
            let len = buffer.m_len;
            return format!("PStringBase.to_string: non-zero-len (len={len})");
        }
    }
}

impl fmt::Display for PStringBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
