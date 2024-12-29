use std::io::Error;

use widestring::U16CString;
use windows::{
    core::PCWSTR,
    Win32::{Foundation::HWND, UI::WindowsAndMessaging::*},
};

unsafe fn show_message_box(title: &str, message: &str) -> Result<(), Error> {
    let lptext = U16CString::from_str(title).unwrap();
    let lpcaption = U16CString::from_str(&message).unwrap();
    let result = MessageBoxW(
        HWND::default(),
        PCWSTR(lptext.as_ptr()),
        PCWSTR(lpcaption.as_ptr()),
        MB_OK,
    );

    println!("got result of {result:?}");

    Ok(())
}

#[ctor::ctor]
fn ctor() {
    println!("lib hello");

    unsafe {
        show_message_box("Hello", "World");
    }
}
