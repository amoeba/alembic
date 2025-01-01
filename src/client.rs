use std::ffi::c_void;
use std::fmt;

#[repr(C)]
pub struct PStringBase<T> {
    // Implement this based on the actual definition of PStringBase
    // For now, we'll use a placeholder
    _data: *mut T,
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
        write!(f, "m_strToken:{:?} m_strEnglish:{:?} m_strComment:{:?} m_LiteralValue:{:?} m_stringID:{:08X} m_tableID:{:08X} m_Override:{}",
               self.m_str_token._data,
               self.m_str_english._data,
               self.m_str_comment._data,
               self.m_literal_value._data,
               self.m_string_id,
               self.m_table_id,
               self.m_override)
    }
}
