// Platform-specific launcher implementations
#[cfg(all(target_os = "windows", target_env = "msvc", feature = "alembic"))]
pub mod windows;

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub mod wine;

// Common trait and types
mod traits;

pub use traits::ClientLauncher;

/// Platform-specific launcher type alias
/// On Windows with alembic feature: WindowsLauncherImpl
/// On other platforms: WineLauncherImpl
#[cfg(all(target_os = "windows", target_env = "msvc", feature = "alembic"))]
pub type Launcher = windows::WindowsLauncherImpl;

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub type Launcher = wine::WineLauncherImpl;
