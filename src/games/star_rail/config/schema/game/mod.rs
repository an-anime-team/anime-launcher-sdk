use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::config::schema_blanks::prelude::*;
use crate::star_rail::consts::launcher_dir;

crate::config_impl_wine_schema!(launcher_dir);
crate::config_impl_dxvk_schema!(launcher_dir);

pub mod enhancements;
pub mod paths;

#[cfg(feature = "fps-unlocker")]
pub mod fps_unlocker;

pub mod prelude {
    pub use super::Wine;
    pub use super::Dxvk;

    #[cfg(feature = "fps-unlocker")]
    pub use super::fps_unlocker::prelude::*;

    pub use super::enhancements::Enhancements;
    pub use super::paths::Paths;

    #[cfg(feature = "fps-unlocker")]
    pub use super::fps_unlocker::FpsUnlocker;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub path: Paths,
    pub wine: Wine,
    pub dxvk: Dxvk,
    pub enhancements: Enhancements,
    pub environment: HashMap<String, String>,
    pub command: Option<String>
}

impl Default for Game {
    #[inline]
    fn default() -> Self {
        Self {
            path: Paths::default(),
            wine: Wine::default(),
            dxvk: Dxvk::default(),
            enhancements: Enhancements::default(),
            environment: HashMap::new(),
            command: None
        }
    }
}

impl From<&JsonValue> for Game {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        Self {
            path: value.get("path")
                .map(Paths::from)
                .unwrap_or(default.path),

            wine: value.get("wine")
                .map(Wine::from)
                .unwrap_or(default.wine),

            dxvk: value.get("dxvk")
                .map(Dxvk::from)
                .unwrap_or(default.dxvk),

            enhancements: value.get("enhancements")
                .map(Enhancements::from)
                .unwrap_or(default.enhancements),

            environment: match value.get("environment") {
                Some(value) => match value.as_object() {
                    Some(values) => {
                        let mut vars = HashMap::new();

                        for (name, value) in values {
                            if let Some(value) = value.as_str() {
                                vars.insert(name.clone(), value.to_string());
                            }
                        }

                        vars
                    },
                    None => default.environment
                },
                None => default.environment
            },

            command: match value.get("command") {
                Some(value) => {
                    if value.is_null() {
                        None
                    } else {
                        match value.as_str() {
                            Some(value) => Some(value.to_string()),
                            None => default.command
                        }
                    }
                },
                None => default.command
            }
        }
    }
}
