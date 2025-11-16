#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub fn try_launch(
    client_config: &Option<libalembic::client_config::ClientConfig>,
    server_info: &Option<libalembic::settings::ServerInfo>,
    account_info: &Option<libalembic::settings::Account>,
    inject_config: Option<libalembic::client_config::InjectConfig>,
) -> anyhow::Result<std::num::NonZero<u32>> {
    use anyhow::bail;
    use libalembic::launch::Launcher;

    // Validate arguments
    let client_config = match client_config {
        Some(config) => config.clone(),
        None => bail!("No client selected."),
    };

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
        inject_config,
        server_info,
        account_info,
    );
    let pid = launcher.find_or_launch()?;
    launcher.inject()?;
    // TODO: How to handle deinject. i.e., store the launch app-wide

    Ok(pid)
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub fn try_launch(
    _client_config: &Option<libalembic::client_config::ClientConfig>,
    _server_info: &Option<libalembic::settings::ServerInfo>,
    _account_info: &Option<libalembic::settings::Account>,
    _inject_config: Option<libalembic::client_config::InjectConfig>,
) -> anyhow::Result<std::num::NonZero<u32>> {
    // TODO: Show some indication we can't launch on this platform

    use std::num::NonZero;

    Ok(NonZero::new(0).unwrap())
}
