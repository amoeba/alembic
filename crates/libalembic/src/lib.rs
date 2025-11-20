pub mod acclient;
pub mod async_runtime;
pub mod client_config;
#[cfg(feature = "alembic")]
pub mod inject;
pub mod inject_config;
pub mod injector;
pub mod launcher;
pub mod msg;
pub mod rpc;
pub mod scanner;
pub mod settings;
pub mod util;
pub mod win;

/// Defines how the game client should be launched
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LaunchMode {
    /// Launch using native Windows APIs (Windows only)
    Windows,
    /// Launch using Wine (macOS and Linux)
    Wine,
}
