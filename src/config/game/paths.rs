use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::config::launcher::GameEdition;
use crate::consts::launcher_dir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paths {
    pub global: PathBuf,
    pub china: PathBuf
}

impl Paths {
    #[inline]
    /// Get game path for given edition
    pub fn for_edition(&self, edition: impl Into<GameEdition>) -> &Path {
        match edition.into() {
            GameEdition::Global => self.global.as_path(),
            GameEdition::China => self.china.as_path()
        }
    }
}

impl Default for Paths {
    fn default() -> Self {
        let launcher_dir = launcher_dir().expect("Failed to get launcher dir");

        Self {
            global: launcher_dir.join(concat!("Ge", "nshi", "n Imp", "act")),
            china: launcher_dir.join(concat!("Yu", "anS", "hen"))
        }
    }
}

impl From<&JsonValue> for Paths {
    fn from(value: &JsonValue) -> Self {
        let default = Self::default();

        // SDK 0.5.11 (launcher 3.3.0) and earlier
        if value.is_string() {
            let path = PathBuf::from(value.as_str().unwrap());

            Self {
                china: match path.parent() {
                    Some(parent) => parent.join(concat!("Yu", "anS", "hen")),
                    None => default.china
                },
                global: path
            }
        }

        // SDK 0.5.12 and later
        else {
            Self {
                global: match value.get("global") {
                    Some(value) => match value.as_str() {
                        Some(value) => PathBuf::from(value),
                        None => default.global
                    },
                    None => default.global
                },
    
                china: match value.get("china") {
                    Some(value) => match value.as_str() {
                        Some(value) => PathBuf::from(value),
                        None => default.china
                    },
                    None => default.china
                }
            }
        }
    }
}
