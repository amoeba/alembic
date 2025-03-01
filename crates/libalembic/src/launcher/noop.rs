use std::error::Error;

use crate::settings::{AccountInfo, ClientInfo, DllInfo, ServerInfo};

use super::launcher::{LaunchResult, Launcher};

/// NoopLauncher
///
/// Launcher implementation that doesn't do anything. Used only on platforms
/// that are unsupported.
#[derive(Debug)]
pub struct NoopLauncher {}

impl Launcher for NoopLauncher {
    fn new(
        _client_info: ClientInfo,
        _server_info: ServerInfo,
        _account_info: AccountInfo,
        _dll_info: DllInfo,
    ) -> Self {
        NoopLauncher {}
    }

    fn launch(&self) -> Result<LaunchResult, Box<dyn Error>> {
        todo!();
    }
}
