use std::io::Error;

use widestring::{U16CString, U16String};
use windows::{
    core::PCWSTR,
    Win32::{Foundation::HWND, UI::WindowsAndMessaging::*},
};

fn main() {
    println!("Hello, world!");

    unsafe {
        let result = show_message_box();
        match result {
            Ok(()) => println!("Good result"),
            Err(e) => println!("Err: {e:?}"),
        }
    };
}

unsafe fn show_message_box() -> Result<(), Error> {
    let lptext = U16String::from_str("Hi from Rust!");
    let lpcaption = U16CString::from_str("Hello").unwrap();
    let result = MessageBoxW(
        HWND::default(),
        PCWSTR(lptext.as_ptr()),
        PCWSTR(lpcaption.as_ptr()),
        MB_OK,
    );

    println!("got result of {result:?}");

    Ok(())
}
