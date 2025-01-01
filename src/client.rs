use std::ffi::c_void;
use std::fmt;
use std::marker::PhantomData;

//
// Section: UIRegion
//

#[repr(C)]
pub struct Turbine_RefCount {
    pub vfptr: *const Vtbl,
    pub m_cRef: u32,
}

#[repr(C)]
pub struct Vtbl {
    pub __scaDelDtor: unsafe extern "thiscall" fn(*mut Turbine_RefCount, u32) -> *mut c_void,
}

impl Turbine_RefCount {
    pub const __SCA_DEL_DTOR_ADDR: usize = 0x00401C30;

    pub unsafe fn __scaDelDtor(&mut self, arg: u32) -> *mut c_void {
        ((*self.vfptr).__scaDelDtor)(self, arg)
    }
}

impl fmt::Display for Turbine_RefCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m_cRef:{:08X}", self.m_cRef)
    }
}

#[repr(C)]
pub struct PSRefBuffer<T: Copy> {
    pub _ref: Turbine_RefCount,
    pub m_len: i32,
    pub m_size: u32,
    pub m_hash: u32,
    pub m_data: [i32; 128],
    _phantom: PhantomData<T>,
}

#[repr(C)]
pub struct PStringBase<T: Copy> {
    pub m_buffer: *mut PSRefBuffer<T>,
}

impl<T: Copy> PStringBase<T> {
    pub fn to_string(&self) -> String {
        unsafe {
            if self.m_buffer.is_null() || (*self.m_buffer).m_len == 0 {
                return "null".to_string();
            }

            let data_ptr = (*self.m_buffer).m_data.as_ptr() as *const u8;
            println!(
                "DEBUG: PStringBase to_string thinks len is {:?}",
                (*self.m_buffer).m_len
            );
            let len = (*self.m_buffer).m_len as usize - 1;

            if std::mem::size_of::<T>() == 2 {
                // Assuming T is u16 (for UTF-16)
                let slice = std::slice::from_raw_parts(data_ptr as *const u16, len);
                String::from_utf16_lossy(slice)
            } else {
                // Assuming T is u8 (for UTF-8)
                let slice = std::slice::from_raw_parts(data_ptr, len);
                String::from_utf8_lossy(slice).into_owned()
            }
        }
    }
}

impl<T: Copy> std::fmt::Display for PStringBase<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[repr(C)]
pub struct HashTable<K, V> {
    // Implement this based on the actual definition of HashTable
    // For now, we'll use placeholders
    _key: K,
    _value: V,
}

#[repr(C)]
pub struct StringInfoData {
    // Implement this based on the actual definition of StringInfoData
    // For now, we'll use a placeholder
    _data: u32,
}

#[repr(C)]
pub struct StringInfo {
    pub m_str_token: PStringBase<u8>,
    pub m_string_id: u32,
    pub m_table_id: u32,
    pub m_variables: HashTable<u32, StringInfoData>,
    pub m_literal_value: PStringBase<u16>,
    pub m_override: u8,
    pub m_str_english: PStringBase<u8>,
    pub m_str_comment: PStringBase<u8>,
}

impl StringInfo {
    pub unsafe fn from_ptr(ptr: *mut c_void) -> &'static mut Self {
        &mut *(ptr as *mut StringInfo)
    }
}

impl fmt::Display for StringInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m_strToken:{} m_strEnglish:{} m_strComment:{} m_LiteralValue:{} m_stringID:{:08X} m_tableID:{:08X} m_Override:{}",
               self.m_str_token,
               self.m_str_english,
               self.m_str_comment,
               self.m_literal_value,
               self.m_string_id,
               self.m_table_id,
               self.m_override)
    }
}
