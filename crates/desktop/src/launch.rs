#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub fn try_launch(
    client_config: &Option<libalembic::client_config::ClientConfig>,
    server_info: &Option<libalembic::settings::ServerInfo>,
    account_info: &Option<libalembic::settings::Account>,
    inject_config: &Option<libalembic::client_config::InjectConfig>,
) -> anyhow::Result<std::num::NonZero<u32>> {
    use anyhow::bail;
    use libalembic::launcher::{traits::ClientLauncher, Launcher};

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
        inject_config.clone(),
        server_info,
        account_info,
    );
    let pid = launcher.launch()?;
    // TODO: How to handle deinject. i.e., store the launch app-wide

    Ok(pid)
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub fn try_launch(
    client_config: &Option<libalembic::client_config::ClientConfig>,
    server_info: &Option<libalembic::settings::ServerInfo>,
    account_info: &Option<libalembic::settings::Account>,
    inject_config: &Option<libalembic::client_config::InjectConfig>,
) -> anyhow::Result<std::num::NonZero<u32>> {
    use anyhow::bail;
    use libalembic::launcher::{traits::ClientLauncher, Launcher};

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
        inject_config.clone(),
        server_info,
        account_info,
    );
    let pid = launcher.launch()?;
    // TODO: How to handle deinject. i.e., store the launch app-wide

    Ok(pid)
}
