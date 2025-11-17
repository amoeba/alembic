#![cfg(all(target_os = "windows", target_env = "msvc"))]

//! Native Rust implementation of DLL injection functionality
//! Based on Mag-ACClientLauncher's Injector.cs
//! https://github.com/Mag-nus/Mag-ACClientLauncher/blob/master/Source/Win32/Injector.cs

use anyhow::{Context, Result};
use std::ffi::CString;
use std::path::Path;
use windows::core::{PCSTR, PSTR};
use windows::Win32::Foundation::{CloseHandle, FreeLibrary, HANDLE, HMODULE, WAIT_OBJECT_0};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateProcessA, CreateRemoteThread, GetExitCodeThread, ResumeThread, WaitForSingleObject,
    CREATE_SUSPENDED, INFINITE, PROCESS_INFORMATION, STARTUPINFOA,
};

/// MemoryGuard automatically calls VirtualFreeEx on Drop
struct MemoryGuard {
    process_handle: HANDLE,
    address: *mut std::ffi::c_void,
    size: usize,
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        unsafe {
            VirtualFreeEx(self.process_handle, self.address, self.size, MEM_RELEASE).ok();
        }
    }
}

/// HandleGuard automatically calls FreeLibrary on Drop
struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0).ok();
        }
    }
}

/// Guard to automatically free loaded libraries
struct LibraryGuard(HMODULE);

impl Drop for LibraryGuard {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.0).ok();
        }
    }
}

/// Launch a process in suspended state, inject a DLL, optionally call a function, and resume the process
///
/// # Arguments
/// * `executable_path` - Absolute path to the executable to launch
/// * `executable_args` - Command line arguments for the executable
/// * `dll_path` - Absolute path to the DLL to inject
/// * `dll_function` - Optional name of a function to execute in the DLL after injection
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err` with details if any step fails
pub fn launch_suspended_inject_and_resume(
    executable_path: &str,
    executable_args: &str,
    dll_path: &str,
    dll_function: Option<&str>,
) -> Result<()> {
    // Combine filename and executable_args into command line
    let command_line = format!("{} {}", executable_path, executable_args);
    let command_line_cstring =
        CString::new(command_line).context("Failed to create command line CString")?;

    // Get working directory from the executable path
    let working_dir = Path::new(executable_path)
        .parent()
        .and_then(|p| p.to_str())
        .context("Failed to get working directory from file path")?;
    let working_dir_cstring =
        CString::new(working_dir).context("Failed to create working directory CString")?;

    let mut startup_info: STARTUPINFOA = unsafe { std::mem::zeroed() };
    startup_info.cb = std::mem::size_of::<STARTUPINFOA>() as u32;

    let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    // Create the process in suspended state
    // CreateProcessA requires a mutable command line buffer, so we need to use PSTR
    let success = unsafe {
        CreateProcessA(
            PCSTR::null(),
            PSTR(command_line_cstring.as_ptr() as *mut u8),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            PCSTR(working_dir_cstring.as_ptr() as *const u8),
            &startup_info,
            &mut process_info,
        )
    };

    if !success.is_ok() {
        return Err(anyhow::anyhow!("CreateProcessA failed"));
    }

    // Inject the DLL (and optionally execute function)
    let result = inject_into_process(process_info.hProcess, dll_path, dll_function);

    // Always resume the thread and close handles
    unsafe {
        ResumeThread(process_info.hThread);
        CloseHandle(process_info.hThread).ok();
        CloseHandle(process_info.hProcess).ok();
    }

    result
}

/// Inject a DLL into an existing process
///
/// # Arguments
/// * `process_handle` - Handle to the target process
/// * `dll_path` - Absolute path to the DLL to inject
/// * `dll_function` - Optional name of a function to execute in the DLL after injection
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err` with details if any step fails
pub fn inject_into_process(
    process_handle: HANDLE,
    dll_path: &str,
    dll_function: Option<&str>,
) -> Result<()> {
    let dll_path_cstring = CString::new(dll_path).context("Failed to create DLL path CString")?;
    let dll_path_bytes = dll_path_cstring.as_bytes_with_nul();
    // let dll_path_size = dll_path_bytes.len();
    // TODO: When I set this manually to 44 injection fails
    let dll_path_size = 44;

    // TODO: should be 44
    println!("dll_path_size is {:?}", dll_path_size);

    // Allocate memory in the target process for the DLL path
    let alloc_mem_address = unsafe {
        VirtualAllocEx(
            process_handle,
            None,
            dll_path_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    if alloc_mem_address.is_null() {
        return Err(anyhow::anyhow!("VirtualAllocEx failed"));
    }

    // Ensure we free the memory even if subsequent operations fail
    let _guard = MemoryGuard {
        process_handle,
        address: alloc_mem_address,
        size: dll_path_size,
    };

    // Write the DLL path to the allocated memory
    let mut bytes_written = 0;
    let success = unsafe {
        WriteProcessMemory(
            process_handle,
            alloc_mem_address,
            dll_path_bytes.as_ptr() as *const _,
            dll_path_size,
            Some(&mut bytes_written),
        )
    };

    if !success.is_ok() {
        return Err(anyhow::anyhow!("WriteProcessMemory failed"));
    }

    // Get the address of LoadLibraryA from kernel32.dll
    let kernel32_cstring = CString::new("kernel32.dll")?;
    let kernel32_handle =
        unsafe { GetModuleHandleA(PCSTR(kernel32_cstring.as_ptr() as *const u8)) }
            .context("GetModuleHandleA failed for kernel32.dll")?;

    let load_library_cstring = CString::new("LoadLibraryA")?;
    let load_library_addr = unsafe {
        GetProcAddress(
            kernel32_handle,
            PCSTR(load_library_cstring.as_ptr() as *const u8),
        )
    }
    .context("GetProcAddress failed for LoadLibraryA")?;

    // Create a remote thread that calls LoadLibraryA with the DLL path
    let remote_thread_handle = unsafe {
        CreateRemoteThread(
            process_handle,
            None,
            0,
            Some(std::mem::transmute(load_library_addr)),
            Some(alloc_mem_address),
            0,
            None,
        )
    }
    .context("CreateRemoteThread failed")?;

    // Ensure we close the thread handle
    let _thread_guard = HandleGuard(remote_thread_handle);

    // Wait for the remote thread to complete
    unsafe {
        WaitForSingleObject(remote_thread_handle, INFINITE);
    }

    // Get the exit code (which is the base address of the loaded DLL)
    let mut injected_dll_address: u32 = 0;
    let success = unsafe { GetExitCodeThread(remote_thread_handle, &mut injected_dll_address) };

    if !success.is_ok() {
        return Err(anyhow::anyhow!("GetExitCodeThread failed"));
    }

    if injected_dll_address == 0 {
        return Err(anyhow::anyhow!(
            "DLL injection failed - LoadLibraryA returned 0"
        ));
    }

    // If we have a function to execute, call it
    if let Some(function_name) = dll_function {
        execute_function(
            process_handle,
            injected_dll_address,
            dll_path,
            function_name,
        )?;
    }

    Ok(())
}

/// Execute a function in an injected DLL
///
/// # Arguments
/// * `process_handle` - Handle to the target process
/// * `injected_dll_address` - Base address of the injected DLL in the target process
/// * `dll_path` - Absolute path to the DLL (used to load it locally to find the function)
/// * `function_name` - Name of the function to execute
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err` with details if any step fails
fn execute_function(
    process_handle: HANDLE,
    injected_dll_address: u32,
    dll_path: &str,
    function_name: &str,
) -> Result<()> {
    let dll_path_cstring = CString::new(dll_path)?;
    let function_name_cstring = CString::new(function_name)?;

    // Load the DLL locally to get the function address
    let library_address = unsafe { LoadLibraryA(PCSTR(dll_path_cstring.as_ptr() as *const u8)) }
        .context("LoadLibraryA failed when loading DLL locally")?;

    // Ensure we free the library
    let _lib_guard = LibraryGuard(library_address);

    // Get the function address
    let function_address = unsafe {
        GetProcAddress(
            library_address,
            PCSTR(function_name_cstring.as_ptr() as *const u8),
        )
    }
    .context(format!(
        "GetProcAddress failed for function '{}'",
        function_name
    ))?;

    // Calculate the offset of the function from the DLL base
    let function_offset = function_address as u64 - library_address.0 as u64;

    // Calculate the address to execute in the target process
    let address_to_execute = injected_dll_address as u64 + function_offset;

    println!(
        "DLL injection debug: function='{}' local_base=0x{:X} func_addr=0x{:X} offset=0x{:X} remote_base=0x{:X} remote_addr=0x{:X}",
        function_name,
        library_address.0 as u64,
        function_address as u64,
        function_offset,
        injected_dll_address,
        address_to_execute
    );

    // Execute the function in the remote process
    execute_at_address(process_handle, address_to_execute as *const ())?;

    Ok(())
}

/// Execute code at a specific address in a remote process
///
/// # Arguments
/// * `process_handle` - Handle to the target process
/// * `address_to_execute` - Address of the code to execute
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err` with details if any step fails
fn execute_at_address(process_handle: HANDLE, address_to_execute: *const ()) -> Result<()> {
    // Create a remote thread at the specified address
    let remote_thread_handle = unsafe {
        CreateRemoteThread(
            process_handle,
            None,
            0,
            Some(std::mem::transmute(address_to_execute)),
            None,
            0,
            None,
        )
    }
    .context("CreateRemoteThread failed for function execution")?;

    // Ensure we close the thread handle
    let _thread_guard = HandleGuard(remote_thread_handle);

    // Wait for the thread to complete
    let wait_result = unsafe { WaitForSingleObject(remote_thread_handle, INFINITE) };

    if wait_result != WAIT_OBJECT_0 {
        return Err(anyhow::anyhow!("WaitForSingleObject failed"));
    }

    // Get the exit code
    let mut exit_code: u32 = 0;
    let success = unsafe { GetExitCodeThread(remote_thread_handle, &mut exit_code) };

    if !success.is_ok() {
        return Err(anyhow::anyhow!(
            "GetExitCodeThread failed for function execution"
        ));
    }

    println!(
        "Remote thread exit code: 0x{:X} (decimal: {})",
        exit_code, exit_code
    );

    // The C# code checks if exit_code != 0, treating non-zero as success
    // This is a bit unusual, but we match that behavior for DecalStartup
    if exit_code == 0 {
        return Err(anyhow::anyhow!("Function execution returned 0 (failure)"));
    }

    Ok(())
}
