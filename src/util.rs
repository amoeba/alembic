use std::{ffi::CString, iter};

use windows::{
    core::{PCSTR, PCWSTR},
    Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
};

pub fn print_dbg_address(addr: isize, friendly_name: &str) {
    let q = region::query(addr as *const ()).unwrap();

    if q.is_executable() {
        println!("{friendly_name} is executable")
    } else {
        println!("{friendly_name} is NOT executable")
    }
}

pub fn get_module_symbol_address(module: &str, symbol: &str) -> Option<usize> {
    let module = module
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    let symbol = CString::new(symbol).unwrap();
    unsafe {
        let handle = GetModuleHandleW(PCWSTR(module.as_ptr() as _)).unwrap();
        match GetProcAddress(handle, PCSTR(symbol.as_ptr() as _)) {
            Some(func) => Some(func as usize),
            None => None,
        }
    }
}

pub fn print_vec(v: &Vec<u8>) {
    for (i, byte) in v.iter().enumerate() {
        print!("{byte:02X} ");

        if (i + 1) % 16 == 0 {
            println!();
        }
    }
}
