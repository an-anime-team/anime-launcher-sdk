use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::io::Write;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::config_file;

pub mod launcher;
pub mod game;
pub mod patch;
pub mod resolution;

pub mod prelude {
    pub use super::launcher::prelude::*;
    pub use super::game::prelude::*;

    pub use super::patch::Patch;
    pub use super::resolution::Resolution;
}

use prelude::*;

static mut CONFIG: Option<Config> = None; 

/// Get config data
/// 
/// This method will load config from file once and store it into the memory.
/// If you know that the config file was updated - you should run `get_raw` method
/// that always loads config directly from the file. This will also update in-memory config
pub fn get() -> anyhow::Result<Config> {
    unsafe {
        match &CONFIG {
            Some(config) => Ok(config.clone()),
            None => get_raw()
        }
    }
}

/// Get config data
/// 
/// This method will always load data directly from the file and update in-memory config
pub fn get_raw() -> anyhow::Result<Config> {
    match config_file() {
        Some(path) => {
            // Try to read config if the file exists
            if Path::new(&path).exists() {
                let mut file = File::open(path)?;
                let mut json = String::new();

                file.read_to_string(&mut json)?;

                match serde_json::from_str(&json) {
                    Ok(config) => {
                        let config = Config::from(&config);

                        unsafe {
                            CONFIG = Some(config.clone());
                        }

                        Ok(config)
                    },
                    Err(err) => Err(anyhow::anyhow!("Failed to decode data from json format: {}", err.to_string()))
                }
            }

            // Otherwise create default config file
            else {
                update_raw(Config::default())?;

                Ok(Config::default())
            }
        },
        None => Err(anyhow::anyhow!("Failed to get config file path"))
    }
}

/// Update in-memory config data
/// 
/// Use `update_raw` if you want to update config file itself
pub fn update(config: Config) {
    unsafe {
        CONFIG = Some(config);
    }
}

/// Update config file
/// 
/// This method will also update in-memory config data
pub fn update_raw(config: Config) -> anyhow::Result<()> {
    update(config.clone());

    match config_file() {
        Some(path) => {
            let mut file = File::create(&path)?;

            match serde_json::to_string_pretty(&config) {
                Ok(json) => {
                    file.write_all(json.as_bytes())?;

                    Ok(())
                },
                Err(err) => Err(anyhow::anyhow!("Failed to encode data into json format: {}", err.to_string()))
            }
        },
        None => Err(anyhow::anyhow!("Failed to get config file path"))
    }
}

/// Update config file from the in-memory saved config
pub fn flush() -> anyhow::Result<()> {
    unsafe {
        match &CONFIG {
            Some(config) => update_raw(config.clone()),
            None => Err(anyhow::anyhow!("Config wasn't loaded into the memory"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub launcher: Launcher,
    pub game: Game,
    pub patch: Patch
}

impl From<&JsonValue> for Config {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            launcher: match value.get("launcher") {
                Some(value) => Launcher::from(value),
                None => default.launcher
            },

            game: match value.get("game") {
                Some(value) => Game::from(value),
                None => default.game
            },

            patch: match value.get("patch") {
                Some(value) => Patch::from(value),
                None => default.patch
            }
        }
    }
}
