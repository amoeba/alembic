use std::error::Error;

use crate::settings::{AccountInfo, ClientInfo, DllInfo, ServerInfo};

use super::{noop::NoopLauncher, windows::WindowsLauncher, wine::WineLauncher};

// Define a container for the return value of launch so this can be
// cross-platform
#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub enum LaunchResult {
    ProcessInformation(windows::Win32::System::Threading::PROCESS_INFORMATION),
}

// Fallback implementation for other platforms
#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub enum LaunchResult {
    ProcessId(u32),
}

// Define an enum of Launcher impls so we can refer to any at once
#[derive(Debug)]
pub enum LauncherImpl {
    WindowsLauncher(WindowsLauncher),
    WineLauncher(WineLauncher),
    NoopLauncher(NoopLauncher),
}

pub trait Launcher {
    fn new(
        client_info: ClientInfo,
        server_info: ServerInfo,
        account_info: AccountInfo,
        dll_path: DllInfo,
    ) -> Self;
    fn launch(&self) -> Result<LaunchResult, Box<dyn Error>>;
}
