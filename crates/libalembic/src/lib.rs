pub mod acclient;
pub mod async_runtime;
pub mod inject;
pub mod launch;
pub mod msg;
pub mod rpc;
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
