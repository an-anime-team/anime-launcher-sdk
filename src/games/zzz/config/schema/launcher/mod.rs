use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

use anime_game_core::zzz::consts::GameEdition;

use crate::config::schema_blanks::prelude::*;
use crate::zzz::consts::launcher_dir;

#[cfg(feature = "environment-emulation")]
use crate::zzz::env_emulation::Environment;

pub mod prelude {
    pub use super::{
        Launcher,
        LauncherStyle,
        LauncherBehavior
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum LauncherStyle {
    Modern,
    Classic
}

impl Default for LauncherStyle {
    #[inline]
    fn default() -> Self {
        Self::Modern
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ordinalize, Serialize, Deserialize)]
pub enum LauncherBehavior {
    Nothing,
    Hide,
    Close
}

impl Default for LauncherBehavior {
    #[inline]
    fn default() -> Self {
        Self::Hide
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Launcher {
    pub language: String,
    pub edition: GameEdition,
    pub style: LauncherStyle,
    pub video_background: bool,
    pub background_index: u8,
    pub temp: Option<PathBuf>,
    pub repairer: Repairer,

    #[cfg(feature = "environment-emulation")]
    pub environment: Environment,

    pub behavior: LauncherBehavior
}

impl Default for Launcher {
    #[inline]
    fn default() -> Self {
        Self {
            language: String::from("en-us"),
            edition: GameEdition::from_system_lang(),
            style: LauncherStyle::default(),
            video_background: true,
            temp: launcher_dir().ok(),
            repairer: Repairer::default(),
            background_index: 0,

            #[cfg(feature = "discord-rpc")]
            discord_rpc: DiscordRpc::default(),

            #[cfg(feature = "environment-emulation")]
            environment: Environment::default(),

            behavior: LauncherBehavior::default()
        }
    }
}

impl From<&JsonValue> for Launcher {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            language: match value.get("language") {
                Some(value) => value.as_str().unwrap_or(&default.language).to_string(),
                None => default.language
            },

            edition: match value.get("edition") {
                Some(value) => serde_json::from_value(value.clone()).unwrap_or(default.edition),
                None => default.edition
            },

            style: match value.get("style") {
                Some(value) => serde_json::from_value(value.to_owned()).unwrap_or_default(),
                None => default.style
            },

            video_background: match value.get("video_background") {
                Some(value) => serde_json::from_value(value.to_owned()).unwrap_or(true),
                None => default.video_background
            },

            background_index: match value.get("background_index") {
                Some(value) => serde_json::from_value(value.to_owned()).unwrap_or_default(),
                None => default.background_index
            },

            temp: match value.get("temp") {
                Some(value) => {
                    if value.is_null() {
                        None
                    } else {
                        match value.as_str() {
                            Some(value) => Some(PathBuf::from(value)),
                            None => default.temp
                        }
                    }
                },
                None => default.temp
            },

            repairer: match value.get("repairer") {
                Some(value) => Repairer::from(value),
                None => default.repairer
            },

            #[cfg(feature = "environment-emulation")]
            environment: match value.get("environment") {
                Some(value) => serde_json::from_value(value.clone()).unwrap_or(default.environment),
                None => default.environment
            },

            behavior: match value.get("behavior") {
                Some(value) => serde_json::from_value(value.clone()).unwrap_or(default.behavior),
                None => default.behavior
            }
        }
    }
}
