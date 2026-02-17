use anyhow::{bail, Context};
use libalembic::{
    launcher::{traits::ClientLauncher, Launcher},
    settings::{Account, ServerInfo, SettingsManager},
};

#[allow(clippy::too_many_arguments)]
pub fn exec_launch(
    mode: String,
    client_path: String,
    hostname: String,
    port: String,
    username: String,
    password: String,
    launcher_path: String,
    wine_prefix: Option<String>,
    env_vars: Vec<(String, String)>,
) -> anyhow::Result<()> {
    use libalembic::client_config::{LaunchCommand, WindowsClientConfig, WineClientConfig};
    use libalembic::settings::ClientConfigType;
    use std::path::PathBuf;

    let client_config = match mode.to_lowercase().as_str() {
        "windows" => ClientConfigType::Windows(WindowsClientConfig {
            name: "CLI-specified Windows client".to_string(),
            client_path: PathBuf::from(&client_path),
            dlls: vec![],
            selected_dll: None,
        }),
        "wine" => {
            let prefix =
                wine_prefix.ok_or_else(|| anyhow::anyhow!("Wine prefix required for wine mode"))?;

            let mut launch_command = LaunchCommand::new(&launcher_path).env("WINEPREFIX", prefix);

            for (key, value) in env_vars {
                launch_command.env.insert(key, value);
            }

            ClientConfigType::Wine(WineClientConfig {
                name: "CLI-specified Wine client".to_string(),
                client_path: PathBuf::from(&client_path),
                launch_command,
                dlls: vec![],
                selected_dll: None,
            })
        }
        _ => bail!(
            "Invalid launch mode '{}'. Must be 'windows' or 'wine'.",
            mode
        ),
    };

    let server_info = ServerInfo {
        name: hostname.clone(),
        hostname,
        port,
    };

    let account_info = Account {
        server_index: 0,
        username,
        password,
    };

    println!("Launch mode: {}", mode);
    println!("Server: {}:{}", server_info.hostname, server_info.port);
    println!("Account: {}", account_info.username);

    // No DLL injection for manual CLI launches (for now)
    run_launcher(client_config, None, server_info, account_info)
}

/// Validate that client and DLL paths exist before launching.
/// For Wine configs, Windows paths are validated by running a check under Wine.
#[allow(dead_code)]
pub fn validate_launch_config(
    client_config: &libalembic::settings::ClientConfigType,
    inject_config: &Option<libalembic::inject_config::InjectConfig>,
) -> anyhow::Result<()> {
    let result = client_config.validate(inject_config.as_ref());

    if result.is_valid {
        Ok(())
    } else {
        bail!(
            "Launch configuration validation failed:\n  - {}",
            result.errors.join("\n  - ")
        )
    }
}

pub fn preset_launch(
    server_name: Option<String>,
    account_name: Option<String>,
) -> anyhow::Result<()> {
    // Get selected client config
    let client_config =
        SettingsManager::get(|s| s.get_selected_client().cloned()).ok_or_else(|| {
            anyhow::anyhow!("No client selected. Use 'alembic client select <index>'")
        })?;

    // Get server (by name override or selected index)
    let server_info = if let Some(name) = server_name {
        SettingsManager::get(|s| s.servers.iter().find(|srv| srv.name == name).cloned())
            .with_context(|| format!("Server '{}' not found in settings", name))?
    } else {
        SettingsManager::get(|s| s.get_selected_server().cloned()).ok_or_else(|| {
            anyhow::anyhow!("No server selected. Use 'alembic server select <index>'")
        })?
    };

    // Get account (by username override or selected index)
    let account_info = if let Some(username) = account_name {
        SettingsManager::get(|s| {
            s.accounts
                .iter()
                .find(|acc| acc.username == username)
                .cloned()
        })
        .with_context(|| format!("Account '{}' not found in settings", username))?
    } else {
        SettingsManager::get(|s| s.get_selected_account().cloned()).ok_or_else(|| {
            anyhow::anyhow!("No account selected. Use 'alembic account select <index>'")
        })?
    };

    // Get selected client index to access its DLLs
    let client_idx = SettingsManager::get(|s| s.selected_client);

    // Get selected DLL for the selected client (optional - if none selected, no injection will occur)
    let inject_config = match client_idx {
        Some(idx) => SettingsManager::get(|s| s.get_client_selected_dll(idx).cloned()),
        None => None,
    };

    // TODO: Validation doesn't support flatpak yet, skip for now
    // validate_launch_config(&client_config, &inject_config)?;

    println!("Client: {}", client_config.name());
    println!(
        "Server: {} ({}:{})",
        server_info.name, server_info.hostname, server_info.port
    );
    println!("Account: {}", account_info.username);
    if let Some(ref dll) = inject_config {
        println!("DLL: {} ({})", dll.dll_type, dll.dll_path.display());
    } else {
        println!("DLL: None (no injection)");
    }

    run_launcher(client_config, inject_config, server_info, account_info)
}

fn run_launcher(
    client_config: libalembic::settings::ClientConfigType,
    inject_config: Option<libalembic::inject_config::InjectConfig>,
    server_info: ServerInfo,
    account_info: Account,
) -> anyhow::Result<()> {
    let mut launcher = Launcher::new(client_config, inject_config, server_info, account_info);

    // Launch the client - stdout/stderr are inherited by the child process
    launcher.launch()?;

    // Wait for user to press Enter to exit (keeps the launcher running)
    println!("\nPress Enter to eject and exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Cleanup launcher
    println!("Ejecting...");
    launcher.eject()?;
    println!("Exited.");

    Ok(())
}
