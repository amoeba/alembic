// Safe logging that doesn't allocate or use CRT
#[inline(always)]
pub unsafe fn log_event(msg: &str) {
    // OutputDebugStringA is safer than println! in DllMain
    use windows::Win32::System::Diagnostics::Debug::OutputDebugStringA;
    let msg = format!("{}\0", msg);
    OutputDebugStringA(msg.as_ptr().cast());
}