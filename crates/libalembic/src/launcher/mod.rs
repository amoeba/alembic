pub mod traits;

#[cfg(all(target_os = "windows", target_env = "msvc", feature = "alembic"))]
pub mod windows;

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub mod wine;

#[cfg(all(target_os = "windows", target_env = "msvc", feature = "alembic"))]
pub type Launcher = windows::WindowsLauncherImpl;

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub type Launcher = wine::WineLauncherImpl;
