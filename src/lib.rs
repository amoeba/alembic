use retour::static_detour;
use std::error::Error;
use std::ffi::c_int;
use std::os::raw::c_void;
use std::{ffi::CString, iter, mem};
use windows::core::{w, PCSTR, PCWSTR};
use windows::Win32::Foundation::{BOOL, HANDLE, HWND};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};

static_detour! {
  static MessageBoxWHook: unsafe extern "system" fn(HWND, PCWSTR, PCWSTR, u32) -> c_int;
}

type FnMessageBoxW = unsafe extern "system" fn(HWND, PCWSTR, PCWSTR, u32) -> c_int;

unsafe fn main() -> Result<(), Box<dyn Error>> {
    let address = get_module_symbol_address("user32.dll", "MessageBoxW")
        .expect("could not find 'MessageBoxW' address");
    let target: FnMessageBoxW = mem::transmute(address);

    MessageBoxWHook
        .initialize(target, messageboxw_detour)?
        .enable()?;
    Ok(())
}

fn messageboxw_detour(hwnd: HWND, text: PCWSTR, _caption: PCWSTR, msgbox_style: u32) -> c_int {
    let replaced_caption = w!("Detoured!");
    unsafe { MessageBoxWHook.call(hwnd, text, replaced_caption, msgbox_style) }
}

fn get_module_symbol_address(module: &str, symbol: &str) -> Option<usize> {
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

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("attaching");
            unsafe { main().unwrap() }
        }
        DLL_PROCESS_DETACH => {
            println!("detaching");
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    };
    return BOOL::from(true);
}
