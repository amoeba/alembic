use once_cell::sync::Lazy;
use retour::GenericDetour;
use std::ffi::c_int;
use std::os::raw::c_void;
use std::ptr::null_mut;
use widestring::U16String;
use windows::core::w;
use windows::core::PCSTR;
use windows::core::PCWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::GetTokenInformation;
use windows::Win32::Security::TokenPrivileges;
use windows::Win32::Security::TOKEN_QUERY;
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::STD_ERROR_HANDLE;
use windows::Win32::System::Memory::VirtualProtect;
use windows::Win32::System::Memory::VirtualProtectEx;
use windows::Win32::System::Memory::PAGE_EXECUTE_READWRITE;
use windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS;
use windows::Win32::System::ProcessStatus::K32GetProcessImageFileNameA;
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::System::Threading::GetProcessId;
use windows::Win32::System::Threading::OpenProcessToken;
use windows::Win32::System::Threading::PROCESS_VM_OPERATION;
use windows::Win32::UI::WindowsAndMessaging::MessageBoxW;
use windows::Win32::UI::WindowsAndMessaging::MB_OK;

use windows::Win32::System::Console::{AllocConsole, SetStdHandle, STD_OUTPUT_HANDLE};

use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};

// Chorizite has this as
// private static int RecvFromImpl(nint s, byte* buf, int len, int flags, byte* from, int fromlen) {
// static_detour! {
//   static RecvFromImplHook: unsafe extern "system" fn(*mut c_void, *mut u8, c_int, c_int, *mut u8, c_int) -> c_int;
// }

type FnRecvFromImplHook =
    unsafe extern "system" fn(*mut c_void, *mut u8, c_int, c_int, *mut u8, c_int) -> c_int;

unsafe fn allocate_console() -> windows::core::Result<()> {
    unsafe {
        // Allocate a new console
        AllocConsole()?;

        // Redirect stdout
        let stdout_handle = CreateFileA(
            PCSTR("CONOUT$\0".as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )?;

        SetStdHandle(STD_OUTPUT_HANDLE, stdout_handle)?;

        // Redirect stderr
        let stderr_handle = CreateFileA(
            PCSTR("CONOUT$\0".as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )?;

        SetStdHandle(STD_ERROR_HANDLE, stderr_handle)?;
    }

    println!("Console allocated and streams redirected successfully!");
    eprintln!("This is an error message test.");

    Ok(())
}

pub fn is_executable_address(address: *const ()) -> bool {
    region::query(address as *const _)
        .unwrap()
        .protection()
        .contains(region::Protection::EXECUTE)
}

extern "system" fn my_recv_from_impl_hook(
    s: *mut c_void,
    buf: *mut u8,
    len: c_int,
    flags: c_int,
    from: *mut u8,
    fromlen: c_int,
) -> c_int {
    println!("my_recv_from_impl_hook called with args:");
    println!("  Socket: {:?}", s);
    println!("  Buffer length: {}", len);
    println!("  Flags: {}", flags);
    println!("  From length: {}", fromlen);
    unsafe { HOOK_RECV.disable().unwrap() };
    let ret_val = unsafe { HOOK_RECV.call(s, buf, len, flags, from, fromlen) };
    unsafe { HOOK_RECV.enable().unwrap() };
    ret_val
}

static HOOK_RECV: Lazy<GenericDetour<FnRecvFromImplHook>> = Lazy::new(|| {
    let address: i32 = 0x007935AC;

    let ori: FnRecvFromImplHook = unsafe { std::mem::transmute(address) };
    return unsafe { GenericDetour::new(ori, my_recv_from_impl_hook).unwrap() };
});

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    unsafe { allocate_console().unwrap() };

    match reason {
        DLL_PROCESS_ATTACH => {
            println!("attaching");
            unsafe {
                // let address: i32 = 0x007935AC;
                let address = 0x00793000;
                let mut old_protect: u32 = 0;

                // C++
                // BOOL VirtualProtectEx(
                //     [in]  HANDLE hProcess,
                //     [in]  LPVOID lpAddress,
                //     [in]  SIZE_T dwSize,
                //     [in]  DWORD  flNewProtect,
                //     [out] PDWORD lpflOldProtect
                //   );
                //
                // Rust
                // pub unsafe fn VirtualProtectEx<P0>(
                //     hprocess: P0,
                //     lpaddress: *const c_void,
                //     dwsize: usize,
                //     flnewprotect: PAGE_PROTECTION_FLAGS,
                //     lpfloldprotect: *mut PAGE_PROTECTION_FLAGS,
                // ) -> Result<()>
                // where
                //     P0: Param<HANDLE>,

                // Chorizite code example:
                // VirtualProtectEx(Process.GetCurrentProcess().Handle, address, (UIntPtr)4, 0x40, out int b);
                // *(int*)address = newValue;
                // VirtualProtectEx(Process.GetCurrentProcess().Handle, address, (UIntPtr)4, b, out b);

                // windbg output
                // 0:022> !address 0x007935AC

                // Mapping file section regions...
                // Mapping module regions...
                // Mapping PEB regions...
                // Mapping TEB and stack regions...
                // Mapping heap regions...
                // Mapping page heap regions...
                // Mapping other regions...
                // Mapping stack trace database regions...
                // Mapping activation context regions...

                // Usage:                  Image
                // Base Address:           00793000
                // End Address:            0080b000
                // Region Size:            00078000 ( 480.000 kB)
                // State:                  00001000          MEM_COMMIT
                // Protect:                00000002          PAGE_READONLY
                // Type:                   01000000          MEM_IMAGE
                // Allocation Base:        00400000
                // Allocation Protect:     00000080          PAGE_EXECUTE_WRITECOPY
                // Image Path:             C:\Games\AC\acclient.exe
                // Module Name:            acclient
                // Loaded Image Name:      C:\Games\AC\acclient.exe
                // Mapped Image Name:
                // More info:              lmv m acclient
                // More info:              !lmi acclient
                // More info:              ln 0x7935ac
                // More info:              !dh 0x400000

                // Content source: 1 (target), length: 77a54

                let process_handle = GetCurrentProcess();
                let process_id = GetProcessId(process_handle);

                // Get process name (path)
                let mut buffer = [0u8; 260]; // MAX_PATH
                K32GetProcessImageFileNameA(process_handle, &mut buffer);
                println!("Process ID: {}", process_id);
                println!(
                    "Process Path: {}",
                    String::from_utf8_lossy(&buffer).trim_matches(char::from(0))
                );

                let mut privileges_length = 0;
                let mut token_handle = HANDLE::default();

                if OpenProcessToken(process_handle, TOKEN_QUERY, &mut token_handle).is_ok() {
                    // First call to get required buffer size
                    GetTokenInformation(
                        token_handle,
                        TokenPrivileges,
                        Some(std::ptr::null_mut()),
                        0,
                        &mut privileges_length,
                    );

                    let mut buffer = vec![0u8; privileges_length as usize];

                    // Second call to actually get privileges
                    if GetTokenInformation(
                        token_handle,
                        TokenPrivileges,
                        Some(buffer.as_mut_ptr() as *mut _),
                        privileges_length,
                        &mut privileges_length,
                    )
                    .is_ok()
                    {
                        // Check if PROCESS_VM_OPERATION is present
                        // let has_vm_operation =
                        //     (process_handle & PROCESS_VM_OPERATION.0) == PROCESS_VM_OPERATION.0;
                        // println!("Has PROCESS_VM_OPERATION: {}", has_vm_operation);
                        println!("We appear to have {process_handle:?}...")
                    }

                    CloseHandle(token_handle);
                }

                if process_handle == _hinst {
                    println!("process is _hinst, FYI");
                } else {
                    println!("process is not _hinst, FYI");
                }

                if is_executable_address(address as *const ()) {
                    println!("before virtualprotectex, address is executable");
                } else {
                    println!("before virtualprotectex, address is not executable");
                }

                println!("Trying VirtualProtectEx");
                // VirtualProtectEx(Process.GetCurrentProcess().Handle, address, (UIntPtr)4, 0x40, out int b);
                let old_protect: *mut PAGE_PROTECTION_FLAGS = std::ptr::null_mut();
                match VirtualProtectEx(
                    process_handle,
                    address as *const c_void,
                    0x00078000 as usize,
                    PAGE_EXECUTE_READWRITE,
                    old_protect,
                ) {
                    Ok(_) => todo!(),
                    Err(error) => {
                        eprintln!("VirtualProcessEx call failed: {error:?}")
                    }
                }
                println!("Done VirtualProtectEx");

                if is_executable_address(address as *const ()) {
                    println!("after virtualprotectex, address is executable");
                } else {
                    println!("after virtualprotectex, address is still not executable");
                }

                // Temporarily comment out until I figure out VirtualProtectEx
                // match HOOK_RECV.enable() {
                //     Ok(_) => {
                //         println!("Hook enable success");
                //     }
                //     Err(error) => {
                //         println!("Hook enable error: {error:?}")
                //     }
                // }
            }
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
