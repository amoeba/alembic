use std::error::Error;

use crate::settings::{AccountInfo, ClientInfo, DllInfo, ServerInfo};

use super::launcher::{LaunchResult, Launcher};

/// WineLauncher
///
/// Launcher implementation that can launch a client using Wine. Does not
/// support injection.
#[derive(Debug)]
pub struct WineLauncher {}

impl Launcher for WineLauncher {
    fn new(
        _client_info: ClientInfo,
        _server_info: ServerInfo,
        _account_info: AccountInfo,
        _dll_info: DllInfo,
    ) -> Self {
        WineLauncher {}
    }

    fn launch(&self) -> Result<LaunchResult, Box<dyn Error>> {
        todo!();
    }
}
