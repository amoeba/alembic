use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use libalembic::{
    launcher::{traits::ClientLauncher, Launcher},
    scanner,
    settings::{Account, ServerInfo, SettingsManager},
};

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
    /// Manage configuration (accounts, clients, servers, DLLs)
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
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

    /// Run cork to find and inject into running acclient.exe
    Inject,

    /// Launch using saved settings with optional overrides
    Launch {
        /// Server name to use (overrides selected server in settings)
        #[arg(long)]
        server: Option<String>,

        /// Account username to use (overrides selected account in settings)
        #[arg(long)]
        account: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
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

    /// Select an account by index
    Select {
        /// Index of the account to select (from 'account list')
        index: usize,
    },

    /// Clear the selected account
    Reset,

    /// Remove an account by index
    Remove {
        /// Index of the account to remove (from 'account list')
        index: usize,
    },

    /// Edit an existing account (only specified fields are updated)
    Edit {
        /// Index of the account to edit (from 'account list')
        index: usize,

        /// Server index (from 'server list')
        #[arg(long)]
        server: Option<usize>,

        /// Account username
        #[arg(long)]
        username: Option<String>,

        /// Account password
        #[arg(long)]
        password: Option<String>,
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

    /// Clear the selected client
    Reset,

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

    /// Edit an existing client configuration (only specified fields are updated)
    Edit {
        /// Index of the client to edit (from 'client list')
        index: usize,

        /// New name for the client
        #[arg(long)]
        name: Option<String>,

        /// Path to game client executable (e.g., "C:\\AC\\acclient.exe")
        #[arg(long)]
        client_path: Option<String>,

        /// Wrapper program path (wine64 executable for Wine, or flatpak for flatpak)
        #[arg(long)]
        wrapper_program: Option<String>,

        /// Arguments to add to the launch command (can be specified multiple times, order matters)
        #[arg(long = "arg")]
        args: Vec<String>,

        /// Clear all existing launch command arguments before adding new ones
        #[arg(long)]
        clear_args: bool,

        /// Environment variables to set (format: KEY=VALUE, can be specified multiple times)
        #[arg(long = "env", value_parser = parse_key_val)]
        env_vars: Vec<(String, String)>,

        /// Environment variables to remove (can be specified multiple times)
        #[arg(long = "unset-env")]
        unset_env_vars: Vec<String>,
    },

    /// Scan for installed clients and wine prefixes
    Scan,

    /// Manage DLL configurations for a client
    Dll {
        /// Index of the client (from 'client list')
        #[arg(long)]
        client: usize,

        #[command(subcommand)]
        command: ClientDllCommands,
    },
}

#[derive(Subcommand)]
enum ClientDllCommands {
    /// Add a DLL configuration to a client
    Add {
        /// DLL type (alembic or decal)
        #[arg(long = "type")]
        dll_type: String,

        /// Path to the DLL
        #[arg(long)]
        path: String,

        /// Startup function name (e.g., DecalStartup)
        #[arg(long)]
        startup_function: Option<String>,
    },

    /// List DLLs for a client
    List,

    /// Select a DLL for a client by index
    Select {
        /// Index of the DLL to select
        index: usize,
    },

    /// Clear the selected DLL for a client
    Reset,

    /// Remove a DLL from a client by index
    Remove {
        /// Index of the DLL to remove
        index: usize,
    },

    /// Show detailed DLL configuration for a client
    Show {
        /// Index of the DLL to show
        index: usize,
    },

    /// Edit an existing DLL configuration for a client
    Edit {
        /// Index of the DLL to edit
        index: usize,

        /// DLL type (alembic or decal)
        #[arg(long = "type")]
        dll_type: Option<String>,

        /// Path to the DLL
        #[arg(long)]
        path: Option<String>,

        /// Startup function name (e.g., DecalStartup)
        #[arg(long)]
        startup_function: Option<String>,
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

    /// Select a server by index
    Select {
        /// Index of the server to select (from 'server list')
        index: usize,
    },

    /// Clear the selected server
    Reset,

    /// Remove a server by index
    Remove {
        /// Index of the server to remove (from 'server list')
        index: usize,
    },

    /// Edit an existing server (only specified fields are updated)
    Edit {
        /// Index of the server to edit (from 'server list')
        index: usize,

        /// Server name
        #[arg(long)]
        name: Option<String>,

        /// Server hostname or IP address
        #[arg(long)]
        hostname: Option<String>,

        /// Server port
        #[arg(long)]
        port: Option<String>,
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
        Commands::Config { command } => match command {
            ConfigCommands::Account { command } => match command {
                AccountCommands::Add {
                    server,
                    username,
                    password,
                } => account_add(server, username, password),
                AccountCommands::List { server } => account_list(server),
                AccountCommands::Select { index } => account_select(index),
                AccountCommands::Reset => account_reset(),
                AccountCommands::Remove { index } => account_remove(index),
                AccountCommands::Edit {
                    index,
                    server,
                    username,
                    password,
                } => account_edit(index, server, username, password),
            },
            ConfigCommands::Client { command } => match command {
                ClientCommands::Add {
                    mode,
                    client_path,
                    launcher_path,
                    wine_prefix,
                    env_vars,
                } => client_add(mode, client_path, launcher_path, wine_prefix, env_vars),
                ClientCommands::List => client_list(),
                ClientCommands::Select { index } => client_select(index),
                ClientCommands::Reset => client_reset(),
                ClientCommands::Remove { index } => client_remove(index),
                ClientCommands::Show { index } => client_show(index),
                ClientCommands::Edit {
                    index,
                    name,
                    client_path,
                    wrapper_program,
                    args,
                    clear_args,
                    env_vars,
                    unset_env_vars,
                } => client_edit(
                    index,
                    name,
                    client_path,
                    wrapper_program,
                    args,
                    clear_args,
                    env_vars,
                    unset_env_vars,
                ),
                ClientCommands::Scan => client_scan(),
                ClientCommands::Dll { client, command } => match command {
                    ClientDllCommands::Add {
                        dll_type,
                        path,
                        startup_function,
                    } => client_dll_add(client, dll_type, path, startup_function),
                    ClientDllCommands::List => client_dll_list(client),
                    ClientDllCommands::Select { index } => client_dll_select(client, index),
                    ClientDllCommands::Reset => client_dll_reset(client),
                    ClientDllCommands::Remove { index } => client_dll_remove(client, index),
                    ClientDllCommands::Show { index } => client_dll_show(client, index),
                    ClientDllCommands::Edit {
                        index,
                        dll_type,
                        path,
                        startup_function,
                    } => client_dll_edit(client, index, dll_type, path, startup_function),
                },
            },
            ConfigCommands::Server { command } => match command {
                ServerCommands::Add {
                    name,
                    hostname,
                    port,
                } => server_add(name, hostname, port),
                ServerCommands::List => server_list(),
                ServerCommands::Select { index } => server_select(index),
                ServerCommands::Reset => server_reset(),
                ServerCommands::Remove { index } => server_remove(index),
                ServerCommands::Edit {
                    index,
                    name,
                    hostname,
                    port,
                } => server_edit(index, name, hostname, port),
            },
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
        Commands::Inject => inject(),
    }
}

#[allow(clippy::too_many_arguments)]
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
fn validate_launch_config(
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

fn preset_launch(server_name: Option<String>, account_name: Option<String>) -> anyhow::Result<()> {
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

fn client_list() -> anyhow::Result<()> {
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

fn client_show(index: usize) -> anyhow::Result<()> {
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

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn client_edit(
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

fn client_add(
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

fn client_select(index: usize) -> anyhow::Result<()> {
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

fn client_reset() -> anyhow::Result<()> {
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

fn client_remove(index: usize) -> anyhow::Result<()> {
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

fn server_add(name: String, hostname: String, port: String) -> anyhow::Result<()> {
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

fn server_list() -> anyhow::Result<()> {
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

fn server_select(index: usize) -> anyhow::Result<()> {
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

fn server_reset() -> anyhow::Result<()> {
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

fn server_edit(
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

fn account_add(server: usize, username: String, password: String) -> anyhow::Result<()> {
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

fn account_list(server_filter: Option<usize>) -> anyhow::Result<()> {
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

fn account_select(index: usize) -> anyhow::Result<()> {
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

fn account_reset() -> anyhow::Result<()> {
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

fn account_edit(
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

fn client_scan() -> anyhow::Result<()> {
    use std::io::{self, Write};

    println!("Scanning for AC client installations...");

    // Get the prefixes that will be scanned (for reporting)
    let scanned_prefixes = scanner::WineScanner::get_scannable_prefixes();

    let discovered = scanner::scan_all()?;

    let mut added_count = 0;
    let mut skipped_configs: Vec<String> = vec![];
    let mut added_configs: Vec<String> = vec![];

    // Check if there are any existing clients before we start adding
    let had_no_clients = SettingsManager::get(|s| s.clients.is_empty());

    for config in &discovered {
        // Check if already exists
        let already_exists = SettingsManager::get(|s| {
            s.clients
                .iter()
                .any(|existing| existing.install_path() == config.install_path())
        });

        if already_exists {
            skipped_configs.push(format!(
                "{} ({})",
                config.name(),
                config.install_path().display()
            ));
            continue;
        }

        // Show details and prompt
        println!();
        println!("Found: {}", config.name());
        println!("Path: {}", config.install_path().display());
        println!(
            "Type: {}",
            if config.is_wine() { "Wine" } else { "Windows" }
        );

        print!("Add this client? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        if response == "y" || response == "yes" {
            let should_select = had_no_clients && added_count == 0;

            SettingsManager::modify(|settings| {
                settings.add_client(config.clone(), should_select);
                settings.is_configured = true;
            })?;

            added_configs.push(format!(
                "{} ({})",
                config.name(),
                config.install_path().display()
            ));
            added_count += 1;
        } else {
            skipped_configs.push(format!(
                "{} ({})",
                config.name(),
                config.install_path().display()
            ));
        }
    }

    // Print summary report
    println!();
    println!("=== Scan Report ===");
    println!();
    println!("Scanned {} wine prefix(es):", scanned_prefixes.len());
    for prefix in &scanned_prefixes {
        println!("  - {}", prefix.display());
    }
    println!();
    println!(
        "Found {} client(s), added {}, skipped {}",
        discovered.len(),
        added_configs.len(),
        skipped_configs.len()
    );

    if !added_configs.is_empty() {
        println!();
        println!("Added:");
        for name in &added_configs {
            println!("  + {}", name);
        }
    }

    if !skipped_configs.is_empty() {
        println!();
        println!("Skipped:");
        for name in &skipped_configs {
            println!("  - {}", name);
        }
    }

    if discovered.is_empty() {
        println!();
        println!("No client installations found.");
        println!("You can add a client manually with: alembic config client add");
    }

    Ok(())
}

fn client_dll_list(client_idx: usize) -> anyhow::Result<()> {
    let dlls = SettingsManager::get(|s| s.get_client_dlls(client_idx).cloned());
    let selected_dll = SettingsManager::get(|s| {
        s.clients.get(client_idx).and_then(|c| match c {
            libalembic::settings::ClientConfigType::Windows(w) => w.selected_dll,
            libalembic::settings::ClientConfigType::Wine(w) => w.selected_dll,
        })
    });

    match dlls {
        Some(dlls) => {
            if dlls.is_empty() {
                println!("No DLLs configured for this client.");
                println!(
                    "Run 'alembic config client dll --client {} add --type <type> --path <path>'",
                    client_idx
                );
                return Ok(());
            }

            for (idx, dll) in dlls.iter().enumerate() {
                let is_selected = Some(idx) == selected_dll;
                let marker = if is_selected { " * " } else { "   " };

                println!(
                    "{}{}: {} ({})",
                    marker,
                    idx,
                    dll.dll_path.display(),
                    dll.dll_type
                );
            }

            Ok(())
        }
        None => {
            bail!(
                "Invalid client index: {}. Use 'alembic client list' to see available clients.",
                client_idx
            )
        }
    }
}

fn client_dll_add(
    client_idx: usize,
    dll_type: String,
    dll_path: String,
    startup_function: Option<String>,
) -> anyhow::Result<()> {
    use libalembic::inject_config::{DllType, InjectConfig};
    use std::path::PathBuf;

    // Validate client exists
    let _client =
        SettingsManager::get(|s| s.clients.get(client_idx).cloned()).ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid client index: {}. Use 'alembic client list' to see available clients.",
                client_idx
            )
        })?;

    // Parse DLL type
    let dll_type_enum = match dll_type.to_lowercase().as_str() {
        "alembic" => DllType::Alembic,
        "decal" => DllType::Decal,
        _ => {
            anyhow::bail!(
                "Invalid DLL type: {}. Must be 'alembic' or 'decal'",
                dll_type
            );
        }
    };

    // Use provided startup function or default
    let startup_fn = startup_function.or_else(|| match dll_type_enum {
        DllType::Decal => Some("DecalStartup".to_string()),
        DllType::Alembic => None,
    });

    // Create the InjectConfig
    let inject_config = InjectConfig {
        dll_type: dll_type_enum,
        dll_path: PathBuf::from(&dll_path),
        startup_function: startup_fn,
    };

    println!("Adding DLL configuration to client {}...", client_idx);
    println!("  Type: {}", inject_config.dll_type);
    println!("  Path: {}", inject_config.dll_path.display());

    SettingsManager::modify(|settings| {
        settings.add_dll_to_client(client_idx, inject_config);
    })?;

    println!();
    println!("✓ DLL configuration added!");

    Ok(())
}

fn client_dll_select(client_idx: usize, dll_idx: usize) -> anyhow::Result<()> {
    let dlls = SettingsManager::get(|s| s.get_client_dlls(client_idx).cloned());

    let dll_count = match dlls {
        Some(dlls) => dlls.len(),
        None => {
            bail!(
                "Invalid client index: {}. Use 'alembic client list' to see available clients.",
                client_idx
            )
        }
    };

    if dll_idx >= dll_count {
        println!("Invalid DLL index: {}", dll_idx);
        println!(
            "Use 'alembic config client dll --client {} list' to see available DLLs.",
            client_idx
        );
        return Ok(());
    }

    SettingsManager::modify(|settings| {
        settings.select_dll_for_client(client_idx, Some(dll_idx));
    })?;

    println!(
        "✓ Selected DLL at index {} for client {}",
        dll_idx, client_idx
    );

    Ok(())
}

fn client_dll_reset(client_idx: usize) -> anyhow::Result<()> {
    let was_selected = SettingsManager::get(|s| {
        s.clients.get(client_idx).and_then(|c| match c {
            libalembic::settings::ClientConfigType::Windows(w) => w.selected_dll,
            libalembic::settings::ClientConfigType::Wine(w) => w.selected_dll,
        })
    });

    SettingsManager::modify(|settings| {
        settings.select_dll_for_client(client_idx, None);
    })?;

    if was_selected.is_some() {
        println!("✓ DLL selection cleared for client {}", client_idx);
    } else {
        println!("No DLL was selected for client {}", client_idx);
    }

    Ok(())
}

fn client_dll_remove(client_idx: usize, dll_idx: usize) -> anyhow::Result<()> {
    use std::io::{self, Write};

    let dll_info = SettingsManager::get(|s| {
        s.get_client_dlls(client_idx).and_then(|dlls| {
            dlls.get(dll_idx)
                .map(|dll| (dll.dll_type.to_string(), dll.dll_path.display().to_string()))
        })
    });

    let (dll_type, dll_path) = match dll_info {
        Some(info) => info,
        None => {
            println!("Invalid DLL index: {}", dll_idx);
            println!(
                "Use 'alembic config client dll --client {} list' to see available DLLs.",
                client_idx
            );
            return Ok(());
        }
    };

    println!("This will remove the following DLL configuration:");
    println!("  [{}] {} - {}", dll_idx, dll_type, dll_path);
    println!();
    print!("Continue? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let response = input.trim().to_lowercase();

    if response != "y" && response != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    SettingsManager::modify(|settings| {
        settings.remove_dll_from_client(client_idx, dll_idx);
    })?;

    println!();
    println!(
        "✓ DLL configuration has been removed from client {}.",
        client_idx
    );

    Ok(())
}

fn client_dll_show(client_idx: usize, dll_idx: usize) -> anyhow::Result<()> {
    let dll = SettingsManager::get(|s| {
        s.get_client_dlls(client_idx)
            .and_then(|dlls| dlls.get(dll_idx).cloned())
    });

    match dll {
        Some(dll) => {
            println!(
                "DLL configuration (client {}, index {}):",
                client_idx, dll_idx
            );
            println!();
            println!("{}", dll);
        }
        None => {
            println!("Invalid client or DLL index.");
            println!(
                "Use 'alembic config client dll --client {} list' to see available DLLs.",
                client_idx
            );
        }
    }

    Ok(())
}

fn client_dll_edit(
    client_idx: usize,
    dll_idx: usize,
    dll_type: Option<String>,
    path: Option<String>,
    startup_function: Option<String>,
) -> anyhow::Result<()> {
    use libalembic::inject_config::DllType;
    use std::path::PathBuf;

    let dll_exists = SettingsManager::get(|s| {
        s.get_client_dlls(client_idx)
            .map(|dlls| dll_idx < dlls.len())
            .unwrap_or(false)
    });

    if !dll_exists {
        bail!(
            "Invalid client or DLL index. Use 'alembic config client dll --client {} list' to see available DLLs.",
            client_idx
        );
    }

    if dll_type.is_none() && path.is_none() && startup_function.is_none() {
        println!(
            "No changes specified. Use --type, --path, or --startup-function to modify the DLL."
        );
        return Ok(());
    }

    println!(
        "Editing DLL at index {} for client {}...",
        dll_idx, client_idx
    );

    SettingsManager::modify(|settings| {
        if let Some(dlls) = settings.get_client_dlls_mut(client_idx) {
            if let Some(dll) = dlls.get_mut(dll_idx) {
                if let Some(t) = &dll_type {
                    match t.to_lowercase().as_str() {
                        "alembic" => {
                            println!("  Updated type to: Alembic");
                            dll.dll_type = DllType::Alembic;
                        }
                        "decal" => {
                            println!("  Updated type to: Decal");
                            dll.dll_type = DllType::Decal;
                        }
                        _ => {
                            println!(
                                "  Warning: Invalid DLL type '{}', ignoring. Use 'alembic' or 'decal'.",
                                t
                            );
                        }
                    }
                }
                if let Some(p) = &path {
                    println!("  Updated path to: {}", p);
                    dll.dll_path = PathBuf::from(p);
                }
                if let Some(f) = &startup_function {
                    if f.is_empty() || f == "none" {
                        println!("  Removed startup function");
                        dll.startup_function = None;
                    } else {
                        println!("  Updated startup function to: {}", f);
                        dll.startup_function = Some(f.clone());
                    }
                }
            }
        }
    })?;

    println!("✓ DLL updated!");

    Ok(())
}

#[allow(dead_code)]
fn dll_scan() -> anyhow::Result<()> {
    use std::io::{self, Write};

    println!("Scanning for DLL installations...");

    // Get prefixes that will be scanned (from configured clients)
    #[cfg(not(target_os = "windows"))]
    let scanned_prefixes = scanner::get_dll_scannable_prefixes();

    let discovered_dlls = scanner::scan_for_decal_dlls()?;

    let mut added_dlls: Vec<String> = vec![];
    let mut skipped_dlls: Vec<String> = vec![];

    // Get the selected client index
    let selected_client_idx = SettingsManager::get(|s| s.selected_client);

    if selected_client_idx.is_none() {
        println!("No client selected. Select a client first with: alembic client select <index>");
        return Ok(());
    }

    let selected_client_idx = selected_client_idx.unwrap();

    for dll in &discovered_dlls {
        // Check if already exists in this client's DLL list
        let already_exists = SettingsManager::get(|s| {
            s.get_client_dlls(selected_client_idx)
                .map(|dlls| {
                    dlls.iter()
                        .any(|existing| existing.dll_path == dll.dll_path)
                })
                .unwrap_or(false)
        });

        if already_exists {
            skipped_dlls.push(format!("{} ({})", dll.dll_type, dll.dll_path.display()));
            continue;
        }

        // Show details and prompt
        println!();
        println!("Found: {}", dll.dll_path.display());
        println!("Type: {}", dll.dll_type);

        print!("Add this DLL to selected client? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        if response == "y" || response == "yes" {
            SettingsManager::modify(|settings| {
                settings.add_dll_to_client(selected_client_idx, dll.clone());

                // Auto-select first DLL if none selected
                let current_selected =
                    settings
                        .clients
                        .get(selected_client_idx)
                        .and_then(|c| match c {
                            libalembic::settings::ClientConfigType::Windows(w) => w.selected_dll,
                            libalembic::settings::ClientConfigType::Wine(w) => w.selected_dll,
                        });

                if current_selected.is_none() && added_dlls.is_empty() {
                    settings.select_dll_for_client(selected_client_idx, Some(0));
                }
            })?;

            added_dlls.push(format!("{} ({})", dll.dll_type, dll.dll_path.display()));
        } else {
            skipped_dlls.push(format!("{} ({})", dll.dll_type, dll.dll_path.display()));
        }
    }

    // Print summary report
    println!();
    println!("=== Scan Report ===");

    #[cfg(not(target_os = "windows"))]
    {
        println!();
        if scanned_prefixes.is_empty() {
            println!(
                "No clients configured. Configure a client first with: alembic config client scan"
            );
        } else {
            println!(
                "Scanned {} prefix(es) from configured clients:",
                scanned_prefixes.len()
            );
            for prefix in &scanned_prefixes {
                println!("  - {}", prefix.display());
            }
        }
    }

    println!();
    println!(
        "Found {} DLL(s), added {}, skipped {}",
        discovered_dlls.len(),
        added_dlls.len(),
        skipped_dlls.len()
    );

    if !added_dlls.is_empty() {
        println!();
        println!("Added:");
        for name in &added_dlls {
            println!("  + {}", name);
        }
    }

    if !skipped_dlls.is_empty() {
        println!();
        println!("Skipped:");
        for name in &skipped_dlls {
            println!("  - {}", name);
        }
    }

    if discovered_dlls.is_empty() {
        println!();
        println!("No DLL installations found.");
        println!("Make sure Decal's Inject.dll is installed in your wine prefix.");
    }

    Ok(())
}

fn inject() -> anyhow::Result<()> {
    use libalembic::settings::ClientConfigType;
    use std::process::Command;

    println!("Running cork to find and inject into acclient.exe...");
    println!();

    // Get selected client config and index
    let (client_config, client_idx) =
        SettingsManager::get(|s| (s.get_selected_client().cloned(), s.selected_client));

    let client_config = match client_config {
        Some(config) => config,
        None => {
            bail!("No client selected. Use 'alembic client select <index>' to select a client.");
        }
    };

    // Only Wine clients are supported for now
    let wine_config = match client_config {
        ClientConfigType::Wine(config) => config,
        ClientConfigType::Windows(_) => {
            bail!("Inject command currently only supports Wine clients");
        }
    };

    // Get selected DLL config for the selected client
    let dll_config = match client_idx {
        Some(idx) => SettingsManager::get(|s| s.get_client_selected_dll(idx).cloned()),
        None => None,
    };

    let dll_path = match dll_config {
        Some(config) => config.dll_path.display().to_string(),
        None => {
            bail!(
                "No DLL selected. Use 'alembic client dll --client <index> select <dll_index>' to select a DLL for injection."
            );
        }
    };

    // Get cork.exe path
    let cork_path = std::env::current_exe()
        .context("Failed to get current executable path")?
        .parent()
        .context("Failed to get executable directory")?
        .join("cork.exe");

    if !cork_path.exists() {
        bail!("cork.exe not found at {:?}. Make sure it's built with: cargo build --package cork --target i686-pc-windows-gnu", cork_path);
    }

    println!("Client: {}", wine_config.name);
    if let Some(prefix) = wine_config.launch_command.env.get("WINEPREFIX") {
        println!("Wine prefix: {}", prefix);
    }
    println!("DLL: {}", dll_path);
    println!("Cork path: {}", cork_path.display());
    println!();

    // Run cork.exe under wine using the launch_command configuration
    let launch_cmd = &wine_config.launch_command;
    let mut cmd = Command::new(&launch_cmd.program);

    // Add pre-args (e.g., for flatpak: "run", "--command=wine", "net.lutris.Lutris")
    for arg in &launch_cmd.args {
        cmd.arg(arg);
    }

    // Set environment variables
    for (key, value) in &launch_cmd.env {
        cmd.env(key, value);
    }

    cmd.arg(cork_path.to_str().context("Invalid cork path")?);
    cmd.arg("--dll").arg(&dll_path);

    let output = cmd
        .output()
        .context("Failed to execute cork.exe under wine")?;

    println!("Cork output:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.stderr.is_empty() {
        println!("Cork stderr:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if output.status.success() {
        println!("Cork completed successfully");
        Ok(())
    } else {
        bail!("Cork exited with status: {}", output.status);
    }
}
