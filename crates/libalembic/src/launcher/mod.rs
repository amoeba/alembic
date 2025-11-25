pub mod launcher;
pub mod noop;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub mod windows;

pub mod wine;
