pub mod launcher;
pub mod noop;

#[cfg(target_os = "windows")]
pub mod windows;

pub mod wine;
