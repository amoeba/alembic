use std::num::NonZero;

use crate::{
    inject_config::InjectConfig,
    settings::{Account, ClientConfigType, ServerInfo},
};

/// Trait for platform-specific client launcher implementations
pub trait ClientLauncher: std::any::Any {
    /// Create a new launcher
    fn new(
        client_config: ClientConfigType,
        inject_config: Option<InjectConfig>,
        server_info: ServerInfo,
        account_info: Account,
    ) -> Self;

    /// Launch a new client process (with automatic injection if configured)
    fn launch(&mut self) -> Result<NonZero<u32>, std::io::Error>;

    /// Find or launch the client process (tries to find existing first, with automatic injection if configured)
    fn find_or_launch(&mut self) -> Result<NonZero<u32>, std::io::Error>;

    /// Inject a DLL into the running client
    fn inject(&mut self) -> Result<(), anyhow::Error>;

    /// Eject the injected DLL
    fn eject(&mut self) -> Result<(), anyhow::Error>;
}
