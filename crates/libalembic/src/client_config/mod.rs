mod traits;
mod windows;
mod wine;

pub use traits::{windows_path_parent, ClientConfig, LaunchCommand};
pub use windows::WindowsClientConfig;
pub use wine::WineClientConfig;
