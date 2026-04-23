use anyhow::bail;
use libalembic::settings::{ServerInfo, SettingsManager};

pub fn server_add(name: String, hostname: String, port: String) -> anyhow::Result<()> {
    println!("Adding server...");

    let had_no_servers = SettingsManager::get(|s| s.servers.is_empty());

    SettingsManager::modify(|settings| {
        settings.servers.push(ServerInfo {
            name: name.clone(),
            hostname: hostname.clone(),
            port: port.clone(),
        });

        // Auto-select if this is the first server
        if had_no_servers && settings.selected_server.is_none() {
            settings.selected_server = Some(0);
        }
    })?;

    if had_no_servers {
        println!("✓ Server added and selected!");
    } else {
        println!("✓ Server added!");
    }
    println!();
    println!("  Name:     {}", name);
    println!("  Hostname: {}", hostname);
    println!("  Port:     {}", port);

    Ok(())
}

pub fn server_list() -> anyhow::Result<()> {
    let servers = SettingsManager::get(|s| s.servers.clone());
    let selected_server = SettingsManager::get(|s| s.selected_server);

    if servers.is_empty() {
        println!("No servers configured. Use 'server add' to add a server.");
        return Ok(());
    }

    for (index, server) in servers.iter().enumerate() {
        let is_selected = Some(index) == selected_server;
        let marker = if is_selected { " * " } else { "   " };
        println!("{}{}: {}", marker, index, server.name);
    }

    Ok(())
}

pub fn server_select(index: usize) -> anyhow::Result<()> {
    let servers = SettingsManager::get(|s| s.servers.clone());

    if index >= servers.len() {
        bail!(
            "Invalid server index: {}. Use 'alembic server list' to see available servers.",
            index
        );
    }

    let server_name = servers[index].name.clone();

    SettingsManager::modify(|settings| {
        settings.selected_server = Some(index);
    })?;

    println!("✓ Selected server: {}", server_name);

    Ok(())
}

pub fn server_reset() -> anyhow::Result<()> {
    let was_selected = SettingsManager::get(|s| s.selected_server.is_some());

    SettingsManager::modify(|settings| {
        settings.selected_server = None;
    })?;

    if was_selected {
        println!("✓ Server selection cleared");
    } else {
        println!("No server was selected");
    }

    Ok(())
}

pub fn server_remove(index: usize) -> anyhow::Result<()> {
    let servers = SettingsManager::get(|s| s.servers.clone());

    if index >= servers.len() {
        bail!(
            "Invalid index {}. No server exists at that index. Run 'alembic server list' to see available servers.",
            index
        );
    }

    let server_name = servers[index].name.clone();

    println!("Removing server '{}'...", server_name);

    SettingsManager::modify(|settings| {
        settings.servers.remove(index);

        // Update selected_server if it was this one or needs adjustment
        if let Some(selected) = settings.selected_server {
            if selected == index {
                settings.selected_server = None;
            } else if selected > index {
                settings.selected_server = Some(selected - 1);
            }
        }

        // Remove accounts that were associated with the deleted server
        settings.accounts.retain(|a| a.server_index != index);

        // Update account server indices (after removing deleted accounts)
        for account in &mut settings.accounts {
            if account.server_index > index {
                account.server_index -= 1;
            }
        }
    })?;

    println!("✓ Server removed!");

    Ok(())
}

pub fn server_edit(
    index: usize,
    name: Option<String>,
    hostname: Option<String>,
    port: Option<String>,
) -> anyhow::Result<()> {
    let servers = SettingsManager::get(|s| s.servers.clone());

    if index >= servers.len() {
        bail!(
            "Invalid index {}. No server exists at that index. Run 'alembic server list' to see available servers.",
            index
        );
    }

    if name.is_none() && hostname.is_none() && port.is_none() {
        println!("No changes specified. Use --name, --hostname, or --port to modify the server.");
        return Ok(());
    }

    println!("Editing server at index {}...", index);

    SettingsManager::modify(|settings| {
        let server = &mut settings.servers[index];

        if let Some(n) = name {
            println!("  Updated name to: {}", n);
            server.name = n;
        }
        if let Some(h) = hostname {
            println!("  Updated hostname to: {}", h);
            server.hostname = h;
        }
        if let Some(p) = port {
            println!("  Updated port to: {}", p);
            server.port = p;
        }
    })?;

    println!("✓ Server updated!");

    Ok(())
}
