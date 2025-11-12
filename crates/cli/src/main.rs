use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use libalembic::{
    client_config::ClientConfig,
    launch::Launcher,
    settings::{Account, ServerInfo, SettingsManager},
};
use std::sync::mpsc::channel;

mod scanner;

#[cfg(debug_assertions)]
const VERSION: &str = env!("DEBUG_VERSION");

#[cfg(not(debug_assertions))]
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(debug_assertions)]
const ABOUT: &str = concat!("Alembic ", env!("DEBUG_VERSION"));

#[cfg(not(debug_assertions))]
const ABOUT: &str = concat!("Alembic ", env!("CARGO_PKG_VERSION"));

#[derive(Parser)]
#[command(name = "alembic")]
#[command(about = ABOUT)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage accounts
    Account {
        #[command(subcommand)]
        command: AccountCommands,
    },

    /// Manage AC client installations
    Client {
        #[command(subcommand)]
        command: ClientCommands,
    },

    /// Execute launch with all parameters specified via command line
    Exec {
        /// Launch mode (windows or wine)
        #[arg(long)]
        mode: String,

        /// Path to game client directory (e.g., "C:\\AC")
        #[arg(long)]
        client_path: String,

        /// Server hostname
        #[arg(long)]
        hostname: String,

        /// Server port
        #[arg(long)]
        port: String,

        /// Account username
        #[arg(long)]
        username: String,

        /// Account password
        #[arg(long)]
        password: String,

        /// Launcher path (DLL for Windows, wine64 executable for Wine)
        #[arg(long)]
        launcher_path: String,

        /// Wine prefix path (Wine mode only)
        #[arg(long)]
        wine_prefix: Option<String>,

        /// Environment variables (format: KEY=VALUE, can be specified multiple times)
        #[arg(long = "env", value_parser = parse_key_val)]
        env_vars: Vec<(String, String)>,
    },

    /// Launch using saved settings with optional overrides
    Launch {
        /// Server name to use (overrides selected server in settings)
        #[arg(long)]
        server: Option<String>,

        /// Account username to use (overrides selected account in settings)
        #[arg(long)]
        account: Option<String>,
    },

    /// Manage servers
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
}

#[derive(Subcommand)]
enum AccountCommands {
    /// Add a new account
    Add {
        /// Server index (from 'server list')
        #[arg(long)]
        server: usize,

        /// Account username
        #[arg(long)]
        username: String,

        /// Account password
        #[arg(long)]
        password: String,
    },

    /// List accounts
    List {
        /// Filter by server index (optional)
        #[arg(long)]
        server: Option<usize>,
    },

    /// Remove an account by index
    Remove {
        /// Index of the account to remove (from 'account list')
        index: usize,
    },
}

#[derive(Subcommand)]
enum ClientCommands {
    /// Add a new client installation
    Add {
        /// Launch mode (windows or wine)
        #[arg(long)]
        mode: String,

        /// Path to game client directory (e.g., "C:\\AC")
        #[arg(long)]
        client_path: String,

        /// Launcher path (DLL for Windows, wine64 executable for Wine)
        #[arg(long)]
        launcher_path: String,

        /// Wine prefix path (Wine mode only)
        #[arg(long)]
        wine_prefix: Option<String>,

        /// Environment variables (format: KEY=VALUE, can be specified multiple times)
        #[arg(long = "env", value_parser = parse_key_val)]
        env_vars: Vec<(String, String)>,
    },

    /// List configured clients (brief)
    List,

    /// Select a client by index
    Select {
        /// Index of the client to select (from 'client list')
        index: usize,
    },

    /// Remove a client by index
    Remove {
        /// Index of the client to remove (from 'client list')
        index: usize,
    },

    /// Show detailed client configuration
    Show {
        /// Index of the client to show (from 'client list')
        index: usize,
    },

    /// Scan for installed clients and wine prefixes
    Scan,
}

#[derive(Subcommand)]
enum ServerCommands {
    /// Add a new server
    Add {
        /// Server name
        #[arg(long)]
        name: String,

        /// Server hostname or IP address
        #[arg(long)]
        hostname: String,

        /// Server port
        #[arg(long)]
        port: String,
    },

    /// List servers
    List,

    /// Remove a server by index
    Remove {
        /// Index of the server to remove (from 'server list')
        index: usize,
    },
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Account { command } => match command {
            AccountCommands::Add {
                server,
                username,
                password,
            } => account_add(server, username, password),
            AccountCommands::List { server } => account_list(server),
            AccountCommands::Remove { index } => account_remove(index),
        },
        Commands::Client { command } => match command {
            ClientCommands::Add {
                mode,
                client_path,
                launcher_path,
                wine_prefix,
                env_vars,
            } => client_add(mode, client_path, launcher_path, wine_prefix, env_vars),
            ClientCommands::List => client_list(),
            ClientCommands::Select { index } => client_select(index),
            ClientCommands::Remove { index } => client_remove(index),
            ClientCommands::Show { index } => client_show(index),
            ClientCommands::Scan => client_scan(),
        },
        Commands::Exec {
            mode,
            client_path,
            hostname,
            port,
            username,
            password,
            launcher_path,
            wine_prefix,
            env_vars,
        } => exec_launch(
            mode,
            client_path,
            hostname,
            port,
            username,
            password,
            launcher_path,
            wine_prefix,
            env_vars,
        ),
        Commands::Launch { server, account } => preset_launch(server, account),
        Commands::Server { command } => match command {
            ServerCommands::Add {
                name,
                hostname,
                port,
            } => server_add(name, hostname, port),
            ServerCommands::List => server_list(),
            ServerCommands::Remove { index } => server_remove(index),
        },
    }
}

fn exec_launch(
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
    use libalembic::client_config::{WindowsClientConfig, WineClientConfig};
    use std::collections::HashMap;
    use std::path::PathBuf;

    let client_config = match mode.to_lowercase().as_str() {
        "windows" => ClientConfig::Windows(WindowsClientConfig {
            display_name: "CLI-specified Windows client".to_string(),
            install_path: PathBuf::from(&client_path),
            dll_path: PathBuf::from(&launcher_path),
        }),
        "wine" => {
            let prefix = wine_prefix.ok_or_else(|| anyhow::anyhow!("Wine prefix required for wine mode"))?;
            let mut additional_env = HashMap::new();
            for (key, value) in env_vars {
                additional_env.insert(key, value);
            }

            ClientConfig::Wine(WineClientConfig {
                display_name: "CLI-specified Wine client".to_string(),
                install_path: PathBuf::from(&client_path),
                wine_executable: PathBuf::from(&launcher_path),
                prefix_path: PathBuf::from(&prefix),
                additional_env,
            })
        }
        _ => bail!("Invalid launch mode '{}'. Must be 'windows' or 'wine'.", mode),
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

    run_launcher(client_config, server_info, account_info)
}

fn preset_launch(server_name: Option<String>, account_name: Option<String>) -> anyhow::Result<()> {
    // Get selected client config
    let client_config = SettingsManager::get(|s| {
        s.get_selected_client().cloned()
    }).ok_or_else(|| anyhow::anyhow!("No client selected. Use 'alembic client select <index>'"))?;

    // Get server (by name override or selected index)
    let server_info = if let Some(name) = server_name {
        SettingsManager::get(|s| {
            s.servers
                .iter()
                .find(|srv| srv.name == name)
                .cloned()
        })
        .with_context(|| format!("Server '{}' not found in settings", name))?
    } else {
        SettingsManager::get(|s| s.get_selected_server().cloned())
            .ok_or_else(|| anyhow::anyhow!("No server selected. Use 'alembic server select <index>'"))?
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
        SettingsManager::get(|s| s.get_selected_account().cloned())
            .ok_or_else(|| anyhow::anyhow!("No account selected. Use 'alembic account select <index>'"))?
    };

    println!("Client: {}", client_config.display_name());
    println!(
        "Server: {} ({}:{})",
        server_info.name, server_info.hostname, server_info.port
    );
    println!("Account: {}", account_info.username);

    run_launcher(client_config, server_info, account_info)
}

fn run_launcher(
    client_config: ClientConfig,
    server_info: ServerInfo,
    account_info: Account,
) -> anyhow::Result<()> {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    let mut launcher = Launcher::new(
        client_config,
        server_info,
        account_info,
    );

    launcher.find_or_launch()?;
    launcher.inject()?;

    println!("Game launched! Press Ctrl+C to exit.");

    // Block until Ctrl+C
    rx.recv().expect("Could not receive from channel.");
    println!("Ctrl+C received...");

    launcher.eject()?;

    println!("Exiting.");

    Ok(())
}

fn client_list() -> anyhow::Result<()> {
    let clients = SettingsManager::get(|s| s.clients.clone());
    let selected_client = SettingsManager::get(|s| s.selected_client);

    if clients.is_empty() {
        println!("No clients configured.");
        println!("Run 'alembic client scan' to discover clients.");
        return Ok(());
    }

    println!("Configured clients:");
    println!();

    for (idx, config) in clients.iter().enumerate() {
        let selected = if Some(idx) == selected_client {
            " *"
        } else {
            ""
        };

        let client_type = if config.is_wine() {
            "Wine"
        } else {
            "Windows"
        };

        println!("[{}]{} {} - {} ({})",
            idx,
            selected,
            config.display_name(),
            config.install_path().display(),
            client_type,
        );
    }

    println!();

    if selected_client.is_some() {
        println!("* = Currently selected");
    } else {
        println!("No client selected. Use 'alembic client select <index>' to choose one.");
    }

    Ok(())
}

fn client_show(index: usize) -> anyhow::Result<()> {
    let client_config = SettingsManager::get(|s| {
        s.clients.get(index).cloned()
    }).ok_or_else(|| anyhow::anyhow!("Invalid client index: {}. Use 'alembic client list' to see available clients.", index))?;

    println!("Client configuration (index {}):", index);
    println!();
    println!("{}", client_config);

    Ok(())
}

fn client_add(
    mode: String,
    client_path: String,
    launcher_path: String,
    wine_prefix: Option<String>,
    env_vars: Vec<(String, String)>,
) -> anyhow::Result<()> {
    use libalembic::client_config::{WindowsClientConfig, WineClientConfig};
    use std::collections::HashMap;
    use std::path::PathBuf;

    println!("Adding client configuration...");

    let client_config = match mode.to_lowercase().as_str() {
        "windows" => ClientConfig::Windows(WindowsClientConfig {
            display_name: "Manual Windows client".to_string(),
            install_path: PathBuf::from(&client_path),
            dll_path: PathBuf::from(&launcher_path),
        }),
        "wine" => {
            let prefix = wine_prefix.ok_or_else(|| anyhow::anyhow!("Wine prefix required for wine mode"))?;
            let mut additional_env = HashMap::new();
            for (key, value) in env_vars {
                additional_env.insert(key, value);
            }

            ClientConfig::Wine(WineClientConfig {
                display_name: "Manual Wine client".to_string(),
                install_path: PathBuf::from(&client_path),
                wine_executable: PathBuf::from(&launcher_path),
                prefix_path: PathBuf::from(&prefix),
                additional_env,
            })
        }
        _ => bail!("Invalid launch mode '{}'. Must be 'windows' or 'wine'.", mode),
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

fn client_select(index: usize) -> anyhow::Result<()> {
    let clients = SettingsManager::get(|s| s.clients.clone());

    if index >= clients.len() {
        bail!("Invalid client index: {}. Use 'alembic client list' to see available clients.", index);
    }

    let client_name = clients[index].display_name().to_string();

    SettingsManager::modify(|settings| {
        settings.selected_client = Some(index);
    })?;

    println!("✓ Selected client: {}", client_name);

    Ok(())
}

fn client_remove(index: usize) -> anyhow::Result<()> {
    let removed = SettingsManager::get(|s| {
        if index < s.clients.len() {
            Some(s.clients[index].display_name().to_string())
        } else {
            None
        }
    }).ok_or_else(|| anyhow::anyhow!("Invalid client index: {}. Use 'alembic client list' to see available clients.", index))?;

    SettingsManager::modify(|settings| {
        settings.remove_client(index);
        settings.is_configured = !settings.clients.is_empty();
    })?;

    println!("✓ Removed client: {}", removed);

    Ok(())
}

fn server_add(name: String, hostname: String, port: String) -> anyhow::Result<()> {
    println!("Adding server...");

    SettingsManager::modify(|settings| {
        settings.servers.push(ServerInfo {
            name: name.clone(),
            hostname: hostname.clone(),
            port: port.clone(),
        });
    })?;

    println!("✓ Server added!");
    println!();
    println!("  Name:     {}", name);
    println!("  Hostname: {}", hostname);
    println!("  Port:     {}", port);

    Ok(())
}

fn server_list() -> anyhow::Result<()> {
    let servers = SettingsManager::get(|s| s.servers.clone());

    if servers.is_empty() {
        println!("No servers configured. Use 'server add' to add a server.");
        return Ok(());
    }

    // Calculate column widths
    let index_width = "Index".len().max(servers.len().to_string().len());
    let name_width = "Name"
        .len()
        .max(servers.iter().map(|s| s.name.len()).max().unwrap_or(0));
    let hostname_width = "Hostname"
        .len()
        .max(servers.iter().map(|s| s.hostname.len()).max().unwrap_or(0));
    let port_width = "Port"
        .len()
        .max(servers.iter().map(|s| s.port.len()).max().unwrap_or(0));

    // Print top border
    println!(
        "┌─{}─┬─{}─┬─{}─┬─{}─┐",
        "─".repeat(index_width),
        "─".repeat(name_width),
        "─".repeat(hostname_width),
        "─".repeat(port_width)
    );

    // Print header
    println!(
        "│ {:<width_idx$} │ {:<width_name$} │ {:<width_host$} │ {:<width_port$} │",
        "Index",
        "Name",
        "Hostname",
        "Port",
        width_idx = index_width,
        width_name = name_width,
        width_host = hostname_width,
        width_port = port_width
    );

    // Print separator
    println!(
        "├─{}─┼─{}─┼─{}─┼─{}─┤",
        "─".repeat(index_width),
        "─".repeat(name_width),
        "─".repeat(hostname_width),
        "─".repeat(port_width)
    );

    // Print data rows
    for (index, server) in servers.iter().enumerate() {
        println!(
            "│ {:<width_idx$} │ {:<width_name$} │ {:<width_host$} │ {:<width_port$} │",
            index,
            server.name,
            server.hostname,
            server.port,
            width_idx = index_width,
            width_name = name_width,
            width_host = hostname_width,
            width_port = port_width
        );
    }

    // Print bottom border
    println!(
        "└─{}─┴─{}─┴─{}─┴─{}─┘",
        "─".repeat(index_width),
        "─".repeat(name_width),
        "─".repeat(hostname_width),
        "─".repeat(port_width)
    );

    Ok(())
}

fn server_remove(index: usize) -> anyhow::Result<()> {
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

fn account_add(server: usize, username: String, password: String) -> anyhow::Result<()> {
    let servers = SettingsManager::get(|s| s.servers.clone());

    if server >= servers.len() {
        bail!(
            "Invalid server index {}. Run 'alembic server list' to see available servers.",
            server
        );
    }

    println!("Adding account...");

    SettingsManager::modify(|settings| {
        settings.accounts.push(Account {
            server_index: server,
            username: username.clone(),
            password: password.clone(),
        });
    })?;

    println!("✓ Account added!");
    println!();
    println!("  Server:   {}", servers[server].name);
    println!("  Username: {}", username);

    Ok(())
}

fn account_list(server_filter: Option<usize>) -> anyhow::Result<()> {
    let accounts = SettingsManager::get(|s| s.accounts.clone());
    let servers = SettingsManager::get(|s| s.servers.clone());

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

    // Calculate column widths
    let index_width = "Index"
        .len()
        .max(accounts.len().to_string().len());
    let server_width = "Server"
        .len()
        .max(servers.iter().map(|s| s.name.len()).max().unwrap_or(0));
    let username_width = "Username"
        .len()
        .max(
            filtered_accounts
                .iter()
                .map(|(_, a)| a.username.len())
                .max()
                .unwrap_or(0),
        );

    // Print top border
    println!(
        "┌─{}─┬─{}─┬─{}─┐",
        "─".repeat(index_width),
        "─".repeat(server_width),
        "─".repeat(username_width)
    );

    // Print header
    println!(
        "│ {:<width_idx$} │ {:<width_srv$} │ {:<width_user$} │",
        "Index",
        "Server",
        "Username",
        width_idx = index_width,
        width_srv = server_width,
        width_user = username_width
    );

    // Print separator
    println!(
        "├─{}─┼─{}─┼─{}─┤",
        "─".repeat(index_width),
        "─".repeat(server_width),
        "─".repeat(username_width)
    );

    // Print data rows
    for (index, account) in &filtered_accounts {
        let server_name = if account.server_index < servers.len() {
            &servers[account.server_index].name
        } else {
            "<unknown>"
        };

        println!(
            "│ {:<width_idx$} │ {:<width_srv$} │ {:<width_user$} │",
            index,
            server_name,
            account.username,
            width_idx = index_width,
            width_srv = server_width,
            width_user = username_width
        );
    }

    // Print bottom border
    println!(
        "└─{}─┴─{}─┴─{}─┘",
        "─".repeat(index_width),
        "─".repeat(server_width),
        "─".repeat(username_width)
    );

    Ok(())
}

fn account_remove(index: usize) -> anyhow::Result<()> {
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

fn client_scan() -> anyhow::Result<()> {
    use std::io::{self, Write};

    println!("Scanning for AC client installations...");
    println!();

    let discovered = scanner::scan_all()?;

    if discovered.is_empty() {
        println!("No client installations found.");
        println!("You can add a client manually with: alembic client add");
        return Ok(());
    }

    println!("Found {} client installation(s):", discovered.len());
    println!();

    let mut added_count = 0;
    let mut skipped_count = 0;

    for config in discovered {
        // Check if already exists
        let already_exists = SettingsManager::get(|s| {
            s.clients.iter().any(|existing| {
                existing.install_path() == config.install_path()
            })
        });

        if already_exists {
            println!("Skipping (already configured): {}", config.display_name());
            println!("Path: {}", config.install_path().display());
            println!();
            skipped_count += 1;
            continue;
        }

        // Show details
        println!("Found: {}", config.display_name());
        println!("Path: {}", config.install_path().display());
        if config.is_wine() {
            println!("Type: Wine");
        } else {
            println!("Type: Windows");
        }

        // Prompt user
        print!("Add this client? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        if response == "y" || response == "yes" {
            SettingsManager::modify(|settings| {
                settings.add_client(config.clone(), false);
                settings.is_configured = true;
            })?;
            println!("✓ Added!");
            added_count += 1;
        } else {
            println!("Skipped.");
            skipped_count += 1;
        }

        println!();
    }

    // Summary
    println!("Scan complete:");
    println!("Added: {}", added_count);
    println!("Skipped: {}", skipped_count);

    if added_count > 0 {
        println!();
        println!("Use 'alembic client list' to see all clients.");
        println!("Use 'alembic client select <index>' to choose a client.");
    }

    Ok(())
}
