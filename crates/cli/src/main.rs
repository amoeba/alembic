mod commands;

use clap::{Parser, Subcommand};

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

    /// Scan for installed DLLs (Alembic, Decal)
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
                } => commands::account::account_add(server, username, password),
                AccountCommands::List { server } => commands::account::account_list(server),
                AccountCommands::Select { index } => commands::account::account_select(index),
                AccountCommands::Reset => commands::account::account_reset(),
                AccountCommands::Remove { index } => commands::account::account_remove(index),
                AccountCommands::Edit {
                    index,
                    server,
                    username,
                    password,
                } => commands::account::account_edit(index, server, username, password),
            },
            ConfigCommands::Client { command } => match command {
                ClientCommands::Add {
                    mode,
                    client_path,
                    launcher_path,
                    wine_prefix,
                    env_vars,
                } => commands::client::client_add(mode, client_path, launcher_path, wine_prefix, env_vars),
                ClientCommands::List => commands::client::client_list(),
                ClientCommands::Select { index } => commands::client::client_select(index),
                ClientCommands::Reset => commands::client::client_reset(),
                ClientCommands::Remove { index } => commands::client::client_remove(index),
                ClientCommands::Show { index } => commands::client::client_show(index),
                ClientCommands::Edit {
                    index,
                    name,
                    client_path,
                    wrapper_program,
                    args,
                    clear_args,
                    env_vars,
                    unset_env_vars,
                } => commands::client::client_edit(
                    index,
                    name,
                    client_path,
                    wrapper_program,
                    args,
                    clear_args,
                    env_vars,
                    unset_env_vars,
                ),
                ClientCommands::Scan => commands::scan::client_scan(),
                ClientCommands::Dll { client, command } => match command {
                    ClientDllCommands::Add {
                        dll_type,
                        path,
                        startup_function,
                    } => commands::dll::client_dll_add(client, dll_type, path, startup_function),
                    ClientDllCommands::List => commands::dll::client_dll_list(client),
                    ClientDllCommands::Select { index } => commands::dll::client_dll_select(client, index),
                    ClientDllCommands::Reset => commands::dll::client_dll_reset(client),
                    ClientDllCommands::Remove { index } => commands::dll::client_dll_remove(client, index),
                    ClientDllCommands::Show { index } => commands::dll::client_dll_show(client, index),
                    ClientDllCommands::Edit {
                        index,
                        dll_type,
                        path,
                        startup_function,
                    } => commands::dll::client_dll_edit(client, index, dll_type, path, startup_function),
                    ClientDllCommands::Scan => commands::dll::client_dll_scan(client),
                },
            },
            ConfigCommands::Server { command } => match command {
                ServerCommands::Add {
                    name,
                    hostname,
                    port,
                } => commands::server::server_add(name, hostname, port),
                ServerCommands::List => commands::server::server_list(),
                ServerCommands::Select { index } => commands::server::server_select(index),
                ServerCommands::Reset => commands::server::server_reset(),
                ServerCommands::Remove { index } => commands::server::server_remove(index),
                ServerCommands::Edit {
                    index,
                    name,
                    hostname,
                    port,
                } => commands::server::server_edit(index, name, hostname, port),
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
        } => commands::launch::exec_launch(
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
        Commands::Launch { server, account } => commands::launch::preset_launch(server, account),
        Commands::Inject => commands::inject::inject(),
    }
}
