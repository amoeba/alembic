use std::path::Path;

/// Common interface for client configurations
pub trait ClientConfiguration {
    fn display_name(&self) -> &str;
    fn install_path(&self) -> &Path;
}
