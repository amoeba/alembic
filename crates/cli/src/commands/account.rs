use anyhow::bail;
use libalembic::settings::{Account, SettingsManager};

pub fn account_add(server: usize, username: String, password: String) -> anyhow::Result<()> {
    let servers = SettingsManager::get(|s| s.servers.clone());

    if server >= servers.len() {
        bail!(
            "Invalid server index {}. Run 'alembic server list' to see available servers.",
            server
        );
    }

    println!("Adding account...");

    let had_no_accounts = SettingsManager::get(|s| s.accounts.is_empty());

    SettingsManager::modify(|settings| {
        settings.accounts.push(Account {
            server_index: server,
            username: username.clone(),
            password: password.clone(),
        });

        // Auto-select if this is the first account
        if had_no_accounts && settings.selected_account.is_none() {
            settings.selected_account = Some(0);
        }
    })?;

    if had_no_accounts {
        println!("✓ Account added and selected!");
    } else {
        println!("✓ Account added!");
    }
    println!();
    println!("  Server:   {}", servers[server].name);
    println!("  Username: {}", username);

    Ok(())
}

pub fn account_list(server_filter: Option<usize>) -> anyhow::Result<()> {
    let accounts = SettingsManager::get(|s| s.accounts.clone());
    let servers = SettingsManager::get(|s| s.servers.clone());
    let selected_account = SettingsManager::get(|s| s.selected_account);

    if servers.is_empty() {
        println!("No servers configured. Use 'server add' to add a server first.");
        return Ok(());
    }

    // Filter accounts if server is specified
    let filtered_accounts: Vec<(usize, &Account)> = accounts
        .iter()
        .enumerate()
        .filter(|(_, account)| {
            if let Some(server) = server_filter {
                account.server_index == server
            } else {
                true
            }
        })
        .collect();

    if filtered_accounts.is_empty() {
        if let Some(server) = server_filter {
            if server >= servers.len() {
                bail!(
                    "Invalid server index {}. Run 'alembic server list' to see available servers.",
                    server
                );
            }
            println!(
                "No accounts configured for server '{}'. Use 'account add' to add an account.",
                servers[server].name
            );
        } else {
            println!("No accounts configured. Use 'account add' to add an account.");
        }
        return Ok(());
    }

    for (index, account) in &filtered_accounts {
        let is_selected = Some(*index) == selected_account;
        let marker = if is_selected { " * " } else { "   " };

        let server_name = if account.server_index < servers.len() {
            &servers[account.server_index].name
        } else {
            "<unknown>"
        };

        println!("{}{}: {}@{}", marker, index, account.username, server_name);
    }

    Ok(())
}

pub fn account_select(index: usize) -> anyhow::Result<()> {
    let accounts = SettingsManager::get(|s| s.accounts.clone());

    if index >= accounts.len() {
        bail!(
            "Invalid account index: {}. Use 'alembic account list' to see available accounts.",
            index
        );
    }

    let username = accounts[index].username.clone();

    SettingsManager::modify(|settings| {
        settings.selected_account = Some(index);
    })?;

    println!("✓ Selected account: {}", username);

    Ok(())
}

pub fn account_reset() -> anyhow::Result<()> {
    let was_selected = SettingsManager::get(|s| s.selected_account.is_some());

    SettingsManager::modify(|settings| {
        settings.selected_account = None;
    })?;

    if was_selected {
        println!("✓ Account selection cleared");
    } else {
        println!("No account was selected");
    }

    Ok(())
}

pub fn account_remove(index: usize) -> anyhow::Result<()> {
    let accounts = SettingsManager::get(|s| s.accounts.clone());

    if index >= accounts.len() {
        bail!(
            "Invalid index {}. No account exists at that index. Run 'alembic account list' to see available accounts.",
            index
        );
    }

    let username = accounts[index].username.clone();

    println!("Removing account '{}'...", username);

    SettingsManager::modify(|settings| {
        settings.accounts.remove(index);

        // Update selected_account if it was this one or needs adjustment
        if let Some(selected) = settings.selected_account {
            if selected == index {
                settings.selected_account = None;
            } else if selected > index {
                settings.selected_account = Some(selected - 1);
            }
        }
    })?;

    println!("✓ Account removed!");

    Ok(())
}

pub fn account_edit(
    index: usize,
    server: Option<usize>,
    username: Option<String>,
    password: Option<String>,
) -> anyhow::Result<()> {
    let accounts = SettingsManager::get(|s| s.accounts.clone());

    if index >= accounts.len() {
        bail!(
            "Invalid index {}. No account exists at that index. Run 'alembic account list' to see available accounts.",
            index
        );
    }

    if server.is_none() && username.is_none() && password.is_none() {
        println!(
            "No changes specified. Use --server, --username, or --password to modify the account."
        );
        return Ok(());
    }

    println!("Editing account at index {}...", index);

    SettingsManager::modify(|settings| {
        let account = &mut settings.accounts[index];

        if let Some(s) = server {
            account.server_index = s;
            println!("  Updated server index to: {}", s);
        }
        if let Some(u) = username {
            println!("  Updated username to: {}", u);
            account.username = u;
        }
        if let Some(p) = password {
            println!("  Updated password");
            account.password = p;
        }
    })?;

    println!("✓ Account updated!");

    Ok(())
}
