use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use enum_ordinalize::Ordinalize;

use crate::config::schema_blanks::prelude::*;
use crate::pgr::consts::launcher_dir;

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
    pub style: LauncherStyle,
    pub temp: Option<PathBuf>,
    pub repairer: Repairer,
    pub behavior: LauncherBehavior
}

impl Default for Launcher {
    #[inline]
    fn default() -> Self {
        Self {
            language: String::from("en-us"),
            style: LauncherStyle::default(),
            temp: launcher_dir().ok(),
            repairer: Repairer::default(),
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

            style: match value.get("style") {
                Some(value) => serde_json::from_value(value.to_owned()).unwrap_or_default(),
                None => default.style
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

            behavior: match value.get("behavior") {
                Some(value) => serde_json::from_value(value.clone()).unwrap_or(default.behavior),
                None => default.behavior
            }
        }
    }
}
