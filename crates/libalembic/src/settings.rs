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

    pub fn dlls(&self) -> &Vec<InjectConfig> {
        match self {
            ClientConfigType::Windows(c) => &c.dlls,
            ClientConfigType::Wine(c) => &c.dlls,
        }
    }

    pub fn dlls_mut(&mut self) -> &mut Vec<InjectConfig> {
        match self {
            ClientConfigType::Windows(c) => &mut c.dlls,
            ClientConfigType::Wine(c) => &mut c.dlls,
        }
    }

    pub fn selected_dll(&self) -> Option<usize> {
        match self {
            ClientConfigType::Windows(c) => c.selected_dll,
            ClientConfigType::Wine(c) => c.selected_dll,
        }
    }

    pub fn selected_dll_mut(&mut self) -> &mut Option<usize> {
        match self {
            ClientConfigType::Windows(c) => &mut c.selected_dll,
            ClientConfigType::Wine(c) => &mut c.selected_dll,
        }
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
        self.get_selected_client()
            .and_then(|client| client.selected_dll().and_then(|idx| client.dlls().get(idx)))
    }

    /// Get mutable reference to DLLs for a specific client
    pub fn get_client_dlls_mut(&mut self, client_idx: usize) -> Option<&mut Vec<InjectConfig>> {
        self.clients.get_mut(client_idx).map(|c| c.dlls_mut())
    }

    /// Get immutable reference to DLLs for a specific client
    pub fn get_client_dlls(&self, client_idx: usize) -> Option<&Vec<InjectConfig>> {
        self.clients.get(client_idx).map(|c| c.dlls())
    }

    /// Get selected DLL for a specific client
    pub fn get_client_selected_dll(&self, client_idx: usize) -> Option<&InjectConfig> {
        self.clients
            .get(client_idx)
            .and_then(|client| client.selected_dll().and_then(|idx| client.dlls().get(idx)))
    }

    /// Add a DLL to a specific client, skipping if a DLL with the same path already exists
    pub fn add_dll_to_client(&mut self, client_idx: usize, inject_config: InjectConfig) -> bool {
        if let Some(client) = self.clients.get_mut(client_idx) {
            let already_exists = client
                .dlls()
                .iter()
                .any(|existing| existing.dll_path == inject_config.dll_path);
            if already_exists {
                return false;
            }
            client.dlls_mut().push(inject_config);
            true
        } else {
            false
        }
    }

    /// Remove a DLL from a specific client by index
    pub fn remove_dll_from_client(&mut self, client_idx: usize, dll_idx: usize) -> bool {
        if let Some(client) = self.clients.get_mut(client_idx) {
            let dlls = client.dlls_mut();
            if dll_idx < dlls.len() {
                dlls.remove(dll_idx);
                // Adjust selected_dll if needed
                let selected_dll = client.selected_dll_mut();
                if let Some(selected) = *selected_dll {
                    if selected == dll_idx {
                        *selected_dll = None;
                    } else if selected > dll_idx {
                        *selected_dll = Some(selected - 1);
                    }
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Set the selected DLL for a specific client
    pub fn select_dll_for_client(&mut self, client_idx: usize, dll_idx: Option<usize>) {
        if let Some(client) = self.clients.get_mut(client_idx) {
            *client.selected_dll_mut() = dll_idx;
        }
    }
}

impl AlembicSettings {
    pub fn load(&mut self) -> anyhow::Result<()> {
        let path = ensure_settings_file()?;
        let file_contents = fs::read_to_string(path)?;
        *self = serde_json::from_str(&file_contents)?;
        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let settings_file_path = ensure_settings_file()?;
        let serialized = serde_json::to_string_pretty(&self)?;
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
