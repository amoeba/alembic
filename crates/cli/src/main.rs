use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use libalembic::{
    launch::Launcher,
    settings::{Account, ClientInfo, LaunchConfig, ServerInfo, SettingsManager},
    LaunchMode,
};
use std::{collections::HashMap, sync::mpsc::channel};

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
            ClientCommands::Remove { index } => client_remove(index),
            ClientCommands::Show { index } => client_show(index),
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
    let launch_mode = match mode.to_lowercase().as_str() {
        "windows" => LaunchMode::Windows,
        "wine" => LaunchMode::Wine,
        _ => bail!("Invalid launch mode '{}'. Must be 'windows' or 'wine'.", mode),
    };

    let client_info = ClientInfo { path: client_path };

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

    let mut environment_variables = HashMap::new();
    for (key, value) in env_vars {
        environment_variables.insert(key, value);
    }

    let launch_config = LaunchConfig {
        launcher_path,
        prefix_path: wine_prefix,
        environment_variables,
    };

    let mode_str = match launch_mode {
        LaunchMode::Windows => "windows",
        LaunchMode::Wine => "wine",
    };

    println!("Launch mode: {}", mode_str);
    println!("Server: {}:{}", server_info.hostname, server_info.port);
    println!("Account: {}", account_info.username);

    run_launcher(launch_mode, client_info, server_info, account_info, launch_config)
}

fn preset_launch(server_name: Option<String>, account_name: Option<String>) -> anyhow::Result<()> {
    // Load settings
    let launch_mode = SettingsManager::get(|s| s.launch_mode);
    let client_info = SettingsManager::get(|s| s.client.clone());
    let launch_config = SettingsManager::get(|s| s.launch_config.clone());

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
        let selected_server_index = SettingsManager::get(|s| s.selected_server);
        match selected_server_index {
            Some(idx) => SettingsManager::get(|s| s.servers.get(idx).cloned())
                .context("Selected server index out of range")?,
            None => bail!("No server selected. Please configure settings or use --server"),
        }
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
        let selected_account_index = SettingsManager::get(|s| s.selected_account);
        match selected_account_index {
            Some(idx) => SettingsManager::get(|s| s.accounts.get(idx).cloned())
                .context("Selected account index out of range")?,
            None => bail!("No account selected. Please configure settings or use --account"),
        }
    };

    let mode_str = match launch_mode {
        LaunchMode::Windows => "windows",
        LaunchMode::Wine => "wine",
    };

    println!("Launch mode: {}", mode_str);
    println!(
        "Server: {} ({}:{})",
        server_info.name, server_info.hostname, server_info.port
    );
    println!("Account: {}", account_info.username);

    run_launcher(
        launch_mode,
        client_info,
        server_info,
        account_info,
        launch_config,
    )
}

fn run_launcher(
    launch_mode: LaunchMode,
    client_info: ClientInfo,
    server_info: ServerInfo,
    account_info: Account,
    launch_config: LaunchConfig,
) -> anyhow::Result<()> {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    let mut launcher = Launcher::new(
        launch_mode,
        client_info,
        server_info,
        account_info,
        launch_config,
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
    let is_configured = SettingsManager::get(|s| s.is_configured);

    if !is_configured {
        println!("No clients configured. Use 'client add' to configure a client.");
        return Ok(());
    }

    let launch_mode = SettingsManager::get(|s| s.launch_mode);
    let client_path = SettingsManager::get(|s| s.client.path.clone());
    let wine_prefix = SettingsManager::get(|s| s.launch_config.prefix_path.clone());

    let (mode_str, path) = match launch_mode {
        LaunchMode::Windows => ("windows", client_path),
        LaunchMode::Wine => ("wine", wine_prefix.unwrap_or_else(|| "(no prefix)".to_string())),
    };

    // Calculate column widths
    let index_width = "Index".len().max(1);
    let method_width = "Method".len().max(mode_str.len());
    let path_width = "Path".len().max(path.len());

    // Print top border
    println!("┌─{}─┬─{}─┬─{}─┐",
        "─".repeat(index_width),
        "─".repeat(method_width),
        "─".repeat(path_width)
    );

    // Print header
    println!("│ {:<width_idx$} │ {:<width_method$} │ {:<width_path$} │",
        "Index", "Method", "Path",
        width_idx = index_width,
        width_method = method_width,
        width_path = path_width
    );

    // Print separator
    println!("├─{}─┼─{}─┼─{}─┤",
        "─".repeat(index_width),
        "─".repeat(method_width),
        "─".repeat(path_width)
    );

    // Print data row
    println!("│ {:<width_idx$} │ {:<width_method$} │ {:<width_path$} │",
        "0", mode_str, path,
        width_idx = index_width,
        width_method = method_width,
        width_path = path_width
    );

    // Print bottom border
    println!("└─{}─┴─{}─┴─{}─┘",
        "─".repeat(index_width),
        "─".repeat(method_width),
        "─".repeat(path_width)
    );

    Ok(())
}

fn client_show(index: usize) -> anyhow::Result<()> {
    let is_configured = SettingsManager::get(|s| s.is_configured);

    // Check if a client exists at the requested index
    let client_exists = index == 0 && is_configured;

    if !client_exists {
        bail!("Invalid index {}. No client exists at that index. Run 'alembic client list' to see available clients.", index);
    }

    println!("Client configuration (index {}):", index);
    println!();

    let launch_mode = SettingsManager::get(|s| s.launch_mode);
    let client_path = SettingsManager::get(|s| s.client.path.clone());
    let launcher_path = SettingsManager::get(|s| s.launch_config.launcher_path.clone());
    let wine_prefix = SettingsManager::get(|s| s.launch_config.prefix_path.clone());
    let env_vars = SettingsManager::get(|s| s.launch_config.environment_variables.clone());

    let mode_str = match launch_mode {
        LaunchMode::Windows => "windows",
        LaunchMode::Wine => "wine",
    };

    println!("  Launch mode:   {}", mode_str);
    println!("  Client path:   {}", client_path);
    println!("  Launcher path: {}", launcher_path);

    if let Some(prefix) = wine_prefix {
        println!("  Wine prefix:   {}", prefix);
    }

    if !env_vars.is_empty() {
        println!();
        println!("  Environment variables:");
        for (key, value) in env_vars {
            println!("    {}={}", key, value);
        }
    }

    Ok(())
}

fn client_add(
    mode: String,
    client_path: String,
    launcher_path: String,
    wine_prefix: Option<String>,
    env_vars: Vec<(String, String)>,
) -> anyhow::Result<()> {
    let launch_mode = match mode.to_lowercase().as_str() {
        "windows" => LaunchMode::Windows,
        "wine" => LaunchMode::Wine,
        _ => bail!("Invalid launch mode '{}'. Must be 'windows' or 'wine'.", mode),
    };

    println!("Adding client configuration...");

    let mut environment_variables = HashMap::new();
    for (key, value) in env_vars {
        environment_variables.insert(key, value);
    }

    SettingsManager::modify(|settings| {
        settings.is_configured = true;
        settings.launch_mode = launch_mode;
        settings.client.path = client_path.clone();
        settings.launch_config.launcher_path = launcher_path.clone();
        settings.launch_config.prefix_path = wine_prefix.clone();
        settings.launch_config.environment_variables = environment_variables.clone();
    })?;

    println!("✓ Client configuration saved!");
    println!();

    // Show what was configured
    client_show(0)
}

fn client_remove(index: usize) -> anyhow::Result<()> {
    let is_configured = SettingsManager::get(|s| s.is_configured);

    // Check if a client exists at the requested index
    let client_exists = index == 0 && is_configured;

    if !client_exists {
        bail!("Invalid index {}. No client exists at that index. Run 'alembic client list' to see available clients.", index);
    }

    println!("Resetting client configuration to defaults...");

    SettingsManager::modify(|settings| {
        settings.is_configured = false;
        settings.launch_mode = LaunchMode::Windows;
        settings.client.path = "C:\\Turbine\\Asheron's Call\\".to_string();
        settings.launch_config.launcher_path = "Alembic.dll".to_string();
        settings.launch_config.prefix_path = None;
        settings.launch_config.environment_variables.clear();
    })?;

    println!("✓ Client configuration reset to defaults!");

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
