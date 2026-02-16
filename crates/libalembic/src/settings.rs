use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::bail;
use directories::BaseDirs;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::client_config::{ClientConfig, LaunchCommand, WindowsClientConfig, WineClientConfig};
use crate::inject_config::InjectConfig;
use crate::validation::ValidationResult;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientConfigType {
    Windows(WindowsClientConfig),
    Wine(WineClientConfig),
}

/// DLL configuration associated with a client
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientDllConfig {
    /// The DLL injection configuration
    pub inject_config: InjectConfig,
}

impl ClientDllConfig {
    pub fn new(inject_config: InjectConfig) -> Self {
        Self { inject_config }
    }
}

impl ClientConfigType {
    pub fn name(&self) -> &str {
        match self {
            ClientConfigType::Windows(c) => &c.name,
            ClientConfigType::Wine(c) => &c.name,
        }
    }

    pub fn name_mut(&mut self) -> &mut String {
        match self {
            ClientConfigType::Windows(c) => &mut c.name,
            ClientConfigType::Wine(c) => &mut c.name,
        }
    }

    pub fn client_path(&self) -> &std::path::Path {
        match self {
            ClientConfigType::Windows(c) => &c.client_path,
            ClientConfigType::Wine(c) => &c.client_path,
        }
    }

    pub fn client_path_mut(&mut self) -> &mut std::path::PathBuf {
        match self {
            ClientConfigType::Windows(c) => &mut c.client_path,
            ClientConfigType::Wine(c) => &mut c.client_path,
        }
    }

    pub fn launch_command(&self) -> Option<&LaunchCommand> {
        match self {
            ClientConfigType::Windows(_) => None,
            ClientConfigType::Wine(c) => Some(&c.launch_command),
        }
    }

    pub fn launch_command_mut(&mut self) -> Option<&mut LaunchCommand> {
        match self {
            ClientConfigType::Windows(_) => None,
            ClientConfigType::Wine(c) => Some(&mut c.launch_command),
        }
    }

    pub fn install_path(&self) -> std::path::PathBuf {
        crate::client_config::windows_path_parent(self.client_path())
    }

    pub fn is_wine(&self) -> bool {
        matches!(self, ClientConfigType::Wine(_))
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, ClientConfigType::Windows(_))
    }

    pub fn validate(&self, inject_config: Option<&InjectConfig>) -> ValidationResult {
        match self {
            ClientConfigType::Windows(c) => c.validate(inject_config),
            ClientConfigType::Wine(c) => c.validate(inject_config),
        }
    }
}

impl std::fmt::Display for ClientConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::client_config::ClientConfig;

        match self {
            ClientConfigType::Windows(c) => ClientConfig::fmt_display(c, f),
            ClientConfigType::Wine(c) => ClientConfig::fmt_display(c, f),
        }
    }
}

const SETTINGS_VERSION: u32 = 1;
const SETTINGS_DIR_NAME: &str = "Alembic";
const SETTINGS_FILE_NAME: &str = "config.json";
#[allow(dead_code)]
const ENV_PREFIX: &str = "ALEMBIC";

#[allow(dead_code)]
pub struct SettingsManager {
    pub settings: Arc<RwLock<AlembicSettings>>,
}

static SETTINGS: Lazy<SettingsManager> =
    Lazy::new(|| SettingsManager::new().expect("Failed to initialize settings"));

impl SettingsManager {
    pub fn new() -> anyhow::Result<Self> {
        let loaded_settings = ensure_settings()?;

        Ok(Self {
            settings: Arc::new(RwLock::new(loaded_settings)),
        })
    }

    pub fn update_selected_account() {
        println!("supposedly updating...");
    }

    pub fn save() -> anyhow::Result<()> {
        let settings_path = ensure_settings_file()?;

        let settings = SETTINGS.settings.read().unwrap();
        let serialized = serde_json::to_string_pretty(&*settings)?;
        fs::write(settings_path, serialized)?;

        Ok(())
    }

    pub fn get<T, F>(f: F) -> T
    where
        F: Fn(&AlembicSettings) -> T,
        T: Clone,
    {
        let settings = SETTINGS.settings.read().unwrap();
        f(&settings)
    }

    pub fn modify<F>(f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut AlembicSettings),
    {
        let settings_path = ensure_settings_file()?;

        let serialized = {
            let mut settings = SETTINGS.settings.write().unwrap();
            f(&mut settings);
            serde_json::to_string_pretty(&*settings)?
        }; // Write lock is released here

        fs::write(settings_path, serialized)?;
        Ok(())
    }

    pub fn to_string() -> Result<std::string::String, serde_json::Error> {
        let settings = SETTINGS.settings.read().unwrap();
        serde_json::to_string_pretty(&*settings)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlembicSettings {
    pub version: u32,
    pub is_configured: bool,

    /// All discovered/configured client installations
    #[serde(default)]
    pub clients: Vec<ClientConfigType>,

    /// Index of the currently selected client
    #[serde(default)]
    pub selected_client: Option<usize>,

    pub selected_server: Option<usize>,
    pub selected_account: Option<usize>,
    pub accounts: Vec<Account>,
    pub servers: Vec<ServerInfo>,
}

impl Default for AlembicSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl AlembicSettings {
    pub fn new() -> AlembicSettings {
        AlembicSettings {
            version: SETTINGS_VERSION,
            is_configured: false,
            clients: vec![],
            selected_client: None,
            selected_account: None,
            selected_server: None,
            accounts: vec![],
            servers: vec![],
        }
    }

    /// Get the currently selected client config
    pub fn get_selected_client(&self) -> Option<&ClientConfigType> {
        self.selected_client.and_then(|idx| self.clients.get(idx))
    }

    /// Get mutable reference to selected client
    pub fn get_selected_client_mut(&mut self) -> Option<&mut ClientConfigType> {
        self.selected_client
            .and_then(|idx| self.clients.get_mut(idx))
    }

    /// Add a new client config and optionally select it
    pub fn add_client(&mut self, config: ClientConfigType, select: bool) {
        self.clients.push(config);
        if select || self.selected_client.is_none() {
            self.selected_client = Some(self.clients.len() - 1);
        }
    }

    /// Remove a client config by index
    pub fn remove_client(&mut self, index: usize) -> Option<ClientConfigType> {
        if index < self.clients.len() {
            let removed = self.clients.remove(index);

            // Adjust selected_client if needed
            if let Some(selected) = self.selected_client {
                if selected == index {
                    // Removed the selected client
                    self.selected_client = if self.clients.is_empty() {
                        None
                    } else {
                        Some(0) // Select first remaining
                    };
                } else if selected > index {
                    // Adjust index after removal
                    self.selected_client = Some(selected - 1);
                }
            }

            Some(removed)
        } else {
            None
        }
    }

    /// Get the selected server
    pub fn get_selected_server(&self) -> Option<&ServerInfo> {
        self.selected_server.and_then(|idx| self.servers.get(idx))
    }

    /// Get the selected account
    pub fn get_selected_account(&self) -> Option<&Account> {
        self.selected_account.and_then(|idx| self.accounts.get(idx))
    }

    /// Get the selected DLL for the selected client
    pub fn get_selected_dll(&self) -> Option<&InjectConfig> {
        self.get_selected_client().and_then(|client| {
            let selected_dll = match client {
                ClientConfigType::Windows(c) => c.selected_dll,
                ClientConfigType::Wine(c) => c.selected_dll,
            };
            selected_dll.and_then(|idx| {
                let dlls = match client {
                    ClientConfigType::Windows(c) => &c.dlls,
                    ClientConfigType::Wine(c) => &c.dlls,
                };
                dlls.get(idx)
            })
        })
    }

    /// Get mutable reference to DLLs for a specific client
    pub fn get_client_dlls_mut(&mut self, client_idx: usize) -> Option<&mut Vec<InjectConfig>> {
        self.clients.get_mut(client_idx).map(|client| match client {
            ClientConfigType::Windows(c) => &mut c.dlls,
            ClientConfigType::Wine(c) => &mut c.dlls,
        })
    }

    /// Get immutable reference to DLLs for a specific client
    pub fn get_client_dlls(&self, client_idx: usize) -> Option<&Vec<InjectConfig>> {
        self.clients.get(client_idx).map(|client| match client {
            ClientConfigType::Windows(c) => &c.dlls,
            ClientConfigType::Wine(c) => &c.dlls,
        })
    }

    /// Get selected DLL for a specific client
    pub fn get_client_selected_dll(&self, client_idx: usize) -> Option<&InjectConfig> {
        self.clients.get(client_idx).and_then(|client| {
            let selected_dll = match client {
                ClientConfigType::Windows(c) => c.selected_dll,
                ClientConfigType::Wine(c) => c.selected_dll,
            };
            selected_dll.and_then(|idx| {
                let dlls = match client {
                    ClientConfigType::Windows(c) => &c.dlls,
                    ClientConfigType::Wine(c) => &c.dlls,
                };
                dlls.get(idx)
            })
        })
    }

    /// Add a DLL to a specific client
    pub fn add_dll_to_client(&mut self, client_idx: usize, inject_config: InjectConfig) -> bool {
        if let Some(client) = self.clients.get_mut(client_idx) {
            let dlls = match client {
                ClientConfigType::Windows(c) => &mut c.dlls,
                ClientConfigType::Wine(c) => &mut c.dlls,
            };
            dlls.push(inject_config);
            true
        } else {
            false
        }
    }

    /// Remove a DLL from a specific client by index
    pub fn remove_dll_from_client(&mut self, client_idx: usize, dll_idx: usize) -> bool {
        if let Some(client) = self.clients.get_mut(client_idx) {
            match client {
                ClientConfigType::Windows(c) => {
                    if dll_idx < c.dlls.len() {
                        c.dlls.remove(dll_idx);
                        // Adjust selected_dll if needed
                        if let Some(selected) = c.selected_dll {
                            if selected == dll_idx {
                                c.selected_dll = None;
                            } else if selected > dll_idx {
                                c.selected_dll = Some(selected - 1);
                            }
                        }
                        true
                    } else {
                        false
                    }
                }
                ClientConfigType::Wine(c) => {
                    if dll_idx < c.dlls.len() {
                        c.dlls.remove(dll_idx);
                        // Adjust selected_dll if needed
                        if let Some(selected) = c.selected_dll {
                            if selected == dll_idx {
                                c.selected_dll = None;
                            } else if selected > dll_idx {
                                c.selected_dll = Some(selected - 1);
                            }
                        }
                        true
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }

    /// Set the selected DLL for a specific client
    pub fn select_dll_for_client(&mut self, client_idx: usize, dll_idx: Option<usize>) {
        if let Some(client) = self.clients.get_mut(client_idx) {
            match client {
                ClientConfigType::Windows(c) => c.selected_dll = dll_idx,
                ClientConfigType::Wine(c) => c.selected_dll = dll_idx,
            }
        }
    }
}

impl AlembicSettings {
    pub fn load(&mut self) -> anyhow::Result<()> {
        let dir = ensure_settings_dir()?;
        let settings_file_path = dir.join(SETTINGS_FILE_NAME);

        println!("Loading settings from {settings_file_path:?}");

        // Just stop now if the file doesn't exist
        if !settings_file_path.exists() {
            println!("Settings file doesn't exist, not loading.");

            return Ok(());
        }

        // Otherwise read in and merge
        let file_contents = fs::read_to_string(settings_file_path)?;
        let new_settings: AlembicSettings = serde_json::from_str(&file_contents)?;

        // Top level
        self.version = new_settings.version;
        self.is_configured = new_settings.is_configured;

        // Clients
        self.clients = new_settings.clients;
        self.selected_client = new_settings.selected_client;

        // Servers
        self.selected_server = new_settings.selected_server;
        self.servers = new_settings.servers;

        // Accounts
        self.selected_account = new_settings.selected_account;
        self.accounts = new_settings.accounts;

        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = ensure_settings_dir()?;
        let settings_file_path = dir.join(SETTINGS_FILE_NAME);
        let serialized = serde_json::to_string_pretty(&self)?;

        println!("Saving settings to {settings_file_path:?}");

        Ok(fs::write(&settings_file_path, serialized)?)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub hostname: String,
    pub port: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Account {
    pub server_index: usize,
    pub username: String,
    pub password: String,
}

// TODO
pub fn merge_settings(target: &mut AlembicSettings, source: &AlembicSettings) {
    println!("Source: {source:?}");
    println!("Target: {target:?}");
    println!("TODO: Merging");
}

#[allow(dead_code)]
fn migrate_settings(settings: AlembicSettings) -> AlembicSettings {
    // Doesn't do anything right now
    // When migrations are needed, this function will be updated
    match settings.version {
        1 => {
            println!("No-op");
        }
        _ => {
            let bad_version = settings.version;
            panic!("Unsupported settings file version: {bad_version}.")
        }
    }

    settings
}

pub fn get_settings_dir() -> anyhow::Result<PathBuf> {
    let base_dir = match BaseDirs::new() {
        Some(dir) => dir,
        None => {
            bail!("Failed to get BaseDirs. Not loading existing settings.");
        }
    };

    Ok(base_dir.config_dir().join(SETTINGS_DIR_NAME))
}

fn ensure_settings_dir() -> anyhow::Result<PathBuf> {
    let settings_dir = get_settings_dir()?;
    fs::create_dir_all(&settings_dir)?;

    Ok(settings_dir)
}

fn ensure_settings_file() -> anyhow::Result<PathBuf> {
    let dir = ensure_settings_dir()?;
    let settings_file_path = dir.join(SETTINGS_FILE_NAME);

    if fs::exists(&settings_file_path)? {
        return Ok(settings_file_path);
    }

    let serialized = serde_json::to_string_pretty(&AlembicSettings::new())?;
    fs::write(&settings_file_path, serialized)?;

    Ok(settings_file_path)
}

fn ensure_settings() -> anyhow::Result<AlembicSettings> {
    let path = ensure_settings_file()?;
    let contents = fs::read_to_string(path)?;
    let settings = serde_json::from_str(&contents)?;

    Ok(settings)
}
