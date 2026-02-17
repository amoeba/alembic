use anyhow::bail;
use libalembic::settings::SettingsManager;

pub fn client_list() -> anyhow::Result<()> {
    let clients = SettingsManager::get(|s| s.clients.clone());
    let selected_client = SettingsManager::get(|s| s.selected_client);

    if clients.is_empty() {
        println!("No clients configured.");
        println!("Run 'alembic client scan' to discover clients.");
        return Ok(());
    }

    for (idx, config) in clients.iter().enumerate() {
        let is_selected = Some(idx) == selected_client;
        let marker = if is_selected { " * " } else { "   " };
        let client_type = if config.is_wine() { "Wine" } else { "Windows" };
        println!("{}{}: {} ({})", marker, idx, config.name(), client_type);
    }

    Ok(())
}

pub fn client_show(index: usize) -> anyhow::Result<()> {
    let client_config =
        SettingsManager::get(|s| s.clients.get(index).cloned()).ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid client index: {}. Use 'alembic client list' to see available clients.",
                index
            )
        })?;

    println!("Client configuration (index {}):", index);
    println!();
    println!("{}", client_config);

    let dlls = client_config.dlls();
    let selected_dll = client_config.selected_dll();

    if dlls.is_empty() {
        println!("DLLs: (none)");
    } else {
        println!("DLLs:");
        for (i, dll) in dlls.iter().enumerate() {
            let marker = if selected_dll == Some(i) { " *" } else { "  " };
            println!(
                "{}  [{}] {} ({})",
                marker,
                i,
                dll.dll_path.display(),
                dll.dll_type
            );
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn client_edit(
    index: usize,
    name: Option<String>,
    client_path: Option<String>,
    wrapper_program: Option<String>,
    args: Vec<String>,
    clear_args: bool,
    env_vars: Vec<(String, String)>,
    unset_env_vars: Vec<String>,
) -> anyhow::Result<()> {
    use libalembic::settings::ClientConfigType;
    use std::path::PathBuf;

    let client_exists = SettingsManager::get(|s| s.clients.get(index).is_some());
    if !client_exists {
        bail!(
            "Invalid client index: {}. Use 'alembic client list' to see available clients.",
            index
        );
    }

    if name.is_none()
        && client_path.is_none()
        && wrapper_program.is_none()
        && args.is_empty()
        && !clear_args
        && env_vars.is_empty()
        && unset_env_vars.is_empty()
    {
        println!("No changes specified. Use --name, --client-path, --wrapper-program, --arg, --clear-args, --env, or --unset-env to modify the client.");
        return Ok(());
    }

    println!("Editing client at index {}...", index);

    SettingsManager::modify(|settings| {
        let client = &mut settings.clients[index];

        match client {
            ClientConfigType::Windows(c) => {
                if let Some(n) = &name {
                    println!("  Updated name to: {}", n);
                    c.name = n.clone();
                }
                if let Some(p) = &client_path {
                    println!("  Updated client path to: {}", p);
                    c.client_path = PathBuf::from(p);
                }
            }
            ClientConfigType::Wine(c) => {
                if let Some(n) = &name {
                    println!("  Updated name to: {}", n);
                    c.name = n.clone();
                }
                if let Some(p) = &client_path {
                    println!("  Updated client path to: {}", p);
                    c.client_path = PathBuf::from(p);
                }
                if let Some(w) = &wrapper_program {
                    println!("  Updated program to: {}", w);
                    c.launch_command.program = PathBuf::from(w);
                }
                if clear_args {
                    println!("  Cleared all args");
                    c.launch_command.args.clear();
                }
                for arg in &args {
                    println!("  Added arg: {}", arg);
                    c.launch_command.args.push(arg.clone());
                }
                for key in &unset_env_vars {
                    if c.launch_command.env.remove(key).is_some() {
                        println!("  Removed env var: {}", key);
                    }
                }
                for (key, value) in &env_vars {
                    println!("  Set env var: {}={}", key, value);
                    c.launch_command.env.insert(key.clone(), value.clone());
                }
            }
        }
    })?;

    println!("✓ Client updated!");

    Ok(())
}

pub fn client_add(
    mode: String,
    client_path: String,
    launcher_path: String,
    wine_prefix: Option<String>,
    env_vars: Vec<(String, String)>,
) -> anyhow::Result<()> {
    use libalembic::client_config::{LaunchCommand, WindowsClientConfig, WineClientConfig};
    use libalembic::settings::ClientConfigType;
    use std::path::PathBuf;

    println!("Adding client configuration...");

    let client_config = match mode.to_lowercase().as_str() {
        "windows" => ClientConfigType::Windows(WindowsClientConfig {
            name: "Manual Windows client".to_string(),
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
                name: "Manual Wine client".to_string(),
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

    let new_index = SettingsManager::get(|s| s.clients.len());

    SettingsManager::modify(|settings| {
        settings.add_client(client_config, true);
        settings.is_configured = true;
    })?;

    println!("✓ Client configuration saved at index {}!", new_index);
    println!();

    // Show what was configured
    client_show(new_index)
}

pub fn client_select(index: usize) -> anyhow::Result<()> {
    let clients = SettingsManager::get(|s| s.clients.clone());

    if index >= clients.len() {
        bail!(
            "Invalid client index: {}. Use 'alembic client list' to see available clients.",
            index
        );
    }

    let client_name = clients[index].name().to_string();

    SettingsManager::modify(|settings| {
        settings.selected_client = Some(index);
    })?;

    println!("✓ Selected client: {}", client_name);

    Ok(())
}

pub fn client_reset() -> anyhow::Result<()> {
    let was_selected = SettingsManager::get(|s| s.selected_client.is_some());

    SettingsManager::modify(|settings| {
        settings.selected_client = None;
    })?;

    if was_selected {
        println!("✓ Client selection cleared");
    } else {
        println!("No client was selected");
    }

    Ok(())
}

pub fn client_remove(index: usize) -> anyhow::Result<()> {
    let removed = SettingsManager::get(|s| {
        if index < s.clients.len() {
            Some(s.clients[index].name().to_string())
        } else {
            None
        }
    })
    .ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid client index: {}. Use 'alembic client list' to see available clients.",
            index
        )
    })?;

    SettingsManager::modify(|settings| {
        settings.remove_client(index);
        settings.is_configured = !settings.clients.is_empty();
    })?;

    println!("✓ Removed client: {}", removed);

    Ok(())
}
