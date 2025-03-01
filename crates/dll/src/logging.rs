/// log_message
///
/// Use this instead of println! calls in DLL code since println! invokes the
/// C runtime whereas this just directly performs a Windows API call. Apparently
/// invoking the C runtime from DLL code can be dangerous.
#[inline(always)]
pub unsafe fn log_message(message: &str) {
    use windows::Win32::System::Diagnostics::Debug::OutputDebugStringA;

    let c_string = match std::ffi::CString::new(message) {
        Ok(s) => s,
        Err(_) => return, // CString::new errors if any non-terminal bytes
                          // are \0 so in that case we just don't log it.
    };

    OutputDebugStringA(windows::core::PCSTR(c_string.as_ptr() as _));
}
