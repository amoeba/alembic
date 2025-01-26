use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::bail;
use config::{Config, File};
use directories::BaseDirs;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const SETTINGS_VERSION: u32 = 1;
const SETTINGS_DIR_NAME: &str = "Alembic";
const SETTINGS_FILE_NAME: &str = "config.toml";
const ENV_PREFIX: &str = "ALEMBIC";

#[allow(dead_code)]
pub struct SettingsManager {
    pub settings: Arc<RwLock<AlembicSettings>>,
}

static SETTINGS: Lazy<SettingsManager> =
    Lazy::new(|| SettingsManager::new().expect("Failed to initialize settings"));

impl SettingsManager {
    pub fn new() -> anyhow::Result<Self> {
        let mut final_settings = AlembicSettings::new();
        let loaded_settings = ensure_settings()?;

        // TODO Merge

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
        let serialized = toml::to_string_pretty(&*settings)?;
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

    pub fn to_string() -> Result<String, toml::ser::Error> {
        let settings = SETTINGS.settings.read().unwrap();
        toml::to_string_pretty(&*settings)
    }
}

// TODO: Make this real
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Account {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct General {
    pub version: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlembicSettings {
    pub general: General,
    pub client: Client,
    pub selected_account: Option<usize>,
    pub accounts: Vec<Account>,
}

impl AlembicSettings {
    pub fn new() -> AlembicSettings {
        AlembicSettings {
            general: General {
                version: SETTINGS_VERSION,
            },
            client: Client {
                path: "".to_string(),
            },
            selected_account: None,
            accounts: vec![
                Account {
                    name: "AAAAA".to_string(),
                },
                Account {
                    name: "BBBBB".to_string(),
                },
                Account {
                    name: "CCCCC".to_string(),
                },
            ],
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
    match settings.general.version {
        1 => {
            println!("No-op");
        }
        _ => {
            let bad_version = settings.general.version;
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

    let serialized = toml::to_string_pretty(&AlembicSettings::new())?;
    fs::write(&settings_file_path, serialized)?;

    Ok(settings_file_path)
}

fn ensure_settings() -> anyhow::Result<AlembicSettings> {
    let path = ensure_settings_file()?;

    let builder = Config::builder()
        .add_source(File::with_name(path.to_str().unwrap()).required(false))
        .add_source(config::Environment::with_prefix(ENV_PREFIX))
        .build()?;

    Ok(builder.try_deserialize::<AlembicSettings>()?)
}
