use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

use anime_game_core::genshin::consts::GameEdition as CoreGameEdition;

use crate::consts::launcher_dir;

pub mod repairer;

#[cfg(feature = "discord-rpc")]
pub mod discord_rpc;

pub mod prelude {
    pub use super::Launcher;
    pub use super::repairer::Repairer;

    #[cfg(feature = "discord-rpc")]
    pub use super::discord_rpc::DiscordRpc;
}

use prelude::*;

#[derive(Ordinalize, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEdition {
    Global,
    China
}

impl Default for GameEdition {
    fn default() -> Self {
        #[allow(clippy::or_fun_call)]
        let locale = std::env::var("LC_ALL")
            .unwrap_or_else(|_| std::env::var("LC_MESSAGES")
            .unwrap_or_else(|_| std::env::var("LANG")
            .unwrap_or(String::from("en_us"))));

        if locale.len() > 4 && &locale[..5].to_ascii_lowercase() == "zh_cn" {
            Self::China
        } else {
            Self::Global
        }
    }
}

impl From<GameEdition> for CoreGameEdition {
    fn from(edition: GameEdition) -> Self {
        match edition {
            GameEdition::Global => CoreGameEdition::Global,
            GameEdition::China  => CoreGameEdition::China
        }
    }
}

impl From<CoreGameEdition> for GameEdition {
    fn from(edition: CoreGameEdition) -> Self {
        match edition {
            CoreGameEdition::Global => Self::Global,
            CoreGameEdition::China  => Self::China
        }
    }
}

// TODO: I can e.g. use `.classic` file to mark launcher style

#[derive(Ordinalize, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LauncherStyle {
    Modern,
    Classic
}

impl Default for LauncherStyle {
    fn default() -> Self {
        Self::Modern
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Launcher {
    pub language: String,
    pub temp: Option<PathBuf>,
    pub repairer: Repairer,
    pub edition: GameEdition,
    pub style: LauncherStyle,

    #[cfg(feature = "discord-rpc")]
    pub discord_rpc: DiscordRpc
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            language: String::from("en-us"),
            temp: launcher_dir().ok(),
            repairer: Repairer::default(),
            edition: GameEdition::default(),
            style: LauncherStyle::default(),

            #[cfg(feature = "discord-rpc")]
            discord_rpc: DiscordRpc::default()
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

            edition: match value.get("edition") {
                Some(value) => serde_json::from_value(value.clone()).unwrap_or(default.edition),
                None => default.edition
            },

            style: match value.get("style") {
                Some(value) => serde_json::from_value(value.clone()).unwrap_or(default.style),
                None => default.style
            },

            #[cfg(feature = "discord-rpc")]
            discord_rpc: match value.get("discord_rpc") {
                Some(value) => DiscordRpc::from(value),
                None => default.discord_rpc
            }
        }
    }
}
