use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::bail;
use directories::BaseDirs;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::client_config::ClientConfig;
use crate::inject_config::InjectConfig;

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
    pub clients: Vec<ClientConfig>,

    /// Index of the currently selected client
    #[serde(default)]
    pub selected_client: Option<usize>,

    /// All discovered DLL injection configurations
    #[serde(default)]
    pub discovered_dlls: Vec<InjectConfig>,

    /// Index of the currently selected DLL for injection
    #[serde(default)]
    pub selected_dll: Option<usize>,

    pub selected_server: Option<usize>,
    pub selected_account: Option<usize>,
    pub accounts: Vec<Account>,
    pub servers: Vec<ServerInfo>,
}

impl AlembicSettings {
    pub fn new() -> AlembicSettings {
        AlembicSettings {
            version: SETTINGS_VERSION,
            is_configured: false,
            clients: vec![],
            selected_client: None,
            discovered_dlls: vec![],
            selected_dll: None,
            selected_account: None,
            selected_server: None,
            accounts: vec![],
            servers: vec![],
        }
    }

    /// Get the currently selected client config
    pub fn get_selected_client(&self) -> Option<&ClientConfig> {
        self.selected_client.and_then(|idx| self.clients.get(idx))
    }

    /// Get mutable reference to selected client
    pub fn get_selected_client_mut(&mut self) -> Option<&mut ClientConfig> {
        self.selected_client
            .and_then(|idx| self.clients.get_mut(idx))
    }

    /// Add a new client config and optionally select it
    pub fn add_client(&mut self, config: ClientConfig, select: bool) {
        self.clients.push(config);
        if select || self.selected_client.is_none() {
            self.selected_client = Some(self.clients.len() - 1);
        }
    }

    /// Remove a client config by index
    pub fn remove_client(&mut self, index: usize) -> Option<ClientConfig> {
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

    /// Get the selected DLL
    pub fn get_selected_dll(&self) -> Option<&InjectConfig> {
        self.selected_dll
            .and_then(|idx| self.discovered_dlls.get(idx))
    }

    /// Add or update a DLL configuration
    pub fn add_or_update_dll(&mut self, inject_config: InjectConfig) {
        // Check if we already have this DLL type
        if let Some(existing) = self
            .discovered_dlls
            .iter_mut()
            .find(|dll| dll.dll_type == inject_config.dll_type)
        {
            // Update existing
            *existing = inject_config;
        } else {
            // Add new
            self.discovered_dlls.push(inject_config);
        }
    }

    /// Remove a DLL by type
    pub fn remove_dll_by_type(&mut self, dll_type: crate::inject_config::DllType) {
        self.discovered_dlls
            .retain(|dll| dll.dll_type != dll_type);
        // If we removed the selected DLL, clear the selection
        if let Some(selected_idx) = self.selected_dll {
            if selected_idx >= self.discovered_dlls.len() {
                self.selected_dll = None;
            } else if self
                .discovered_dlls
                .get(selected_idx)
                .map(|dll| dll.dll_type)
                == Some(dll_type)
            {
                self.selected_dll = None;
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

        // DLLs
        self.discovered_dlls = new_settings.discovered_dlls;
        self.selected_dll = new_settings.selected_dll;

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
