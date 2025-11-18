use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use libalembic::{
    client_config::ClientConfig,
    launcher::{ClientLauncher, Launcher},
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

    /// Manage DLL configurations
    Dll {
        #[command(subcommand)]
        command: DllCommands,
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

    /// Select a server by index
    Select {
        /// Index of the server to select (from 'server list')
        index: usize,
    },

    /// Remove a server by index
    Remove {
        /// Index of the server to remove (from 'server list')
        index: usize,
    },
}

#[derive(Subcommand)]
enum DllCommands {
    /// Manually add a DLL configuration
    Add {
        /// Platform (windows or wine)
        #[arg(long)]
        platform: String,
        /// DLL type (alembic or decal)
        #[arg(long = "type")]
        dll_type: String,
        /// Path to the DLL
        #[arg(long)]
        path: String,
        /// Wine prefix path (required for wine platform)
        #[arg(long)]
        wine_prefix: Option<String>,
    },

    /// List all discovered DLLs (brief)
    List,

    /// Select a DLL by index
    Select {
        /// Index of the DLL to select
        index: usize,
    },

    /// Remove a DLL configuration by index
    Remove {
        /// Index of the DLL to remove
        index: usize,
    },

    /// Show detailed DLL configuration
    Show {
        /// Index of the DLL to show
        index: usize,
    },

    /// Scan for Decal installations
    Scan,
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
                AccountCommands::Remove { index } => account_remove(index),
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
                ClientCommands::Remove { index } => client_remove(index),
                ClientCommands::Show { index } => client_show(index),
                ClientCommands::Scan => client_scan(),
            },
            ConfigCommands::Server { command } => match command {
                ServerCommands::Add {
                    name,
                    hostname,
                    port,
                } => server_add(name, hostname, port),
                ServerCommands::List => server_list(),
                ServerCommands::Select { index } => server_select(index),
                ServerCommands::Remove { index } => server_remove(index),
            },
            ConfigCommands::Dll { command } => match command {
                DllCommands::Add {
                    platform,
                    dll_type,
                    path,
                    wine_prefix,
                } => dll_add(platform, dll_type, path, wine_prefix),
                DllCommands::List => dll_list(),
                DllCommands::Select { index } => dll_select(index),
                DllCommands::Remove { index } => dll_remove(index),
                DllCommands::Show { index } => dll_show(index),
                DllCommands::Scan => dll_scan(),
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
        }),
        "wine" => {
            let prefix =
                wine_prefix.ok_or_else(|| anyhow::anyhow!("Wine prefix required for wine mode"))?;
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

    // Get selected DLL (optional - if none selected, no injection will occur)
    let inject_config = SettingsManager::get(|s| {
        s.selected_dll
            .and_then(|idx| s.discovered_dlls.get(idx).cloned())
    });

    println!("Client: {}", client_config.display_name());
    println!(
        "Server: {} ({}:{})",
        server_info.name, server_info.hostname, server_info.port
    );
    println!("Account: {}", account_info.username);
    if let Some(ref dll) = inject_config {
        println!("DLL: {} ({})", dll.dll_type(), dll.dll_path().display());
    } else {
        println!("DLL: None (no injection)");
    }

    run_launcher(client_config, inject_config, server_info, account_info)
}

fn run_launcher(
    client_config: ClientConfig,
    inject_config: Option<libalembic::client_config::InjectConfig>,
    server_info: ServerInfo,
    account_info: Account,
) -> anyhow::Result<()> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout},
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
        Terminal,
    };
    use std::io::{self, BufRead, BufReader};
    use std::sync::mpsc::{channel, Receiver};
    use std::thread;
    use std::time::Duration;

    // Launch or simulate
    let (log_tx, log_rx): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();
    let (status_tx, status_rx): (
        std::sync::mpsc::Sender<ProcessStatus>,
        std::sync::mpsc::Receiver<ProcessStatus>,
    ) = channel();

    #[derive(Clone, Debug)]
    enum ProcessStatus {
        Starting,
        Running,
        Exited(i32),
        Error(String),
    }

    let mut launcher = Launcher::new(
        client_config.clone(),
        inject_config,
        server_info.clone(),
        account_info.clone(),
    );
    launcher.find_or_launch()?;
    launcher.inject()?;

    // Capture child process stdout/stderr on non-Windows platforms
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    if let Some(mut child) = launcher.take_child() {
        // Spawn thread for stdout
        if let Some(stdout) = child.stdout.take() {
            let tx = log_tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let timestamp = get_timestamp();
                        let _ = tx.send(format!("[{}] {}", timestamp, line));
                    }
                }
            });
        }

        // Spawn thread for stderr
        if let Some(stderr) = child.stderr.take() {
            let tx = log_tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let timestamp = get_timestamp();
                        let _ = tx.send(format!("[{}] {}", timestamp, line));
                    }
                }
            });
        }

        // Spawn thread to monitor process exit
        let status_tx_clone = status_tx.clone();
        let log_tx_clone = log_tx.clone();
        thread::spawn(move || {
            // Process started successfully, update status to Running
            let _ = status_tx_clone.send(ProcessStatus::Running);

            match child.wait() {
                Ok(exit_status) => {
                    let code = exit_status.code().unwrap_or(-1);
                    let timestamp = get_timestamp();
                    let _ = log_tx_clone
                        .send(format!("[{}] Process exited with code {}", timestamp, code));
                    let _ = status_tx_clone.send(ProcessStatus::Exited(code));
                }
                Err(e) => {
                    let timestamp = get_timestamp();
                    let _ = log_tx_clone
                        .send(format!("[{}] Error waiting for process: {}", timestamp, e));
                    let _ = status_tx_clone.send(ProcessStatus::Error(e.to_string()));
                }
            }
        });
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut show_logs = false;
    let mut running = true;
    let mut logs: Vec<String> = vec![];
    let mut process_status = ProcessStatus::Starting;

    while running {
        // Collect any new log messages from child process
        while let Ok(log_line) = log_rx.try_recv() {
            logs.push(log_line);
        }

        // Check for process status updates
        if let Ok(status) = status_rx.try_recv() {
            process_status = status;
        }

        // Render UI
        terminal.draw(|f| {
            let chunks = if show_logs {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(10), Constraint::Min(5)])
                    .split(f.area())
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(10)])
                    .split(f.area())
            };

            // Status panel
            let status_text = vec![
                Line::from(vec![
                    Span::raw("Client: "),
                    Span::raw(client_config.display_name()),
                ]),
                Line::from(vec![
                    Span::raw("Server: "),
                    Span::raw(format!(
                        "{} ({}:{})",
                        server_info.name, server_info.hostname, server_info.port
                    )),
                ]),
                Line::from(vec![
                    Span::raw("Account: "),
                    Span::raw(&account_info.username),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        match &process_status {
                            ProcessStatus::Starting => "● Starting...".to_string(),
                            ProcessStatus::Running => "● Running".to_string(),
                            ProcessStatus::Exited(code) => format!("● Exited (code {})", code),
                            ProcessStatus::Error(e) => format!("● Error: {}", e),
                        },
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from("Controls: Ctrl+C/q to quit"),
                Line::from("          ? to show logs"),
            ];

            let status_block = Paragraph::new(status_text).block(
                Block::default()
                    .title("Alembic")
                    .borders(Borders::ALL)
                    .padding(Padding::uniform(1)),
            );
            f.render_widget(status_block, chunks[0]);

            // Logs panel (if toggled)
            if show_logs {
                let log_items: Vec<ListItem> = logs
                    .iter()
                    .rev()
                    .take(chunks[1].height as usize - 2)
                    .rev()
                    .map(|log| ListItem::new(log.clone()))
                    .collect();

                let logs_list = List::new(log_items).block(
                    Block::default()
                        .title("Logs (Press ? to hide)")
                        .borders(Borders::ALL)
                        .padding(Padding::uniform(1)),
                );
                f.render_widget(logs_list, chunks[1]);
            }
        })?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        logs.push(format!("[{}] Shutting down...", get_timestamp()));
                        running = false;
                    }
                    KeyEvent {
                        code: KeyCode::Char('?'),
                        ..
                    } => {
                        show_logs = !show_logs;
                    }
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('Q'),
                        ..
                    } => {
                        logs.push(format!("[{}] Shutting down...", get_timestamp()));
                        running = false;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Cleanup launcher
    println!("Ejecting...");
    launcher.eject()?;
    println!("Exited.");

    Ok(())
}

fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = now.as_secs();
    let millis = now.subsec_millis();

    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
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
        let client_type = if config.is_wine() { "wine" } else { "Windows" };
        println!("{}{}: {} ({})", marker, idx, config.install_path().display(), client_type);
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
        }),
        "wine" => {
            let prefix =
                wine_prefix.ok_or_else(|| anyhow::anyhow!("Wine prefix required for wine mode"))?;
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

    // Check if there are any existing clients before we start adding
    let had_no_clients = SettingsManager::get(|s| s.clients.is_empty());

    for config in discovered {
        // Check if already exists
        let already_exists = SettingsManager::get(|s| {
            s.clients
                .iter()
                .any(|existing| existing.install_path() == config.install_path())
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
            // Auto-select if this is the first client being added
            let should_select = had_no_clients && added_count == 0;

            SettingsManager::modify(|settings| {
                settings.add_client(config.clone(), should_select);
                settings.is_configured = true;
            })?;

            if should_select {
                println!("✓ Added and selected!");
            } else {
                println!("✓ Added!");
            }
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

fn dll_scan() -> anyhow::Result<()> {
    println!("Scanning for Decal installations...");
    println!();

    let discovered_dlls = scanner::scan_for_decal_dlls()?;

    if discovered_dlls.is_empty() {
        println!("No Decal installations found.");
        println!("Make sure Decal's Inject.dll is installed.");
        return Ok(());
    }

    println!("Found Decal installation:");
    for dll in &discovered_dlls {
        println!("  Path: {}", dll.dll_path().display());
    }
    println!();

    // Add/update each found DLL
    SettingsManager::modify(|settings| {
        let had_no_dlls = settings.discovered_dlls.is_empty();

        for dll in discovered_dlls {
            settings.add_or_update_dll(dll);
        }

        // Auto-select first DLL if there were no DLLs before
        if had_no_dlls && !settings.discovered_dlls.is_empty() && settings.selected_dll.is_none() {
            settings.selected_dll = Some(0);
        }
    })?;

    println!("✓ Decal configuration saved!");
    println!();
    println!("Use 'alembic dll status' to see the configuration.");

    Ok(())
}

fn dll_list() -> anyhow::Result<()> {
    let (discovered_dlls, selected_dll) =
        SettingsManager::get(|s| (s.discovered_dlls.clone(), s.selected_dll));

    if discovered_dlls.is_empty() {
        println!("No DLLs configured.");
        println!("Run 'alembic dll scan' to discover DLLs.");
        return Ok(());
    }

    for (idx, dll) in discovered_dlls.iter().enumerate() {
        let is_selected = Some(idx) == selected_dll;
        let marker = if is_selected { " * " } else { "   " };

        let dll_variant = match dll {
            libalembic::client_config::InjectConfig::Wine(_) => "wine",
            libalembic::client_config::InjectConfig::Windows(_) => "Windows",
        };

        println!("{}{}: {} ({})", marker, idx, dll.dll_path().display(), dll_variant);
    }

    Ok(())
}

fn dll_remove(index: usize) -> anyhow::Result<()> {
    use std::io::{self, Write};

    let dll_info = SettingsManager::get(|s| {
        s.discovered_dlls.get(index).map(|dll| {
            (
                dll.dll_type().to_string(),
                dll.dll_path().display().to_string(),
            )
        })
    });

    let (dll_type, dll_path) = match dll_info {
        Some(info) => info,
        None => {
            println!("Invalid DLL index: {}", index);
            println!("Use 'alembic dll list' to see available DLLs.");
            return Ok(());
        }
    };

    println!("This will remove the following DLL configuration:");
    println!("  [{}] {} - {}", index, dll_type, dll_path);
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
        if index < settings.discovered_dlls.len() {
            settings.discovered_dlls.remove(index);

            // Adjust selected_dll if needed
            if let Some(selected) = settings.selected_dll {
                if selected == index {
                    settings.selected_dll = None;
                } else if selected > index {
                    settings.selected_dll = Some(selected - 1);
                }
            }
        }
    })?;

    println!();
    println!("✓ DLL configuration has been removed.");

    Ok(())
}

fn dll_add(
    platform: String,
    dll_type: String,
    dll_path: String,
    wine_prefix: Option<String>,
) -> anyhow::Result<()> {
    use libalembic::client_config::{DllType, InjectConfig, WindowsInjectConfig, WineInjectConfig};
    use std::path::PathBuf;

    // Parse DLL type
    let dll_type = match dll_type.to_lowercase().as_str() {
        "alembic" => DllType::Alembic,
        "decal" => DllType::Decal,
        _ => {
            anyhow::bail!(
                "Invalid DLL type: {}. Must be 'alembic' or 'decal'",
                dll_type
            );
        }
    };

    // Create the appropriate InjectConfig variant
    let inject_config = match platform.to_lowercase().as_str() {
        "windows" => InjectConfig::Windows(WindowsInjectConfig {
            dll_type,
            dll_path: PathBuf::from(dll_path),
        }),
        "wine" => {
            let wine_prefix = wine_prefix
                .ok_or_else(|| anyhow::anyhow!("--wine-prefix is required for wine platform"))?;

            InjectConfig::Wine(WineInjectConfig {
                dll_type,
                wine_prefix: PathBuf::from(wine_prefix),
                dll_path: PathBuf::from(dll_path),
            })
        }
        _ => {
            anyhow::bail!(
                "Invalid platform: {}. Must be 'windows' or 'wine'",
                platform
            );
        }
    };

    println!("Adding DLL configuration:");
    println!("  Type: {}", inject_config.dll_type());
    println!("  Path: {}", inject_config.dll_path().display());

    SettingsManager::modify(|settings| {
        settings.add_or_update_dll(inject_config);
    })?;

    println!();
    println!("✓ DLL configuration added!");

    Ok(())
}

fn dll_select(index: usize) -> anyhow::Result<()> {
    let dll_count = SettingsManager::get(|s| s.discovered_dlls.len());

    if index >= dll_count {
        println!("Invalid DLL index: {}", index);
        println!("Use 'alembic dll list' to see available DLLs.");
        return Ok(());
    }

    SettingsManager::modify(|settings| {
        settings.selected_dll = Some(index);
    })?;

    println!("✓ Selected DLL at index {}", index);

    Ok(())
}

fn dll_show(index: usize) -> anyhow::Result<()> {
    let dll = SettingsManager::get(|s| s.discovered_dlls.get(index).cloned());

    match dll {
        Some(dll) => {
            println!("DLL configuration (index {}):", index);
            println!();
            println!("{}", dll);
        }
        None => {
            println!("Invalid DLL index: {}", index);
            println!("Use 'alembic dll list' to see available DLLs.");
        }
    }

    Ok(())
}

fn inject() -> anyhow::Result<()> {
    use libalembic::client_config::ClientConfig;
    use std::process::Command;

    println!("Running cork to find and inject into acclient.exe...");
    println!();

    // Get selected client config
    let client_config = SettingsManager::get(|s| s.get_selected_client().cloned());
    let client_config = match client_config {
        Some(config) => config,
        None => {
            bail!("No client selected. Use 'alembic client select <index>' to select a client.");
        }
    };

    // Only Wine clients are supported for now
    let wine_config = match client_config {
        ClientConfig::Wine(config) => config,
        ClientConfig::Windows(_) => {
            bail!("Inject command currently only supports Wine clients");
        }
    };

    // Get selected DLL config
    let dll_config = SettingsManager::get(|s| s.get_selected_dll().cloned());
    let dll_path = match dll_config {
        Some(config) => config.dll_path().display().to_string(),
        None => {
            bail!(
                "No DLL selected. Use 'alembic dll select <index>' to select a DLL for injection."
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
        bail!("cork.exe not found at {:?}. Make sure it's built with: cargo build --package cork --target x86_64-pc-windows-gnu", cork_path);
    }

    println!("Client: {}", wine_config.display_name);
    println!("Wine prefix: {}", wine_config.prefix_path.display());
    println!("DLL: {}", dll_path);
    println!("Cork path: {}", cork_path.display());
    println!();

    // Run cork.exe under wine
    let mut cmd = Command::new(&wine_config.wine_executable);
    cmd.env("WINEPREFIX", &wine_config.prefix_path);

    for (key, value) in &wine_config.additional_env {
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
