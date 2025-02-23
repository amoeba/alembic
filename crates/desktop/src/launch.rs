#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub fn try_launch(
    client_info: &Option<libalembic::settings::ClientInfo>,
    server_info: &Option<libalembic::settings::ServerInfo>,
    account_info: &Option<libalembic::settings::AccountInfo>,
    dll_info: Option<libalembic::settings::DllInfo>,
) -> anyhow::Result<std::num::NonZero<u32>> {
    use std::fs;

    use anyhow::bail;
    use libalembic::launcher::{launcher::Launcher, windows::WindowsLauncher};

    // Validate arguments and return an error if any checks fail
    match client_info {
        Some(value) => match fs::exists(&value.path) {
            Ok(exists) => {
                if !exists {
                    bail!("Client does not exist at path '{}'.", value.path);
                }
            }
            Err(_) => bail!("Couldn't determine whether client exists or not."),
        },
        None => bail!("Couldn't get client information."),
    }

    match server_info {
        Some(_) => {}
        None => bail!("Couldn't get server information."),
    }

    match account_info {
        Some(_) => {}
        None => bail!("Couldn't get account information."),
    }

    match &dll_info {
        Some(info) => match fs::exists(&info.dll_path) {
            Ok(exists) => {
                if !exists {
                    bail!("Alembic DLL does not exist at path '{}'.", info.dll_path);
                }
            }
            Err(_) => bail!("Couldn't determine Alembic DLL exists or not."),
        },
        None => bail!("Couldn't get DLL information."),
    }

    let mut launcher = WindowsLauncher::new(
        client_info.clone().unwrap(),
        server_info.clone().unwrap(),
        account_info.clone().unwrap(),
        dll_info.clone().unwrap(),
    );
    let pid = launcher.find_or_launch()?;
    launcher.inject()?;
    // TODO: How to handle deinject. i.e., store the launch app-wide

    Ok(pid)
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub fn try_launch(
    client_info: &Option<libalembic::settings::ClientInfo>,
    server: &Option<libalembic::settings::ServerInfo>,
    account_info: &Option<libalembic::settings::AccountInfo>,
    dll_path: Option<String>,
) -> anyhow::Result<std::num::NonZero<u32>> {
    // TODO: Show some indication we can't launch on this platform

    use std::num::NonZero;

    Ok(NonZero::new(0).unwrap())
}
