pub fn try_launch(
    client_config: &Option<libalembic::settings::ClientConfigType>,
    server_info: &Option<libalembic::settings::ServerInfo>,
    account_info: &Option<libalembic::settings::Account>,
    inject_config: &Option<libalembic::inject_config::InjectConfig>,
) -> anyhow::Result<std::num::NonZero<u32>> {
    use anyhow::bail;
    use libalembic::launcher::{Launcher, traits::ClientLauncher};

    // Validate arguments
    let client_config = match client_config {
        Some(config) => config.clone(),
        None => bail!("No client selected."),
    };

    // Check if client type is supported on this platform
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    if client_config.is_wine() {
        bail!(
            "Wine client configuration is not supported on Windows. Please use a Windows client configuration."
        );
    }
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    if !client_config.is_wine() {
        bail!(
            "Windows client configuration is not supported on this platform. Please use a Wine client configuration."
        );
    }

    let server_info = match server_info {
        Some(info) => info.clone(),
        None => bail!("No server selected."),
    };

    let account_info = match account_info {
        Some(info) => info.clone(),
        None => bail!("No account selected."),
    };

    let mut launcher = Launcher::new(
        client_config,
        inject_config.clone(),
        server_info,
        account_info,
    );
    let pid = launcher.launch()?;
    // TODO: How to handle deinject. i.e., store the launch app-wide

    Ok(pid)
}
