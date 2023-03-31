use std::fs::File;
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::consts::config_file;

pub mod launcher;
pub mod game;
pub mod components;
pub mod patch;
pub mod resolution;

pub mod prelude {
    pub use super::launcher::prelude::*;
    pub use super::game::prelude::*;

    pub use super::components::Components;
    pub use super::patch::Patch;
    pub use super::resolution::Resolution;
}

use prelude::*;

static mut CONFIG: Option<Config> = None; 

#[tracing::instrument(level = "trace")]
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

#[tracing::instrument(level = "debug", ret)]
/// Get config data
/// 
/// This method will always load data directly from the file and update in-memory config
pub fn get_raw() -> anyhow::Result<Config> {
    tracing::debug!("Reading config data from file");

    let path = config_file()?;

    // Try to read config if the file exists
    if path.exists() {
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
            }

            Err(err) => {
                tracing::error!("Failed to decode config data from json format: {err}");

                Err(anyhow::anyhow!("Failed to decode config data from json format: {err}"))
            }
        }
    }

    // Otherwise create default config file
    else {
        update_raw(Config::default())?;

        Ok(Config::default())
    }
}

#[inline]
#[tracing::instrument(level = "trace")]
/// Update in-memory config data
/// 
/// Use `update_raw` if you want to update config file itself
pub fn update(config: Config) {
    tracing::trace!("Updating hot config record");

    unsafe {
        CONFIG = Some(config);
    }
}

#[tracing::instrument(level = "debug", ret)]
/// Update config file
/// 
/// This method will also update in-memory config data
pub fn update_raw(config: Config) -> anyhow::Result<()> {
    tracing::debug!("Updating config data");

    update(config.clone());

    let mut file = File::create(config_file()?)?;

    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            file.write_all(json.as_bytes())?;

            Ok(())
        },
        Err(err) => {
            tracing::error!("Failed to encode config data into json format: {}", err.to_string());

            Err(anyhow::anyhow!("Failed to encode config data into json format: {}", err.to_string()))
        }
    }
}

#[tracing::instrument(level = "debug", ret)]
/// Update config file from the in-memory saved config
pub fn flush() -> anyhow::Result<()> {
    tracing::debug!("Flushing config data");

    unsafe {
        match &CONFIG {
            Some(config) => update_raw(config.clone()),
            None => {
                tracing::error!("Config wasn't loaded into the memory");

                Err(anyhow::anyhow!("Config wasn't loaded into the memory"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Config {
    pub launcher: Launcher,
    pub game: Game,
    pub components: Components,
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

            components: match value.get("components") {
                Some(value) => Components::from(value),
                None => default.components
            },

            patch: match value.get("patch") {
                Some(value) => Patch::from(value),
                None => default.patch
            }
        }
    }
}

#[cfg(feature = "components")]
use crate::components::wine;

#[cfg(feature = "components")]
use crate::components::dxvk;

#[cfg(feature = "components")]
impl Config {
    #[inline]
    /// Try to get selected wine version
    /// 
    /// Returns:
    /// 1) `Ok(Some(..))` if version selected and found
    /// 2) `Ok(None)` if version wasn't found, so likely too old or just incorrect
    /// 3) `Err(..)` if failed to get selected wine version
    pub fn get_selected_wine(&self) -> anyhow::Result<Option<wine::Version>> {
        match &self.game.wine.selected {
            Some(selected) => wine::Version::find_in(&self.components.path, selected),
            None => Ok(None)
        }
    }

    #[inline]
    /// Try to get DXVK version applied to wine prefix
    /// 
    /// Returns:
    /// 1) `Ok(Some(..))` if version was found
    /// 2) `Ok(None)` if version wasn't found, so too old or dxvk is not applied
    /// 3) `Err(..)` if failed to get applied dxvk version, likely because wrong prefix path specified
    pub fn get_selected_dxvk(&self) -> anyhow::Result<Option<dxvk::Version>> {
        match wincompatlib::dxvk::Dxvk::get_version(&self.game.wine.prefix)? {
            Some(version) => dxvk::Version::find_in(&self.components.path, version),
            None => Ok(None)
        }
    }
}

