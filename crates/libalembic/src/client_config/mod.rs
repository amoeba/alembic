mod traits;
mod windows;
mod wine;

pub use traits::{ClientConfig, LaunchCommand, windows_path_parent};
pub use windows::WindowsClientConfig;
pub use wine::WineClientConfig;
