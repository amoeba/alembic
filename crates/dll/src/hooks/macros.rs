/// Macro for defining address-based function hooks.
///
/// # Example
/// ```ignore
/// define_hook! {
///     name: AddTextToScroll,
///     address: 0x005649F0,
///     convention: thiscall,
///     args: (This: *mut c_void, text: *mut c_void, a: u32, b: u8, c: u32),
///     ret: i32,
///     body: |this, text, a, b, c| {
///         // Custom logic here
///         println!("Hooked!");
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_hook {
    (
        name: $name:ident,
        address: $address:expr,
        convention: thiscall,
        args: ($($arg:ident : $arg_ty:ty),* $(,)?),
        ret: $ret:ty,
        body: |$($body_arg:ident),* $(,)?| $body:expr
    ) => {
        paste::paste! {
            type [<fn_ $name _Impl>] = extern "thiscall" fn($($arg_ty),*) -> $ret;

            extern "thiscall" fn [<Hook_ $name _Impl>]($($arg : $arg_ty),*) -> $ret {
                // Bind arguments for the body closure
                $(let $body_arg = $arg;)*
                let _ = (|| -> Result<(), Box<dyn std::error::Error>> {
                    $body
                    Ok(())
                })();

                [<Hook_ $name>].call($($arg),*)
            }

            pub static [<Hook_ $name>]: once_cell::sync::Lazy<
                retour::GenericDetour<[<fn_ $name _Impl>]>,
            > = once_cell::sync::Lazy::new(|| {
                let address: isize = $address as isize;
                let ori: [<fn_ $name _Impl>] = unsafe { std::mem::transmute(address) };
                unsafe { retour::GenericDetour::new(ori, [<Hook_ $name _Impl>]).unwrap() }
            });
        }
    };

    (
        name: $name:ident,
        address: $address:expr,
        convention: system,
        args: ($($arg:ident : $arg_ty:ty),* $(,)?),
        ret: $ret:ty,
        body: |$($body_arg:ident),* $(,)?| $body:expr
    ) => {
        paste::paste! {
            type [<fn_ $name _Impl>] = extern "system" fn($($arg_ty),*) -> $ret;

            extern "system" fn [<Hook_ $name _Impl>]($($arg : $arg_ty),*) -> $ret {
                $(let $body_arg = $arg;)*
                let _ = (|| -> Result<(), Box<dyn std::error::Error>> {
                    $body
                    Ok(())
                })();

                [<Hook_ $name>].call($($arg),*)
            }

            pub static [<Hook_ $name>]: once_cell::sync::Lazy<
                retour::GenericDetour<[<fn_ $name _Impl>]>,
            > = once_cell::sync::Lazy::new(|| {
                let address: isize = $address as isize;
                let ori: [<fn_ $name _Impl>] = unsafe { std::mem::transmute(address) };
                unsafe { retour::GenericDetour::new(ori, [<Hook_ $name _Impl>]).unwrap() }
            });
        }
    };
}

/// Macro for defining DLL import hooks (e.g., hooking wsock32.dll functions).
///
/// # Example
/// ```ignore
/// define_dll_hook! {
///     name: Network_SendTo,
///     dll: b"wsock32.dll\0",
///     proc: b"sendto\0",
///     convention: system,
///     args: (s: *mut c_void, buf: *mut u8, len: i32, flags: i32, to: *mut u8, tolen: *mut i32),
///     ret: i32,
///     body: |result, s, buf, len, flags, to, tolen| {
///         // result is the return value from calling the original
///         if result > 0 {
///             println!("Sent {} bytes", result);
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_dll_hook {
    (
        name: $name:ident,
        dll: $dll:expr,
        proc: $proc:expr,
        convention: system,
        args: ($($arg:ident : $arg_ty:ty),* $(,)?),
        ret: $ret:ty,
        body: |$result:ident, $($body_arg:ident),* $(,)?| $body:expr
    ) => {
        paste::paste! {
            type [<fn_ $name _Impl>] = extern "system" fn($($arg_ty),*) -> $ret;

            pub static [<Hook_ $name>]: once_cell::sync::Lazy<
                retour::GenericDetour<[<fn_ $name _Impl>]>,
            > = once_cell::sync::Lazy::new(|| {
                use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
                use windows::core::PCSTR;

                let library_handle = unsafe { LoadLibraryA(PCSTR($dll.as_ptr() as _)) }.unwrap();
                let address = unsafe { GetProcAddress(library_handle, PCSTR($proc.as_ptr() as _)) };
                let ori: [<fn_ $name _Impl>] = unsafe { std::mem::transmute(address) };

                unsafe { retour::GenericDetour::new(ori, [<Hook_ $name _Impl>]).unwrap() }
            });

            extern "system" fn [<Hook_ $name _Impl>]($($arg : $arg_ty),*) -> $ret {
                let $result = [<Hook_ $name>].call($($arg),*);

                $(let $body_arg = $arg;)*
                let _ = std::panic::catch_unwind(|| {
                    $body
                });

                $result
            }
        }
    };
}

/// Macro for registering hooks. Generates `attach_hooks()` and `detach_hooks()` functions.
///
/// # Example
/// ```ignore
/// register_hooks!(
///     hooks::chat::Hook_AddTextToScroll,
///     hooks::net::Hook_Network_SendTo,
///     hooks::net::Hook_Network_RecvFrom,
/// );
/// ```
#[macro_export]
macro_rules! register_hooks {
    ($($hook:path),* $(,)?) => {
        fn attach_hooks() -> anyhow::Result<()> {
            $(
                unsafe { $hook.enable()? };
            )*
            Ok(())
        }

        fn detach_hooks() -> anyhow::Result<()> {
            $(
                unsafe { $hook.disable()? };
            )*
            Ok(())
        }
    };
}
