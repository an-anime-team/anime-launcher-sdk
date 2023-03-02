use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::io::Write;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::consts::config_file;

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
#[tracing::instrument(level = "trace")]
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
#[tracing::instrument(level = "debug", ret)]
pub fn get_raw() -> anyhow::Result<Config> {
    tracing::debug!("Reading config data from file");

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
                    Err(err) => {
                        tracing::error!("Failed to decode config data from json format: {}", err.to_string());

                        Err(anyhow::anyhow!("Failed to decode config data from json format: {}", err.to_string()))
                    }
                }
            }

            // Otherwise create default config file
            else {
                update_raw(Config::default())?;

                Ok(Config::default())
            }
        },
        None => {
            tracing::error!("Failed to get config file path");

            Err(anyhow::anyhow!("Failed to get config file path"))
        }
    }
}

/// Update in-memory config data
/// 
/// Use `update_raw` if you want to update config file itself
#[tracing::instrument(level = "trace")]
pub fn update(config: Config) {
    tracing::trace!("Updating hot config record");

    unsafe {
        CONFIG = Some(config);
    }
}

/// Update config file
/// 
/// This method will also update in-memory config data
#[tracing::instrument(level = "debug", ret)]
pub fn update_raw(config: Config) -> anyhow::Result<()> {
    tracing::debug!("Updating config data");

    update(config.clone());

    match config_file() {
        Some(path) => {
            let mut file = File::create(&path)?;

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
        },
        None => {
            tracing::error!("Failed to get config file path");

            Err(anyhow::anyhow!("Failed to get config file path"))
        }
    }
}

/// Update config file from the in-memory saved config
#[tracing::instrument(level = "debug", ret)]
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

#[cfg(feature = "components")]
use crate::components::wine::{self, Version as WineVersion};

#[cfg(feature = "components")]
use crate::components::dxvk::{self, Version as DxvkVersion};

#[cfg(feature = "components")]
impl Config {
    pub fn try_get_selected_wine_info(&self) -> Option<WineVersion> {
        match &self.game.wine.selected {
            Some(selected) => {
                wine::get_groups()
                    .iter()
                    .flat_map(|group| group.versions.clone())
                    .find(|version| version.name.eq(selected))
            },
            None => None
        }
    }

    /// Try to get a path to the wine64 executable based on `game.wine.builds` and `game.wine.selected`
    /// 
    /// Returns `Some("wine64")` if:
    /// 1) `game.wine.selected = None`
    /// 2) wine64 installed and available in system
    pub fn try_get_wine_executable(&self) -> Option<PathBuf> {
        match self.try_get_selected_wine_info() {
            Some(selected) => Some(self.game.wine.builds.join(selected.name).join(selected.files.wine64)),
            None => {
                if crate::is_available("wine64") {
                    Some(PathBuf::from("wine64"))
                } else {
                    None
                }
            }
        }
    }

    /// Try to get DXVK version applied to wine prefix
    /// 
    /// Returns:
    /// 1) `Ok(Some(..))` if version was found
    /// 2) `Ok(None)` if version wasn't found, so too old or dxvk is not applied
    /// 3) `Err(..)` if failed to get applied dxvk version, likely because wrong prefix path specified
    pub fn try_get_selected_dxvk_info(&self) -> std::io::Result<Option<DxvkVersion>> {
        Ok(match wincompatlib::dxvk::Dxvk::get_version(&self.game.wine.prefix)? {
            Some(version) => {
                dxvk::get_groups()
                    .iter()
                    .flat_map(|group| group.versions.clone())
                    .find(move |dxvk| dxvk.version == version)
            },
            None => None
        })
    }
}

