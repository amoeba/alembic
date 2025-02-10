use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::bail;
use directories::BaseDirs;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const SETTINGS_VERSION: u32 = 1;
const SETTINGS_DIR_NAME: &str = "Alembic";
const SETTINGS_FILE_NAME: &str = "config.json";
const ENV_PREFIX: &str = "ALEMBIC";

#[allow(dead_code)]
pub struct SettingsManager {
    pub settings: Arc<RwLock<AlembicSettings>>,
}

static SETTINGS: Lazy<SettingsManager> =
    Lazy::new(|| SettingsManager::new().expect("Failed to initialize settings"));

impl SettingsManager {
    pub fn new() -> anyhow::Result<Self> {
        let final_settings = AlembicSettings::new();
        let loaded_settings = ensure_settings()?;

        // TODO Merge loaded_settings into final settings object

        Ok(Self {
            settings: Arc::new(RwLock::new(final_settings)),
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
        let mut settings = SETTINGS.settings.write().unwrap();
        f(&mut settings);
        Self::save()
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
    pub client: ClientInfo,
    pub dll: DllInfo,
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
            client: ClientInfo::default(),
            dll: DllInfo::default(),
            selected_account: None,
            selected_server: None,
            accounts: vec![],
            servers: vec![],
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

        // TODO: Top level
        self.version = new_settings.version.clone();
        self.is_configured = new_settings.is_configured.clone();

        // TODO: Client
        self.client = new_settings.client.clone();

        // TODO: DLL
        self.dll = new_settings.dll.clone();

        // TODO: Servers
        self.selected_server = new_settings.selected_server.clone();
        new_settings
            .servers
            .iter()
            .for_each(|a| self.servers.push(a.clone()));

        // TODO: Accounts
        self.selected_account = new_settings.selected_account.clone();
        new_settings
            .accounts
            .iter()
            .for_each(|a| self.accounts.push(a.clone()));

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

// TODO: Make this real
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Account {
    pub server_index: usize,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientInfo {
    pub workdir_path: String,
    pub client_path: String,
}

impl ClientInfo {
    fn default() -> ClientInfo {
        Self {
            workdir_path: "C:\\Turbine\\Asheron's Call\\".to_string(),
            client_path: "C:\\Turbine\\Asheron's Call\\acclient.exe".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DllInfo {
    pub dll_path: String,
}

impl DllInfo {
    fn default() -> DllInfo {
        Self {
            dll_path: "Alembic.dll".to_string(),
        }
    }
}

// TODO
pub fn merge_settings(target: &mut AlembicSettings, source: &AlembicSettings) {
    println!("Source: {source:?}");
    println!("Target: {target:?}");
    println!("TODO: Merging");
}

#[allow(dead_code)]
fn migrate_settings(mut settings: AlembicSettings) -> AlembicSettings {
    settings = settings; // Just disables warning about mut qualifier

    // Doesn't do anything right now
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
